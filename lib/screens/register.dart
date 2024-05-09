import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/generated/service.pbgrpc.dart';
import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart';

class Register extends StatelessWidget {
  const Register({
    super.key,
    required this.onRegisterSuccess,
    required this.stub,
  });

  final void Function(String) onRegisterSuccess;
  final BrongnalClient stub;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    TextEditingController usernameInput = TextEditingController();
    return Scaffold(
      backgroundColor: theme.colorScheme.background,
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text('Connect with Brongnal', style: theme.textTheme.displayMedium),
            Padding(
              padding: const EdgeInsets.all(20.0),
              child: TextField(
                controller: usernameInput,
                decoration: const InputDecoration(hintText: "name"),
              ),
            ),
            ElevatedButton(
              child: const Text('Register', style: conversationNameStyle),
              onPressed: () async {
                final name = usernameInput.text;
                try {
                  final _ = await stub.registerPreKeyBundle(
                      RegisterPreKeyBundleRequest(identity: name),
                      options:
                          CallOptions(timeout: const Duration(seconds: 5)));
                  onRegisterSuccess(name);
                } catch (e) {
                  // TODO Log this error.
                  if (!context.mounted) return;

                  final messenger = ScaffoldMessenger.of(context);
                  messenger.removeCurrentSnackBar();
                  messenger.showSnackBar(SnackBar(
                      content: Text(
                          "Failed to register \"$name\" at signal.brongan.com.")));
                } // Register
              },
            ),
          ],
        ),
      ),
    );
  }
}
