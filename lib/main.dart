import 'package:brongnal_app/firebase_options.dart';
import 'dart:io' show Platform, Directory;
import 'dart:ui';

import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/database.dart';
import 'package:brongnal_app/src/bindings/bindings.dart';
import 'package:brongnal_app/models/conversations.dart';
import 'package:brongnal_app/screens/home.dart';
import 'package:brongnal_app/screens/register.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:provider/provider.dart';
import 'package:rinf/rinf.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:xdg_directories/xdg_directories.dart';

@pragma('vm:entry-point')
Future<void> _firebaseMessagingBackgroundHandler(RemoteMessage message) async {
  print("Handling a background message: ${message.messageId}");
  await initializeRust(assignRustSignal);
  final SharedPreferences prefs = await SharedPreferences.getInstance();
  final username = prefs.getString("username");

  Directory databaseDirectory;
  try {
    databaseDirectory = Directory(p.join(dataHome.path, "brongnal"));
  } on StateError catch (_) {
    databaseDirectory = await getApplicationCacheDirectory();
  }
  RustStartup(databaseDirectory: databaseDirectory.path, username: username)
      .sendSignalToRust();
  PushNotification(message: message.data["payload"]).sendSignalToRust();
}

Future<String?> setupNotifications() async {
  FirebaseMessaging messaging = FirebaseMessaging.instance;
  NotificationSettings settings = await messaging.requestPermission(
    alert: true,
    announcement: false,
    badge: true,
    carPlay: false,
    criticalAlert: false,
    provisional: false,
    sound: true,
  );

  if (settings.authorizationStatus == AuthorizationStatus.authorized) {
    print('User granted permission: ${settings.authorizationStatus}');
    final String? token = await messaging.getToken();
    print('Firebase Token: $token');
    return token;
  }
  return null;
}

void main() async {
  setupWindow();
  final String? fcmToken;
  if (!Platform.isLinux) {
    await Firebase.initializeApp(
      options: DefaultFirebaseOptions.currentPlatform,
    );
    FirebaseMessaging.onBackgroundMessage(_firebaseMessagingBackgroundHandler);
    fcmToken = await setupNotifications();
  } else {
    fcmToken = null;
  }
  await initializeRust(assignRustSignal);

  final SharedPreferences prefs = await SharedPreferences.getInstance();
  final username = prefs.getString("username");

  Directory databaseDirectory;
  try {
    databaseDirectory = Directory(p.join(dataHome.path, "brongnal"));
  } on StateError catch (_) {
    databaseDirectory = await getApplicationCacheDirectory();
  }

  RustStartup(
          databaseDirectory: databaseDirectory.path,
          username: username,
          fcmToken: fcmToken)
      .sendSignalToRust();

  final database = AppDatabase(databaseDirectory);
  List<MessageModel> allMessages =
      await database.select(database.messages).get();
  final Map<String, List<MessageModel>> conversations = {};
  for (final messageModel in allMessages) {
    final peer = messageModel.sender == username
        ? messageModel.receiver
        : messageModel.sender;
    conversations.putIfAbsent(peer, () => []);
    conversations[peer]!.add(messageModel);
  }

  runApp(
    ChangeNotifierProvider(
      create: (context) => ConversationModel(
        database: database,
        conversations: conversations,
      ),
      child: BrongnalApp(username: username, directory: databaseDirectory),
    ),
  );
}

void setupWindow() {
  if (!kIsWeb &&
      (Platform.isWindows ||
          Platform.isLinux ||
          Platform.isMacOS ||
          Platform.isAndroid)) {
    WidgetsFlutterBinding.ensureInitialized();
  }
}

class BrongnalApp extends StatefulWidget {
  const BrongnalApp(
      {super.key, required this.username, required this.directory});
  final String? username;
  final Directory directory;

  @override
  State<BrongnalApp> createState() => _BrongnalAppState();
}

class _BrongnalAppState extends State<BrongnalApp> {
  _BrongnalAppState();
  String? username;
  late final AppLifecycleListener _listener;

  @override
  void initState() {
    super.initState();
    username = widget.username;
    listenForRegister();
    _listener = AppLifecycleListener(
      onExitRequested: () async {
        finalizeRust(); // Shut down the async Rust runtime.
        return AppExitResponse.exit;
      },
    );
  }

  @override
  void dispose() {
    _listener.dispose();
    super.dispose();
  }

  void listenForRegister() async {
    final stream = RegisterUserResponse.rustSignalStream;
    await for (final rustSignal in stream) {
      RegisterUserResponse message = rustSignal.message;
      setState(() {
        username = message.username;
      });
      final SharedPreferences prefs = await SharedPreferences.getInstance();
      prefs.setString("username", message.username);
    }
  }

  void listenForPushNotifications() async {
    FirebaseMessaging.onMessage.listen((RemoteMessage message) {
      print('Got a message whilst in the foreground!');
      print('Message data: ${message.data}');
    });
  }

  @override
  Widget build(BuildContext context) {
    final Widget child;
    if (username == null) {
      child = Register();
    } else {
      child = Consumer<ConversationModel>(
        builder: (_, model, __) => Home(
          conversations: model,
          username: username!,
        ),
      );
    }

    return MaterialApp(
      title: 'Brongnal',
      debugShowCheckedModeBanner: false,
      darkTheme: bronganlDarkTheme,
      themeMode: ThemeMode.dark,
      home: child,
    );
  }
}
