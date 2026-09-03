import 'dart:io' show File, Platform;

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated_io.dart'
    show ExternalLibrary;
import 'package:path/path.dart' as p;

import 'frb_generated.dart';

ExternalLibrary? _bundledRustLibrary() {
  if (!Platform.isLinux) {
    return null;
  }

  final library = File(
    p.join(File(Platform.resolvedExecutable).parent.path, 'lib', 'libhub.so'),
  );
  return library.existsSync() ? ExternalLibrary.open(library.path) : null;
}

Future<void> initializeRustLib() async {
  if (RustLib.instance.initialized) {
    return;
  }

  await RustLib.init(externalLibrary: _bundledRustLibrary());
}
