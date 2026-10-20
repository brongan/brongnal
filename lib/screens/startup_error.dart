import 'dart:io' show Platform;

import 'package:brongnal_app/src/rust/bridge.dart';
import 'package:flutter/material.dart';

/// Tells the user how to clear Brongnal's local data on this platform.
String resetInstructions(String databaseDirectory) {
  if (Platform.isAndroid) {
    return 'To reset Brongnal, clear its storage in Android Settings.';
  }
  if (Platform.isIOS) {
    return 'To reset Brongnal, delete the app and install it again.';
  }
  return 'To reset Brongnal, delete the folder $databaseDirectory.';
}

class StartupError extends StatelessWidget {
  const StartupError({
    super.key,
    required this.error,
    required this.username,
    required this.resetInstructions,
    required this.retrying,
    required this.onRetry,
    required this.onRegisterAgain,
  });
  final Object error;
  final String username;
  final String resetInstructions;
  final bool retrying;
  final VoidCallback onRetry;
  final VoidCallback onRegisterAgain;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final (message, actionLabel, action) = switch (error) {
      BridgeError_ServerUnreachable() => (
        "Can't reach the Brongnal server. Check your connection.",
        'Retry',
        onRetry,
      ),
      BridgeError_ServerError() => (
        'The Brongnal server had an error. Try again later.',
        'Retry',
        onRetry,
      ),
      BridgeError_RegistrationRejected() => (
        'The server rejected "$username".',
        'Register again',
        onRegisterAgain,
      ),
      _ => (
        "Can't open your local data. $resetInstructions This deletes "
            'your messages and your identity key.',
        null,
        null,
      ),
    };
    return Scaffold(
      backgroundColor: theme.colorScheme.surface,
      body: SafeArea(
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Padding(
                padding: const EdgeInsets.all(20.0),
                child: SelectableText(
                  message,
                  textAlign: TextAlign.center,
                  style: theme.textTheme.headlineSmall,
                ),
              ),
              if (actionLabel != null)
                ElevatedButton(
                  onPressed: retrying ? null : action,
                  child: Text(
                    actionLabel,
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 36,
                      fontFamily: 'Roboto',
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
