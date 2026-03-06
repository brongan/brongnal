import 'package:brongnal_app/src/rust/bridge.dart' as bridge;
import 'package:brongnal_app/common/config.dart';
import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

class Register extends StatelessWidget {
  const Register({super.key, required this.onRegister});
  final void Function(String username) onRegister;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final TextEditingController usernameInput = TextEditingController();
    return Scaffold(
      backgroundColor: theme.colorScheme.surface,
      body: SafeArea(
        child: Center(
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
                        color: Colors.white,
                        fontSize: 36,
                        fontFamily: 'Roboto')),
                onPressed: () async {
                  final String username = usernameInput.text;
                  if (username.isEmpty) return;

                  final String dbPath = await AppConfig.getDatabaseDirectory();

                  try {
                    await bridge.registerUser(
                        username: username,
                        backendAddress: AppConfig.defaultBackendAddr,
                        databaseDirectory: dbPath);

                    final SharedPreferences prefs =
                        await SharedPreferences.getInstance();
                    await prefs.setString("username", username);
                    onRegister(username);
                  } catch (e) {
                    if (context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text('Registration failed: $e')),
                      );
                    }
                  }
                },
              ),
            ],
          ),
        ),
      ),
    );
  }
}
