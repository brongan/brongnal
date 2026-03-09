import 'dart:async';
import 'package:brongnal_app/common/core.dart';
import 'package:brongnal_app/src/rust/bridge.dart' show MessageModel, MessageState;

class MockCore implements BrongnalCore {
  final List<MessageModel> _messages = [];
  final StreamController<MessageModel> _controller = StreamController.broadcast();
  bool isHubStarted = false;
  String? registeredUser;

  MockCore({this.registeredUser});

  @override
  Future<void> startHub({
    required String databaseDirectory,
    String? username,
    String? fcmToken,
    String? backendAddress,
  }) async {
    isHubStarted = true;
    registeredUser = username;
  }

  @override
  Future<void> registerUser({
    required String username,
    String? fcmToken,
    required String backendAddress,
    required String databaseDirectory,
  }) async {
    registeredUser = username;
  }

  @override
  Future<MessageModel> sendMessage({
    required String recipient,
    required String text,
  }) async {
    final sender = registeredUser ?? 'unknown';
    final message = MessageModel(
      sender: sender,
      receiver: recipient,
      dbRecvTime: DateTime.now().millisecondsSinceEpoch ~/ 1000,
      state: MessageState.sent,
      text: text,
    );
    _messages.add(message);
    return message;
  }

  @override
  Future<List<MessageModel>> getAllMessages() async {
    return List.from(_messages);
  }

  @override
  Stream<MessageModel> subscribeMessages() {
    return _controller.stream;
  }

  // --- Methods for testing ---

  void receiveMockMessage(MessageModel message) {
    _messages.add(message);
    _controller.add(message);
  }

  void close() {
    _controller.close();
  }
}
