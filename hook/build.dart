import 'dart:io';

import 'package:code_assets/code_assets.dart';
import 'package:flutter_rust_bridge_hooks/flutter_rust_bridge_hooks.dart';
import 'package:path/path.dart' as p;

/// Makes the Android compiler honor the minimum API supplied by Flutter.
Map<String, String> _androidCompilerEnvironment(BuildInput input) {
  if (!input.config.buildCodeAssets) {
    return const {};
  }

  final code = input.config.code;
  final cCompiler = code.cCompiler;
  if (code.targetOS != OS.android || cCompiler == null) {
    return const {};
  }

  final (target, ndkTarget) = switch (code.targetArchitecture) {
    Architecture.arm => ('armv7-linux-androideabi', 'armv7a-linux-androideabi'),
    Architecture.arm64 => ('aarch64-linux-android', 'aarch64-linux-android'),
    Architecture.x64 => ('x86_64-linux-android', 'x86_64-linux-android'),
    final architecture => throw UnsupportedError(
      'Unsupported Android architecture: $architecture',
    ),
  };
  final executableSuffix = Platform.isWindows ? '.cmd' : '';
  final compilerDirectory = p.dirname(p.fromUri(cCompiler.compiler));
  final api = code.android.targetNdkApi;
  final clang = p.join(
    compilerDirectory,
    '$ndkTarget$api-clang$executableSuffix',
  );
  final clangPp = p.join(
    compilerDirectory,
    '$ndkTarget$api-clang++$executableSuffix',
  );
  final environmentTarget = target.replaceAll('-', '_');

  return {
    'CC_$environmentTarget': clang,
    'CXX_$environmentTarget': clangPp,
    'CARGO_TARGET_${environmentTarget.toUpperCase()}_LINKER': clang,
  };
}

void main(List<String> args) async {
  await build(args, (input, output) async {
    final builder = FlutterRustBridgeNativeAssetsBuilder(
      cratePath: 'native/hub',
      extraCargoEnvironmentVariables: _androidCompilerEnvironment(input),
    );
    await builder.run(input: input, output: output);
  });
}
