import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:brongnal_app/screens/register.dart';
import 'package:brongnal_app/common/config.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:brongnal_app/mocks/mock_core.dart';

void main() {
  testWidgets('Register screen shows UI elements and triggers onRegister', (WidgetTester tester) async {
    AppConfig.setDatabaseOverride('/tmp/test_db');
    SharedPreferences.setMockInitialValues({});
    final mockCore = MockCore();
    String? registeredName;

    await tester.pumpWidget(MaterialApp(
      home: Register(
        core: mockCore,
        onRegister: (name) {
          registeredName = name;
        },
      ),
    ));

    // Verify UI elements
    expect(find.text('Connect with Brongnal'), findsOneWidget);
    expect(find.byType(TextField), findsOneWidget);
    expect(find.text('Register'), findsOneWidget);

    // Empty submit should not trigger
    await tester.tap(find.text('Register'));
    await tester.pumpAndSettle();
    expect(registeredName, isNull);
    expect(mockCore.registeredUser, isNull);

    // Enter username and submit
    await tester.enterText(find.byType(TextField), 'testuser');
    await tester.tap(find.text('Register'));
    await tester.pumpAndSettle();

    // Verify registration works
    expect(registeredName, 'testuser');
    expect(mockCore.registeredUser, 'testuser');
  });
}
