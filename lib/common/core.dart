import 'dart:async';
import 'package:brongnal_app/src/rust/bridge.dart' as rust;

abstract class BrongnalCore {
  Future<void> startHub({
    required String databaseDirectory,
    String? username,
    String? fcmToken,
    String? backendAddress,
  });

  Future<void> registerUser({
    required String username,
    String? fcmToken,
    required String backendAddress,
    required String databaseDirectory,
  });

  Future<rust.MessageModel> sendMessage({
    required String recipient,
    required String text,
  });

  Future<List<rust.MessageModel>> getAllMessages();

  Stream<rust.MessageModel> subscribeMessages();
}

class RustBrongnalCore implements BrongnalCore {
  const RustBrongnalCore();

  @override
  Future<void> startHub({
    required String databaseDirectory,
    String? username,
    String? fcmToken,
    String? backendAddress,
  }) {
    return rust.startHub(
      databaseDirectory: databaseDirectory,
      username: username,
      fcmToken: fcmToken,
      backendAddress: backendAddress,
    );
  }

  @override
  Future<void> registerUser({
    required String username,
    String? fcmToken,
    required String backendAddress,
    required String databaseDirectory,
  }) {
    return rust.registerUser(
      username: username,
      fcmToken: fcmToken,
      backendAddress: backendAddress,
      databaseDirectory: databaseDirectory,
    );
  }

  @override
  Future<rust.MessageModel> sendMessage({
    required String recipient,
    required String text,
  }) {
    return rust.sendMessage(recipient: recipient, text: text);
  }

  @override
  Future<List<rust.MessageModel>> getAllMessages() {
    return rust.getAllMessages();
  }

  @override
  Stream<rust.MessageModel> subscribeMessages() {
    return rust.subscribeMessages();
  }
}
