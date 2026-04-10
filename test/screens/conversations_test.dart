import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:brongnal_app/screens/conversations.dart';
import 'package:brongnal_app/models/chat_history.dart';
import 'package:brongnal_app/common/theme.dart';
import 'package:provider/provider.dart';
import 'package:brongnal_app/mocks/mock_core.dart';
import 'package:brongnal_app/src/rust/bridge.dart' show MessageModel, MessageState;

void main() {
  testWidgets('ConversationsScreen shows list of peers', (WidgetTester tester) async {
    final mockCore = MockCore();
    final chatHistory = ChatHistory(
      username: 'me',
      core: mockCore,
      onMessageReceived: (msg) async {},
    );

    chatHistory.addMessage(MessageModel(
      sender: 'alice',
      receiver: 'me',
      dbRecvTime: DateTime.now().millisecondsSinceEpoch ~/ 1000,
      state: MessageState.delivered,
      text: 'latest from alice',
    ));

    chatHistory.addMessage(MessageModel(
      sender: 'bob',
      receiver: 'me',
      dbRecvTime: DateTime.now().millisecondsSinceEpoch ~/ 1000,
      state: MessageState.delivered,
      text: 'hi from bob',
    ));

    await tester.pumpWidget(MaterialApp(
      theme: bronganlDarkTheme,
      home: Scaffold(
        body: ChangeNotifierProvider.value(
          value: chatHistory,
          child: ConversationsScreen(
            self: 'me',
            conversations: chatHistory.items,
          ),
        ),
      ),
    ));

    // Wait for async stream to build
    await tester.pumpAndSettle();

    // Verify peers render
    expect(find.text('alice'), findsOneWidget);
    expect(find.text('latest from alice'), findsOneWidget);
    expect(find.text('bob'), findsOneWidget);
    expect(find.text('hi from bob'), findsOneWidget);

    chatHistory.dispose();
  });
}
