import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:brongnal_app/src/rust/bridge.dart' as bridge;
import 'package:brongnal_app/src/rust/frb_generated.dart';
import 'package:brongnal_app/main.dart' as app;
import 'package:brongnal_app/common/config.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  group('End-to-end test', () {
    setUpAll(() async {
      await RustLib.init();
      await bridge.startMockServer(port: 50051);
      AppConfig.setBackendOverride("http://localhost:50051");
    });

    testWidgets('verify registration and message sending', (tester) async {
      // Clear shared preferences to ensure we are on the registration screen
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      final tempDir = Directory.systemTemp.createTempSync('brongnal_test_');
      debugPrint('Using temporary database directory: ${tempDir.path}');

      await app.runBrongnalApp(dbDirOverride: tempDir.path);
      await tester.pumpAndSettle();

      // Verify we are on the registration screen
      expect(find.textContaining('Connect with Brongnal'), findsOneWidget);
      expect(find.byType(TextField), findsOneWidget);

      // Enter username
      final String testUsername =
          'TestUser_${DateTime.now().millisecondsSinceEpoch}';
      await tester.enterText(find.byType(TextField), testUsername);

      // Tap Register button
      await tester.tap(find.text('Register'));
      await tester.pumpAndSettle();

      // Wait for registration to complete (rust logic)
      int retry = 0;
      while (find.text('Brongnal').evaluate().isEmpty && retry < 20) {
        await tester.pump(const Duration(seconds: 1));
        retry++;
      }

      expect(find.text('Brongnal'), findsOneWidget);

      // Go to compose message
      await tester.tap(find.byIcon(Icons.create_outlined));
      await tester.pumpAndSettle();

      // Find user (self-messaging)
      expect(find.text('New message'), findsOneWidget);
      await tester.enterText(find.byType(TextField), testUsername);
      await tester.testTextInput.receiveAction(TextInputAction.send);
      await tester.pumpAndSettle();

      // Now on ChatScreen
      expect(find.text(testUsername), findsWidgets);

      // Send a message
      const String testMsg = 'Hello from Integration Test';
      await tester.enterText(find.byType(TextField), testMsg);
      await tester.testTextInput.receiveAction(TextInputAction.send);
      await tester.pumpAndSettle();

      // Wait for message to appear in list (it might take a moment to round-trip through Rust and back)
      retry = 0;
      final messageFinder = find.byWidgetPredicate((widget) =>
          widget is RichText && widget.text.toPlainText().contains(testMsg));
      while (messageFinder.evaluate().isEmpty && retry < 10) {
        await tester.pump(const Duration(milliseconds: 500));
        retry++;
      }

      // Verify message appears in list (at least one, possibly two due to self-messaging)
      expect(messageFinder, findsAtLeastNWidgets(1));
    });
  });
}
