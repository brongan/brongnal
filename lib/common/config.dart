import 'dart:io';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:xdg_directories/xdg_directories.dart';

class AppConfig {
  static String? _dbOverride;
  static String? _backendOverride;

  static void setDatabaseOverride(String? path) {
    _dbOverride = path;
  }

  static void setBackendOverride(String? addr) {
    _backendOverride = addr;
  }

  static String get defaultBackendAddr =>
      _backendOverride ??
      const String.fromEnvironment(
        'BACKEND_ADDR',
        defaultValue: 'https://signal.brongan.com:443',
      );

  static Future<String> getDatabaseDirectory({String? override}) async {
    final effectiveOverride = override ?? _dbOverride;
    if (effectiveOverride != null) return effectiveOverride;

    Directory directory;
    try {
      // Logic from Register.dart / XDG fallback
      directory = Directory(p.join(dataHome.path, "brongnal"));
    } on StateError catch (_) {
      directory = await getApplicationSupportDirectory();
    }

    if (!await directory.exists()) {
      await directory.create(recursive: true);
    }

    return directory.path;
  }
}
