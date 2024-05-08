import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/screens/home.dart';
import 'package:flutter/material.dart';

void main() {
  runApp(const BrongnalApp());
}

class BrongnalAppState extends ChangeNotifier {}

class BrongnalApp extends StatelessWidget {
  const BrongnalApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Brongnal',
      debugShowCheckedModeBanner: false,
      darkTheme: bronganlDarkTheme,
      themeMode: ThemeMode.dark,
      home: const Home(),
    );
  }
}
