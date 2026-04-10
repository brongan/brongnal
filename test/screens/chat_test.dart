import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:brongnal_app/screens/chat.dart';
import 'package:brongnal_app/models/chat_history.dart';
import 'package:brongnal_app/common/theme.dart';
import 'package:provider/provider.dart';
import 'package:brongnal_app/mocks/mock_core.dart';
import 'package:brongnal_app/src/rust/bridge.dart' show MessageModel, MessageState;

void main() {
  testWidgets('ChatScreen shows messages and handles input', (WidgetTester tester) async {
    final mockCore = MockCore(registeredUser: 'me');
    final chatHistory = ChatHistory(
      username: 'me',
      core: mockCore,
      onMessageReceived: (msg) async {},
    );

    // Pre-seed some messages
    chatHistory.addMessage(MessageModel(
      sender: 'alice',
      receiver: 'me',
      dbRecvTime: DateTime.now().millisecondsSinceEpoch ~/ 1000,
      state: MessageState.delivered,
      text: 'hi me',
    ));
    chatHistory.addMessage(MessageModel(
      sender: 'me',
      receiver: 'alice',
      dbRecvTime: DateTime.now().millisecondsSinceEpoch ~/ 1000,
      state: MessageState.sent,
      text: 'hello alice',
    ));

    await tester.pumpWidget(
      ChangeNotifierProvider.value(
        value: chatHistory,
        child: MaterialApp(
          theme: bronganlDarkTheme,
          home: ChatScreen(self: 'me', peer: 'alice'),
        ),
      ),
    );
    await tester.pumpAndSettle();

    // Verify messages render
    expect(find.byType(MessageWidget), findsNWidgets(2));

    // Find input and send button
    expect(find.byType(TextField), findsOneWidget);
    expect(find.byType(IconButton), findsWidgets);

    // Enter text and send via keyboard action
    await tester.enterText(find.byType(TextField), 'new message');
    await tester.testTextInput.receiveAction(TextInputAction.send);
    await tester.pumpAndSettle();

    // Verify message appears
    expect(find.byType(MessageWidget), findsNWidgets(3));
    
    // Check it went via bridge
    final all = await mockCore.getAllMessages();
    expect(all.where((m) => m.text == 'new message'), isNotEmpty);

    chatHistory.dispose();
  });
}
