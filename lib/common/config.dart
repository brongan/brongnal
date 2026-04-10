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

  static String get defaultMailboxAddr =>
      _backendOverride ??
      const String.fromEnvironment(
        'MAILBOX_ADDR',
        defaultValue: 'https://signal.brongan.com:443',
      );

  static String get defaultIdentityAddr =>
      const String.fromEnvironment(
        'IDENTITY_ADDR',
        defaultValue: 'https://gossamer.brongan.com:443',
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
