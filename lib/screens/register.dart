import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/generated/service.pbgrpc.dart';
import 'package:brongnal_app/messages/brongnal.pb.dart';
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
                BrongnalAction(
                        register: RegisterAction(name: usernameInput.text))
                    .sendSignalToRust();
              },
            ),
          ],
        ),
      ),
    );
  }
}
