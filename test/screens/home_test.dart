import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:brongnal_app/screens/home.dart';
import 'package:brongnal_app/models/chat_history.dart';
import 'package:brongnal_app/common/theme.dart';
import 'package:provider/provider.dart';
import 'package:brongnal_app/mocks/mock_core.dart';

void main() {
  testWidgets('Home screen navigates to compose on FAB tap', (WidgetTester tester) async {
    final mockCore = MockCore();
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
          home: Home(username: 'me'),
        ),
      ),
    );

    // Verify Title and FAB exist
    expect(find.text('Brongnal'), findsOneWidget);
    expect(find.byType(FloatingActionButton), findsWidgets);

    // Tap FAB
    await tester.tap(find.byIcon(Icons.create_outlined));
    await tester.pumpAndSettle();

    // Verify nav to Compose message
    expect(find.text('New message'), findsOneWidget);

    chatHistory.dispose();
  });
}
