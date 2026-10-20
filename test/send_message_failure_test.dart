import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/mocks/mock_core.dart';
import 'package:brongnal_app/models/chat_history.dart';
import 'package:brongnal_app/screens/chat.dart';
import 'package:brongnal_app/src/rust/bridge.dart' show MessageModel;
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';

class FailingSendCore extends MockCore {
  FailingSendCore() : super(registeredUser: 'alice');

  @override
  Future<MessageModel> sendMessage({
    required String recipient,
    required String text,
  }) async {
    throw Exception('username not found');
  }
}

void main() {
  testWidgets('failed send keeps the text and shows the error', (tester) async {
    final core = FailingSendCore();
    await tester.pumpWidget(
      ChangeNotifierProvider(
        create: (_) => ChatHistory(
          username: 'alice',
          onMessageReceived: (_) async {},
          core: core,
        ),
        child: MaterialApp(
          darkTheme: bronganlDarkTheme,
          themeMode: ThemeMode.dark,
          home: const ChatScreen(self: 'alice', peer: 'bob'),
        ),
      ),
    );

    await tester.enterText(find.byType(TextField), 'hello');
    await tester.testTextInput.receiveAction(TextInputAction.send);
    await tester.pump();

    final input = tester.widget<TextField>(find.byType(TextField));
    expect(input.controller!.text, 'hello');
    expect(find.textContaining('Send failed'), findsOneWidget);
    expect(find.byType(MessageWidget), findsNothing);
  });
}
