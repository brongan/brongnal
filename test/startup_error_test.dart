import 'package:brongnal_app/main.dart';
import 'package:brongnal_app/mocks/mock_core.dart';
import 'package:brongnal_app/screens/home.dart';
import 'package:brongnal_app/screens/register.dart';
import 'package:brongnal_app/src/rust/bridge.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

class FailingHistoryCore extends MockCore {
  FailingHistoryCore() : super(registeredUser: 'alice');

  @override
  Future<List<MessageModel>> getAllMessages() async {
    throw const BridgeError.initializationFailed('corrupt database');
  }
}

Future<void> pumpApp(
  WidgetTester tester,
  List<BridgeError> errors, {
  MockCore? core,
}) async {
  SharedPreferences.setMockInitialValues({'username': 'alice'});
  await tester.pumpWidget(
    BrongnalApp(
      username: 'alice',
      core: core ?? MockCore(registeredUser: 'alice'),
      onMessageReceived: (_) async {},
      // The real startHub crosses FFI, so it never completes synchronously.
      startHub: () => Future.delayed(const Duration(milliseconds: 1), () {
        if (errors.isNotEmpty) throw errors.removeAt(0);
      }),
    ),
  );
  await tester.pumpAndSettle();
}

Future<void> tapAndSettle(WidgetTester tester, String label) async {
  await tester.tap(find.text(label));
  await tester.pump();
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('unreachable server offers retry until it succeeds', (
    tester,
  ) async {
    await pumpApp(tester, [
      const BridgeError.serverUnreachable(),
      const BridgeError.serverUnreachable(),
    ]);
    expect(find.textContaining("Can't reach"), findsOneWidget);

    await tapAndSettle(tester, 'Retry');
    expect(find.textContaining("Can't reach"), findsOneWidget);

    await tapAndSettle(tester, 'Retry');
    expect(find.byType(Home), findsOneWidget);
  });

  testWidgets('server error offers retry', (tester) async {
    await pumpApp(tester, [const BridgeError.serverError()]);
    expect(find.textContaining('server had an error'), findsOneWidget);

    await tapAndSettle(tester, 'Retry');
    expect(find.byType(Home), findsOneWidget);
  });

  testWidgets('rejected registration leads back to register', (tester) async {
    await pumpApp(tester, [const BridgeError.registrationRejected()]);
    expect(find.text('The server rejected "alice".'), findsOneWidget);

    await tapAndSettle(tester, 'Register again');
    expect(find.byType(Register), findsOneWidget);
    final prefs = await SharedPreferences.getInstance();
    expect(prefs.getString('username'), isNull);
  });

  testWidgets('local data failure shows no action', (tester) async {
    await pumpApp(tester, [const BridgeError.initializationFailed('locked')]);
    expect(find.text("Can't open your local data."), findsOneWidget);
    expect(find.byType(ElevatedButton), findsNothing);
  });

  testWidgets('history load failure shows the local data error', (
    tester,
  ) async {
    await pumpApp(tester, [], core: FailingHistoryCore());
    expect(find.text("Can't open your local data."), findsOneWidget);
    expect(find.byType(Home), findsNothing);
  });
}
