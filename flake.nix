{
  description = "Brongan's attempt at signal";
  inputs = {
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
  outputs = {  nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
          overlays = [ (import rust-overlay) ];
        };
        sqliteStatic = pkgs.pkgsStatic.sqlite;
        inherit (pkgs) lib;
        toolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" ];
          targets = [
            "aarch64-linux-android"
            "armv7-linux-androideabi"
            "x86_64-linux-android"
            "x86_64-unknown-linux-musl"
          ];
        };
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
                  printf '%s\\n' \
                    aarch64-linux-android \
                    armv7-linux-androideabi \
                    x86_64-linux-android \
                    x86_64-unknown-linux-gnu \
                    x86_64-unknown-linux-musl
                  ;;
                add) exit 0 ;;
              esac
              ;;
            component) exit 0 ;;
          esac
        '';
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        src = lib.cleanSource ./.;
        androidNdkVersion = "28.2.13676358";
        androidComposition = pkgs.androidenv.composeAndroidPackages {
          # rust_builder requires API 34, flutter_local_notifications requires
          # API 35, and Flutter 3.38 builds the app against API 36.
          platformVersions = [ "34" "35" "36" ];
          buildToolsVersions = [ "35.0.0" ];
          cmakeVersions = [ "3.22.1" ];
          includeNDK = true;
          ndkVersions = [ androidNdkVersion ];
        };
        androidSdk = androidComposition.androidsdk;
        androidHome = "${androidSdk}/libexec/android-sdk";
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
        gradleUnwrapped = pkgs.gradle-packages.mkGradle {
          version = "8.13";
          hash = "sha256-IPGxF2I3JUpvwgTYQ0GW+hGkz7OHVnUZxhVW6HEK7Xg=";
          defaultJava = pkgs.jdk17;
        };
        gradle = pkgs.callPackage pkgs.gradle-packages.wrapGradle {
          gradle-unwrapped = gradleUnwrapped;
        };
        gradleInitScript =
          "${nixpkgs.outPath}/pkgs/development/tools/build-managers/gradle/init-build.gradle";
        gradlew = pkgs.writeShellScript "gradlew" ''
          set -eu
          extra=(
            --no-daemon
            --console plain
            --init-script ${gradleInitScript}
          )
          if [[ -n "''${MITM_CACHE_ADDRESS:-}" ]]; then
            truststore="''${TMPDIR:-/tmp}/gradle-mitm-keystore"
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
        mkApp = name: runtimeInputs: text:
          let
            package = pkgs.writeShellApplication {
              inherit name runtimeInputs;
              text = developmentEnvironment + text;
            };
          in
          {
            type = "app";
            program = "${package}/bin/${name}";
          };
        nativeDevelopmentTools = [
          pkgs.pkg-config
          pkgs.protobuf
          toolchain
        ];
        args = {
          inherit src;
          version = "0.1.0";
          strictDeps = true;
          cargoExtraArgs = "--package=server";
          nativeBuildInputs = with pkgs; [ pkg-config protobuf ];
		  CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
          buildInputs = [ sqliteStatic ];
		  pname = "server";
        };
        nativeArtifacts = craneLib.buildDepsOnly args;
        myServer = craneLib.buildPackage (args // {
          cargoArtifacts = nativeArtifacts;
          postFixup = ''
            ${pkgs.removeReferencesTo}/bin/remove-references-to \
              -t ${toolchain} "$out/bin/server"
          '';
        });
        apk = pkgs.flutter.buildFlutterApplication {
          pname = "brongnal-apk";
          version = "1.0.0";
          inherit src;
          targetFlutterPlatform = "universal";
          pubspecLock = lib.importJSON ./nix/pubspec.lock.json;

          nativeBuildInputs = [
            androidSdk
            gradle
            pkgs.cmake
            pkgs.jdk17
            pkgs.ninja
            pkgs.pkg-config
            pkgs.protobuf
            rustup
            toolchain
          ];

          mitmCache = gradle.fetchDeps {
            pkg = apk;
            data = ./nix/gradle-deps.json;
          };
          gradleUpdateScript = ''
            ${flutterAndroid}/bin/flutter build apk --release --no-pub
          '';

          ANDROID_HOME = androidHome;
          ANDROID_SDK_ROOT = androidHome;
          JAVA_HOME = pkgs.jdk17.home;
          dontUseCmakeConfigure = true;

          postPatch = ''
            cp ${gradlew} android/gradlew
            chmod +x android/gradlew
            echo 'android.aapt2FromMavenOverride=${androidHome}/build-tools/35.0.0/aapt2' \
              >> android/gradle.properties
            sed -i '/signingConfig signingConfigs\.release/d' \
              android/app/build.gradle
            rm -f android/key.properties android/local.properties
            cat > android/local.properties <<EOF
            flutter.sdk=${flutterAndroid}
            sdk.dir=${androidHome}
            EOF
          '';

          dontDartBuild = true;
          buildPhase = ''
            runHook preBuild
            export MITM_CACHE_HOST MITM_CACHE_PORT MITM_CACHE_ADDRESS MITM_CACHE_CA
            ${flutterAndroid}/bin/flutter build apk --release --no-pub
            runHook postBuild
          '';
          dontDartInstall = true;
          installPhase = ''
            runHook preInstall
            install -Dm644 build/app/outputs/flutter-apk/app-release.apk \
              $out/brongnal.apk
            mkdir -p $out/nix-support
            echo "file binary-dist $out/brongnal.apk" \
              > $out/nix-support/hydra-build-products
            runHook postInstall
          '';
        };
        dockerImage = pkgs.dockerTools.streamLayeredImage {
          name = "brongnal";
          tag = "latest";
          contents = [ myServer ];
          config = {
            Cmd = [
              "${myServer}/bin/server"
            ];
            Env = ["RUST_LOG=info"];
          };
        };
      in
      {
        apps = {
          codegen = mkApp "brongnal-codegen" (
            nativeDevelopmentTools
            ++ [
              flutterDevelopment
              pkgs.flutter_rust_bridge_codegen
            ]
          ) ''
            flutter_rust_bridge_codegen generate \
              --rust-input crate::bridge \
              --rust-root native/hub \
              --dart-output lib/src/rust
          '';
          format = mkApp "brongnal-format" (
            nativeDevelopmentTools ++ [ flutterDevelopment ]
          ) ''
            dart analyze --fatal-infos
            dart format .
            cargo fmt
            cargo clippy --fix --allow-dirty
          '';
          test = mkApp "brongnal-test" (
            nativeDevelopmentTools
            ++ [
              flutterDevelopment
              rustup
            ]
          ) ''
            cargo test --workspace --verbose
            LD_LIBRARY_PATH="$PWD/target/debug" \
              flutter test -d linux integration_test/app_test.dart
          '';
        };

        devShells.default = pkgs.mkShell {
          packages = nativeDevelopmentTools ++ [
            androidSdk
            flutterDevelopment
            pkgs.flutter_rust_bridge_codegen
            pkgs.jdk17
            rustup
          ];
          ANDROID_HOME = androidHome;
          ANDROID_SDK_ROOT = androidHome;
          FLUTTER_ROOT = "${flutterDevelopment}";
          JAVA_HOME = pkgs.jdk17.home;
          PKG_CONFIG_PATH = "${lib.getDev pkgs.sqlite}/lib/pkgconfig";
        };

        packages = {
          inherit apk myServer dockerImage;
          default = myServer;
        };
      }
    );
}
