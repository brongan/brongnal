import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/models/conversations.dart';
import 'package:brongnal_app/screens/home.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import './messages/generated.dart';
import 'dart:io' show Platform;
import 'package:flutter/foundation.dart' show kIsWeb;

void main() async {
  setupWindow();
  await initializeRust();
  runApp(
    ChangeNotifierProvider(
      create: (context) => ConversationModel(),
      child: const BrongnalApp(),
    ),
  );
}

void setupWindow() {
  if (!kIsWeb && (Platform.isWindows || Platform.isLinux || Platform.isMacOS)) {
    WidgetsFlutterBinding.ensureInitialized();
  }
}

class BrongnalApp extends StatelessWidget {
  const BrongnalApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Brongnal',
      debugShowCheckedModeBanner: false,
      darkTheme: bronganlDarkTheme,
      themeMode: ThemeMode.dark,
      home: Consumer<ConversationModel>(
        builder: (_, model, __) => Home(conversations: model),
      ),
    );
  }
}
