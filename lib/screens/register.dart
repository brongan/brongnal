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
      backgroundColor: backgroundColor,
      body: Center(
        child: Column(
          children: [
            const Text('Register'),
            Padding(
              padding: const EdgeInsets.all(20.0),
              child: TextField(
                controller: usernameInput,
                decoration: const InputDecoration(hintText: "name"),
              ),
            ),
            ElevatedButton(
              child: const Text('OK', style: conversationNameStyle),
              onPressed: () {
                final name = usernameInput.text;
                try {
                  final _ = stub.registerPreKeyBundle(
                      RegisterPreKeyBundleRequest(identity: name),
                      options:
                          CallOptions(timeout: const Duration(seconds: 5)));
                  onRegisterSuccess(name);
                } catch (e) {
                  final messenger = ScaffoldMessenger.of(context);
                  messenger.removeCurrentSnackBar();
                  messenger.showSnackBar(SnackBar(content: Text(name)));
                } // Register
              },
            ),
          ],
        ),
      ),
    );
  }
}
