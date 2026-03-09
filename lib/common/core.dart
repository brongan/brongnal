import 'dart:async';
import 'package:brongnal_app/src/rust/bridge.dart';
import 'package:brongnal_app/src/rust/frb_generated.dart';

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

  Future<MessageModel> sendMessage({
    required String recipient,
    required String text,
  });

  Future<List<MessageModel>> getAllMessages();

  Stream<MessageModel> subscribeMessages();
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
    return RustLib.instance.api.crateBridgeStartHub(
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
    return RustLib.instance.api.crateBridgeRegisterUser(
      username: username,
      fcmToken: fcmToken,
      backendAddress: backendAddress,
      databaseDirectory: databaseDirectory,
    );
  }

  @override
  Future<MessageModel> sendMessage({
    required String recipient,
    required String text,
  }) {
    return RustLib.instance.api.crateBridgeSendMessage(recipient: recipient, text: text);
  }

  @override
  Future<List<MessageModel>> getAllMessages() {
    return RustLib.instance.api.crateBridgeGetAllMessages();
  }

  @override
  Stream<MessageModel> subscribeMessages() {
    return RustLib.instance.api.crateBridgeSubscribeMessages();
  }
}
