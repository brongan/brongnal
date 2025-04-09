import 'package:brongnal_app/generated/service.pbgrpc.dart';
import 'package:brongnal_app/src/bindings/bindings.dart';
import 'package:flutter/material.dart';

class Register extends StatelessWidget {
  const Register({
    super.key,
    required this.stub,
  });

  final BrongnalClient stub;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    TextEditingController usernameInput = TextEditingController();
    return Scaffold(
      backgroundColor: theme.colorScheme.surface,
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Padding(
              padding: const EdgeInsets.all(20.0),
              child: Text('Connect with Brongnal',
                  style: theme.textTheme.displayLarge),
            ),
            Padding(
              padding: const EdgeInsets.all(40.0),
              child: SizedBox(
                width: 550,
                child: TextField(
                  controller: usernameInput,
                  decoration: const InputDecoration(
                    labelText: "Username",
                    border: OutlineInputBorder(),
                  ),
                ),
              ),
            ),
            ElevatedButton(
              child: const Text('Register',
                  style: TextStyle(
                      color: Colors.white, fontSize: 36, fontFamily: 'Roboto')),
              onPressed: () async {
                RegisterUserRequest(username: usernameInput.text)
                    .sendSignalToRust();
              },
            ),
          ],
        ),
      ),
    );
  }
}
