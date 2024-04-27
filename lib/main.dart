import 'package:flutter/material.dart';
import 'dart:math' as math;
import 'package:provider/provider.dart';

void main() {
  runApp(const BrongnalApp());
}

Color randomColor() {
  final random = math.Random();
  return Color.fromRGBO(random.nextInt(256), random.nextInt(256),
      random.nextInt(256), 1.0); // 1.0 for full opacity
}

class BrongnalAppState extends ChangeNotifier {}

class BrongnalApp extends StatelessWidget {
  const BrongnalApp({super.key});

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
      create: (context) => BrongnalAppState(),
      child: MaterialApp(
        title: 'Brongnal',
        theme: ThemeData(
          useMaterial3: true,
          colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepOrange),
        ),
        home: const HomePage(),
      ),
    );
  }
}

class HomePage extends StatelessWidget {
  const HomePage({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(
          backgroundColor: theme.dialogBackgroundColor,
          title: const Text("Brongnal"),
        ),
        body: const Text("hello world"),
        backgroundColor: theme.colorScheme.inversePrimary,
        floatingActionButton: FloatingActionButton(
          onPressed: () {
            // ToDo create message modal?
          },
          child: const Icon(
            Icons.message,
          ),
        ),
        bottomNavigationBar: BottomNavigationBar(
          selectedItemColor: theme.bottomNavigationBarTheme.selectedItemColor,
          backgroundColor: theme.bottomNavigationBarTheme.backgroundColor,
          items: const [
            BottomNavigationBarItem(
              icon: Icon(Icons.messenger),
              label: 'Chat',
            ),
            BottomNavigationBarItem(
              icon: Icon(Icons.info),
              label: 'Info',
            ),
          ],
        ),
      ),
    );
  }
}
