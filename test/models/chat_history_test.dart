import 'package:flutter_test/flutter_test.dart';
import 'package:brongnal_app/models/chat_history.dart';
import 'package:brongnal_app/src/rust/bridge.dart' show MessageModel, MessageState;
import 'package:brongnal_app/mocks/mock_core.dart';

void main() {
  group('ChatHistory', () {
    late ChatHistory chatHistory;
    late MockCore mockCore;
    bool notificationReceived = false;

    setUp(() async {
      mockCore = MockCore(registeredUser: 'me');
      notificationReceived = false;

      // Seed mock history
      mockCore.receiveMockMessage(MessageModel(
        sender: 'alice',
        receiver: 'me',
        dbRecvTime: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        state: MessageState.delivered,
        text: 'hello from alice',
      ));

      chatHistory = ChatHistory(
        username: 'me',
        core: mockCore,
        onMessageReceived: (msg) async {
          notificationReceived = true;
        },
      );
      
      // Allow async init to finish
      await pumpEventQueue();
    });

    tearDown(() {
      mockCore.close();
      chatHistory.dispose();
    });

    test('Loads initial history and groups by peer', () {
      expect(chatHistory.items.keys, contains('alice'));
      expect(chatHistory.items['alice']!.length, 1);
      expect(chatHistory.items['alice']!.first.text, 'hello from alice');
    });

    test('sendMessage adds to local history and calls backend', () async {
      await chatHistory.sendMessage('bob', 'hi bob');

      expect(chatHistory.items.keys, contains('bob'));
      expect(chatHistory.items['bob']!.length, 1);
      expect(chatHistory.items['bob']!.first.text, 'hi bob');
      expect(chatHistory.items['bob']!.first.state, MessageState.sent);
    });

    test('Receiving message via stream adds to history and calls callback', () async {
      // Simulate live message via stream
      mockCore.receiveMockMessage(MessageModel(
        sender: 'charlie',
        receiver: 'me',
        dbRecvTime: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        state: MessageState.delivered,
        text: 'live message',
      ));

      await pumpEventQueue();

      expect(chatHistory.items.keys, contains('charlie'));
      expect(chatHistory.items['charlie']!.length, 1);
      expect(notificationReceived, isTrue);
    });
  });
}
