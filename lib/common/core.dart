import 'dart:async';
import 'package:brongnal_app/src/rust/bridge.dart' as bridge;

abstract class BrongnalCore {
  Future<void> startHub({
    required String databaseDirectory,
    String? username,
    String? backendAddress,
  });

  Future<void> registerDevice({String? fcmToken});

  Future<void> registerUser({
    required String username,
    String? fcmToken,
    required String backendAddress,
    required String databaseDirectory,
  });

  Future<bridge.MessageModel> sendMessage({
    required String recipient,
    required String text,
  });

  Future<List<bridge.MessageModel>> getAllMessages();

  Stream<bridge.MessageModel> subscribeMessages();
}

class RustBrongnalCore implements BrongnalCore {
  const RustBrongnalCore();

  @override
  Future<void> startHub({
    required String databaseDirectory,
    String? username,
    String? backendAddress,
  }) {
    return bridge.startHub(
      databaseDirectory: databaseDirectory,
      username: username,
      backendAddress: backendAddress,
    );
  }

  @override
  Future<void> registerDevice({String? fcmToken}) {
    return bridge.registerDevice(fcmToken: fcmToken);
  }

  @override
  Future<void> registerUser({
    required String username,
    String? fcmToken,
    required String backendAddress,
    required String databaseDirectory,
  }) {
    return bridge.registerUser(
      username: username,
      fcmToken: fcmToken,
      backendAddress: backendAddress,
      databaseDirectory: databaseDirectory,
    );
  }

  @override
  Future<bridge.MessageModel> sendMessage({
    required String recipient,
    required String text,
  }) {
    return bridge.sendMessage(recipient: recipient, text: text);
  }

  @override
  Future<List<bridge.MessageModel>> getAllMessages() {
    return bridge.getAllMessages();
  }

  @override
  Stream<bridge.MessageModel> subscribeMessages() {
    return bridge.subscribeMessages();
  }
}
