{
  description = "Brongan's attempt at signal";
  inputs = {
    # Recent Flutter versions are only in unstable.
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs = {
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachSystem ["x86_64-linux"] (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
          overlays = [(import rust-overlay)];
        };
        sqliteStatic = pkgs.pkgsStatic.sqlite;
        inherit (pkgs) lib;
        frbCodegenVersion = "2.13.0";
        frbCodegenSrc = pkgs.fetchFromGitHub {
          owner = "fzyzcjy";
          repo = "flutter_rust_bridge";
          tag = "v${frbCodegenVersion}";
          hash = "sha256-NMM5QyqoduhXMpV9b6b3qRpfwqWtHkoucVN4xO81+fw=";
          fetchSubmodules = true;
        };
        frbCodegen = pkgs.flutter_rust_bridge_codegen.overrideAttrs {
          version = frbCodegenVersion;
          src = frbCodegenSrc;
          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            pname = "flutter_rust_bridge_codegen";
            version = frbCodegenVersion;
            src = frbCodegenSrc;
            hash = "sha256-xxdBo5rxuWiq5YMRPpVp2+0JX1lKvvzrT8z5Rq8S9g0=";
          };
        };
        toolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = ["rust-src"];
          # Android ABIs Cargokit may request in debug builds; musl is the server.
          targets = [
            "aarch64-linux-android"
            "armv7-linux-androideabi"
            "i686-linux-android"
            "x86_64-linux-android"
            "x86_64-unknown-linux-musl"
          ];
        };
        # Fake rustup. Cargokit drives the toolchain through rustup (`toolchain
        # list`, `target add`, `rustup run <tc> cargo ...`), but this flake pins
        # the toolchain as a store path, so the stub answers "installed" and
        # passes through to the cargo on PATH. Dev-shell/test path only; the apk
        # shims cargokit out (see apk postPatch).
        rustup = pkgs.writeShellScriptBin "rustup" ''
          case "$1" in
            run)
              shift 2
              exec "$@"
              ;;
            toolchain)
              case "$2" in
                list) echo "nightly-x86_64-unknown-linux-gnu (default)" ;;
                install) exit 0 ;;
              esac
              ;;
            target)
              case "$2" in
                list)
                  printf '%s\n' \
                    aarch64-linux-android \
                    armv7-linux-androideabi \
                    i686-linux-android \
                    x86_64-linux-android \
                    x86_64-unknown-linux-gnu \
                    x86_64-unknown-linux-musl
                  ;;
                add)
                  echo "rustup shim: requested target is not in the pinned toolchain" >&2
                  exit 1
                  ;;
              esac
              ;;
            component) exit 0 ;;
          esac
        '';
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        # Sliced workspace sources: full code for the crates in `keep`, but only
        # the manifest plus an empty stub target for the other members. Cargo
        # needs every member's manifest to resolve the workspace and verify
        # Cargo.lock, yet only compiles the requested package's graph -- so
        # code edits in excluded crates cannot invalidate the build, while
        # manifest edits (which affect the lock) still do.
        rustMembers = ["client" "gossamer" "hub" "proto" "protocol" "server"];
        rustStubTarget = {
          client = "src/lib.rs";
          gossamer = "src/lib.rs";
          hub = "src/lib.rs";
          proto = "src/lib.rs";
          protocol = "src/lib.rs";
          server = "src/main.rs";
        };
        rustSrcFor = keep: let
          stubbed = lib.subtractLists keep rustMembers;
          filtered = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions (
              [./Cargo.toml ./Cargo.lock]
              ++ map (m: ./native + "/${m}") keep
              ++ map (m: ./native + "/${m}/Cargo.toml") stubbed
            );
          };
        in
          pkgs.runCommand "rust-src-${lib.concatStringsSep "-" keep}" {} ''
            cp -r ${filtered} $out
            chmod -R u+w $out
            ${lib.concatMapStrings (m: ''
              mkdir -p "$out/native/${m}/$(dirname "${rustStubTarget.${m}}")"
              : > "$out/native/${m}/${rustStubTarget.${m}}"
            '') stubbed}
          '';
        hubRustSrc = rustSrcFor ["hub" "client" "proto" "protocol"];
        serverRustSrc = rustSrcFor ["server" "client" "gossamer" "proto" "protocol"];
        # native/ is deliberately absent: the apk consumes prebuilt libhub.so
        # derivations (hubAndroidLibs) instead of compiling Rust in-sandbox.
        # nix/pubspec.lock.json is included because preBuild reads it from $src;
        # nix/gradle-deps.json enters via the mitmCache input instead.
        apkSrc = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./android
            ./lib
            ./rust_builder
            ./nix/pubspec.lock.json
            ./pubspec.yaml
            ./pubspec.lock
            ./firebase.json
          ];
        };
        # Must equal Flutter's pinned NDK default (FlutterExtension.kt); ndkBin
        # bakes this string into the clang paths below.
        androidNdkVersion = "28.2.13676358";
        androidComposition = pkgs.androidenv.composeAndroidPackages {
          # rust_builder plugin compiles against API 34, flutter_local_notifications
          # against API 35, and Flutter 3.44 builds the app against API 36.
          platformVersions = ["34" "35" "36"];
          buildToolsVersions = ["35.0.0"];
          # Required: AGP auto-detects android/app/src/main/cpp/CMakeLists.txt
          # and generates a configureCMakeRelease task, which needs this exact
          # cmake in the SDK (removing it fails with CXX1300).
          cmakeVersions = ["3.22.1"];
          includeNDK = true;
          ndkVersions = [androidNdkVersion];
        };
        androidSdk = androidComposition.androidsdk;
        androidHome = "${androidSdk}/libexec/android-sdk";
        # Two Flutter instances: the apk build only needs android artifacts,
        # while the dev shell and integration tests also need linux desktop.
        # Keeping them separate keeps desktop artifacts out of the apk closure.
        flutterAndroid = pkgs.flutter.override {
          supportedTargetFlutterPlatforms = [
            "universal"
            "android"
          ];
        };
        flutterDevelopment = pkgs.flutter.override {
          supportedTargetFlutterPlatforms = [
            "universal"
            "android"
            "linux"
          ];
        };
        # Repo-pinned Gradle. Must stay in lockstep with the version in
        # android/gradle/wrapper/gradle-wrapper.properties (used by non-nix
        # builds). AGP 8.x requires JDK 17, matching JAVA_HOME everywhere.
        gradleUnwrapped = pkgs.gradle-packages.mkGradle {
          version = "8.14.3";
          hash = "sha256-vXEQIhNJMGCVbsIp2Ua+7lcVjb2J0OYrkbyg+ixfNTE=";
          defaultJava = pkgs.jdk17;
        };
        gradle = pkgs.callPackage pkgs.gradle-packages.wrapGradle {
          gradle-unwrapped = gradleUnwrapped;
        };
        # nixpkgs' init script that redirects all maven repositories into the
        # offline deps derivation.
        gradleInitScript = "${nixpkgs.outPath}/pkgs/development/tools/build-managers/gradle/init-build.gradle";
        # The gradlew that postPatch drops into android/. In a nix build one of
        # two mitm proxies is live (recording during deps regen, replaying
        # during the real build), signalled by MITM_CACHE_ADDRESS; route gradle
        # through it and trust its ephemeral CA. With no proxy, fall back to
        # --offline so a network attempt fails fast instead of hanging.
        gradlew = pkgs.writeShellScript "gradlew" ''
          set -eu
          # During deps recording, fetchDeps exports its own init script via
          # $gradleInitScript; otherwise use the baked-in repo-redirect one.
          initScript="''${gradleInitScript:-${gradleInitScript}}"
          extra=(
            --no-daemon
            --console plain
            --init-script "$initScript"
          )
          if [[ -n "''${MITM_CACHE_ADDRESS:-}" ]]; then
            truststore="''${TMPDIR:-/tmp}/gradle-mitm-keystore"
            # Recreate the truststore each time: keytool fails when the alias is
            # already present, and with `set -e` plus suppressed output that turns a
            # second gradle invocation in the same build into a silent exit 1.
            rm -f "$truststore"
            ${pkgs.jdk17}/bin/keytool -importcert -noprompt \
              -file "$MITM_CACHE_CA" -alias nix-mitm \
              -keystore "$truststore" -storepass nix-build >/dev/null 2>&1
            extra+=(
              "-Dhttp.proxyHost=$MITM_CACHE_HOST"
              "-Dhttp.proxyPort=$MITM_CACHE_PORT"
              "-Dhttps.proxyHost=$MITM_CACHE_HOST"
              "-Dhttps.proxyPort=$MITM_CACHE_PORT"
              "-Djavax.net.ssl.trustStore=$truststore"
              "-Djavax.net.ssl.trustStorePassword=nix-build"
            )
          else
            extra+=(--offline)
          fi
          exec ${gradle}/bin/gradle "''${extra[@]}" "$@"
        '';
        developmentEnvironment = ''
          export FLUTTER_ROOT=${flutterDevelopment}
          export PKG_CONFIG_PATH=${lib.getDev pkgs.sqlite}/lib/pkgconfig
        '';
        mkApp = name: runtimeInputs: text: let
          package = pkgs.writeShellApplication {
            inherit name runtimeInputs;
            text = developmentEnvironment + text;
          };
        in {
          type = "app";
          program = "${package}/bin/${name}";
        };
        nativeDevelopmentTools = [
          pkgs.pkg-config
          pkgs.protobuf
          toolchain
        ];
        args = {
          src = serverRustSrc;
          version = "0.1.0";
          strictDeps = true;
          cargoExtraArgs = "--package=server";
          nativeBuildInputs = with pkgs; [pkg-config protobuf];
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
          buildInputs = [sqliteStatic];
          pname = "server";
        };
        # libhub.so cross-compiled per Android ABI in dedicated derivations so
        # the Rust dependency tree is compiled once per target and cached; the
        # apk derivation only copies the finished libraries (see the cargokit
        # shim in postPatch). The clang wrapper name pins the Android API level
        # and must match the app's minSdkVersion (Flutter 3.44 default: 24).
        androidMinSdk = "24";
        ndkBin = "${androidHome}/ndk/${androidNdkVersion}/toolchains/llvm/prebuilt/linux-x86_64/bin";
        androidAbis = {
          "arm64-v8a" = {
            triple = "aarch64-linux-android";
            cc = "aarch64-linux-android${androidMinSdk}-clang";
          };
          "armeabi-v7a" = {
            triple = "armv7-linux-androideabi";
            cc = "armv7a-linux-androideabi${androidMinSdk}-clang";
          };
          "x86_64" = {
            triple = "x86_64-linux-android";
            cc = "x86_64-linux-android${androidMinSdk}-clang";
          };
        };
        hubLibFor = abi: target: let
          shoutTriple = lib.toUpper (builtins.replaceStrings ["-"] ["_"] target.triple);
          snakeTriple = builtins.replaceStrings ["-"] ["_"] target.triple;
          hubArgs =
            {
              src = hubRustSrc;
              pname = "hub-${abi}";
              version = "0.1.0";
              strictDeps = true;
              doCheck = false;
              cargoExtraArgs = "--package=hub";
              CARGO_BUILD_TARGET = target.triple;
              nativeBuildInputs = [pkgs.protobuf];
            }
            // {
              "CARGO_TARGET_${shoutTriple}_LINKER" = "${ndkBin}/${target.cc}";
              "CC_${snakeTriple}" = "${ndkBin}/${target.cc}";
              "AR_${snakeTriple}" = "${ndkBin}/llvm-ar";
            };
        in
          craneLib.buildPackage (hubArgs
            // {
              cargoArtifacts = craneLib.buildDepsOnly hubArgs;
            });
        hubAndroidLibs = pkgs.linkFarm "hub-android-libs" (
          lib.mapAttrsToList (abi: target: {
            name = "${abi}/libhub.so";
            path = "${hubLibFor abi target}/lib/libhub.so";
          })
          androidAbis
        );
        nativeArtifacts = craneLib.buildDepsOnly args;
        myServer = craneLib.buildPackage (args
          // {
            cargoArtifacts = nativeArtifacts;
            # Strip the toolchain store path out of the binary so the runtime
            # closure (and docker image) doesn't retain the multi-GB compiler.
            postFixup = ''
              ${pkgs.removeReferencesTo}/bin/remove-references-to \
                -t ${toolchain} "$out/bin/server"
            '';
          });
        apk = pkgs.flutter.buildFlutterApplication {
          pname = "brongnal-apk";
          version = "1.0.0";
          src = apkSrc;
          # buildFlutterApplication has no android target; "linux" selects its
          # pub-cache/package-config machinery, while postPatch/preBuild/
          # buildPhase perform the actual android build.
          targetFlutterPlatform = "linux";
          pubspecLock = lib.importJSON ./nix/pubspec.lock.json;

          # cmake + ninja: AGP auto-detects android/app/src/main/cpp/CMakeLists.txt
          # and generates a configureCMakeRelease task that needs both.
          nativeBuildInputs = [
            androidSdk
            gradle
            pkgs.cmake
            pkgs.jdk17
            pkgs.ninja
          ];

          # The recorded maven universe (nix/gradle-deps.json), replayed offline.
          # bwrapFlags because the recording sandbox is otherwise bare -- no $PWD
          # bind, no /bin/sh -- and gradle assumes both exist.
          mitmCache = gradle.fetchDeps {
            pkg = apk;
            data = ./nix/gradle-deps.json;
            bwrapFlags = ''--ro-bind "$PWD" "$PWD" --dir /bin --symlink ${pkgs.runtimeShell} /bin/sh'';
          };
          # Record the dependency set for nix/gradle-deps.json by running the
          # exact build the offline derivation runs. A complete `flutter build
          # apk` through the recording proxy captures everything that build
          # requests -- runtime configurations and each project's buildscript
          # classpath alike -- so no separate resolve-all task is needed.
          # No --no-pub: the pub step is what regenerates GeneratedPluginRegistrant
          # in release mode (excluding dev-dependency plugins such as
          # integration_test); gradle unconditionally drops those plugin projects
          # from release builds, so skipping the regeneration leaves the
          # registrant referencing classes that are not on the classpath. The
          # preceding offline pub get satisfies pub's up-to-date check, so no
          # network resolution happens.
          gradleUpdateScript = ''
            runHook preBuild
            ${flutterAndroid}/bin/flutter build apk --release
          '';

          ANDROID_HOME = androidHome;
          ANDROID_SDK_ROOT = androidHome;
          JAVA_HOME = pkgs.jdk17.home;
          # Suppress buildFlutterApplication's linux cmake configure phase (we
          # do the android build ourselves); the app's own cmake runs via gradle.
          dontUseCmakeConfigure = true;

          postPatch = ''
            # Gradle's contract with cargokit: exec this script with env vars
            # naming the wanted platforms and an output dir; it puts a
            # libhub.so per ABI there. Satisfy it by copying the prebuilt
            # libraries instead of bootstrapping dart/rustup/cargo in-sandbox.
            cat > rust_builder/cargokit/run_build_tool.sh <<'EOF'
            #!${pkgs.runtimeShell}
            set -eu
            [ "$1" = build-gradle ] || {
              echo "cargokit shim: unsupported command $1" >&2
              exit 1
            }
            IFS=,
            for platform in $CARGOKIT_TARGET_PLATFORMS; do
              case "$platform" in
                android-arm) abi=armeabi-v7a ;;
                android-arm64) abi=arm64-v8a ;;
                android-x64) abi=x86_64 ;;
                *)
                  echo "cargokit shim: no prebuilt libhub.so for $platform" >&2
                  exit 1
                  ;;
              esac
              mkdir -p "$CARGOKIT_OUTPUT_DIR/$abi"
              cp ${hubAndroidLibs}/"$abi"/libhub.so "$CARGOKIT_OUTPUT_DIR/$abi/"
            done
            EOF
            chmod +x rust_builder/cargokit/run_build_tool.sh
            cp ${gradlew} android/gradlew
            chmod +x android/gradlew
            # Use the SDK's aapt2 binary rather than the one maven ships as a
            # prebuilt (prebuilt ELF binaries don't run in the nix sandbox).
            echo 'android.aapt2FromMavenOverride=${androidHome}/build-tools/35.0.0/aapt2' \
              >> android/gradle.properties
            # Drop release signing and let it sign with the debug key, so the
            # real keystore/secrets never enter the nix store. This is why the
            # nix apk and a dev-shell release build differ at signing.
            sed -i '/signingConfig signingConfigs\.release/d' \
              android/app/build.gradle
            rm -f android/key.properties android/local.properties
            cat > android/local.properties <<EOF
            flutter.sdk=${flutterAndroid}
            sdk.dir=${androidHome}
            EOF
          '';

          dontDartBuild = true;
          # Rewrite the nix-generated package config into pubspec_overrides.yaml
          # path overrides, so `flutter pub get --offline` resolves every hosted
          # package from the store instead of pub.dev; hub points at rust_builder.
          preBuild = ''
            ${pkgs.jq}/bin/jq --slurpfile lock nix/pubspec.lock.json '
              {
                dependency_overrides: (
                  .packages
                  | map(select($lock[0].packages[.name].source == "hosted"))
                  | map({
                      key: .name,
                      value: { path: (.rootUri | sub("^file://"; "")) }
                    })
                  | from_entries
                  | .hub = { path: "rust_builder" }
                )
              }
            ' .dart_tool/package_config.json > pubspec_overrides.yaml
            cp --remove-destination "$pubspecLockFilePath" pubspec.lock
            chmod u+w .dart_tool/package_config.json
            ${flutterAndroid}/bin/flutter pub get --offline
          '';
          buildPhase = ''
            runHook preBuild
            export MITM_CACHE_HOST MITM_CACHE_PORT MITM_CACHE_ADDRESS MITM_CACHE_CA
            ${flutterAndroid}/bin/flutter build apk --release
            runHook postBuild
          '';
          dontDartInstall = true;
          installPhase = ''
            runHook preInstall
            install -Dm644 build/app/outputs/flutter-apk/app-release.apk \
              $out/brongnal.apk
            mkdir -p "$debug"
            mkdir -p $out/nix-support
            echo "file binary-dist $out/brongnal.apk" \
              > $out/nix-support/hydra-build-products
            runHook postInstall
          '';
        };
        dockerImage = pkgs.dockerTools.streamLayeredImage {
          name = "brongnal";
          tag = "latest";
          contents = [myServer];
          config = {
            Cmd = [
              "${myServer}/bin/server"
            ];
            Env = ["RUST_LOG=info"];
          };
        };
      in {
        apps = {
          codegen =
            mkApp "brongnal-codegen" (
              nativeDevelopmentTools
              ++ [
                flutterDevelopment
                frbCodegen
              ]
            ) ''
              flutter_rust_bridge_codegen generate \
                --rust-input crate::bridge \
                --rust-root native/hub \
                --dart-output lib/src/rust
            '';
          format =
            mkApp "brongnal-format" (
              nativeDevelopmentTools ++ [flutterDevelopment]
            ) ''
              dart analyze --fatal-infos
              dart format .
              cargo fmt
              cargo clippy --fix --allow-dirty
            '';
          test =
            mkApp "brongnal-test" (
              nativeDevelopmentTools
              ++ [
                flutterDevelopment
                rustup
              ]
            ) ''
              cargo test --workspace --verbose
              # Desktop Dart FFI dlopen's libhub.so from cargo's target dir.
              LD_LIBRARY_PATH="$PWD/target/debug" \
                flutter test -d linux integration_test/app_test.dart
            '';
        };

        devShells.default = pkgs.mkShell {
          packages =
            nativeDevelopmentTools
            ++ [
              androidSdk
              flutterDevelopment
              frbCodegen
              pkgs.jdk17
              pkgs.yubico-piv-tool
              rustup
            ];
          ANDROID_HOME = androidHome;
          ANDROID_SDK_ROOT = androidHome;
          FLUTTER_ROOT = "${flutterDevelopment}";
          JAVA_HOME = pkgs.jdk17.home;
          # android/gradlew execs $NIX_GRADLE when set, so dev-shell builds use
          # the pinned Gradle instead of downloading a distribution to ~/.gradle.
          NIX_GRADLE = "${gradle}/bin/gradle";
          PKG_CONFIG_PATH = "${lib.getDev pkgs.sqlite}/lib/pkgconfig";
        };

        packages = {
          inherit apk myServer dockerImage;
          default = myServer;
        };
      }
    );
}
