import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:brongnal_app/screens/compose.dart';
import 'package:brongnal_app/screens/chat.dart';
import 'package:brongnal_app/models/chat_history.dart';
import 'package:brongnal_app/common/theme.dart';
import 'package:provider/provider.dart';
import 'package:brongnal_app/mocks/mock_core.dart';

void main() {
  testWidgets('ComposeMessage allows starting new chat', (WidgetTester tester) async {
    final mockCore = MockCore();
    final chatHistory = ChatHistory(
      username: 'me',
      core: mockCore,
      onMessageReceived: (msg) async {},
    );

    bool popped = false;
    await tester.pumpWidget(
      ChangeNotifierProvider.value(
        value: chatHistory,
        child: MaterialApp(
          theme: bronganlDarkTheme,
          home: Navigator(
            onPopPage: (route, result) {
              popped = true;
              return route.didPop(result);
            },
            pages: [
              MaterialPage(child: ComposeMessage(self: 'me')),
            ],
          ),
        ),
      ),
    );

    expect(find.text('New message'), findsOneWidget);
    expect(find.byType(TextField), findsOneWidget);

    // Enter username and submit to push the ChatScreen
    await tester.enterText(find.byType(TextField), 'newfriend');
    await tester.testTextInput.receiveAction(TextInputAction.send);
    await tester.pumpAndSettle();

    // Verification - ComposeMessage should push ChatScreen.
    // The ChatScreen should render the conversation name 'newfriend' in its app bar.
    expect(find.byType(ChatScreen), findsOneWidget);
    // There shouldn't be any "New message" text anymore because that was the Compose app bar
    expect(find.text('New message'), findsNothing);

    chatHistory.dispose();
  });

  testWidgets('ComposeMessage updates conversation list after a message is sent', (WidgetTester tester) async {
    final mockCore = MockCore(registeredUser: 'me');
    final chatHistory = ChatHistory(
      username: 'me',
      core: mockCore,
      onMessageReceived: (msg) async {},
    );

    await tester.pumpWidget(
      ChangeNotifierProvider.value(
        value: chatHistory,
        child: MaterialApp(
          theme: bronganlDarkTheme,
          home: Scaffold(body: ComposeMessage(self: 'me')),
        ),
      ),
    );

    // Initial state: no conversations visible under the search bar
    expect(find.text('newfriend'), findsNothing);

    // Simulate sending a message to 'newfriend' directly via the model
    // (This mocks what happens when ChatScreen pops back or sends a message in the background)
    await chatHistory.sendMessage('newfriend', 'first message');
    await tester.pumpAndSettle();

    // Verification - The ConversationsScreen embedded in the Compose body should now show 'newfriend'
    expect(find.text('newfriend'), findsOneWidget);
    expect(find.text('first message'), findsOneWidget);

    chatHistory.dispose();
  });
}
