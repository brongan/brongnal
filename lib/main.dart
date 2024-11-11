import 'package:firebase_messaging/firebase_messaging.dart';

import './messages/generated.dart';
import 'dart:io' show Platform, Directory;
import 'messages/brongnal.pb.dart';
import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/database.dart';
import 'package:brongnal_app/generated/service.pbgrpc.dart';
import 'package:brongnal_app/models/conversations.dart';
import 'package:brongnal_app/screens/home.dart';
import 'package:brongnal_app/screens/register.dart';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart';
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
}

void main() async {
  FirebaseMessaging.onBackgroundMessage(_firebaseMessagingBackgroundHandler);
  setupWindow();
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
  late String? fcmToken;
  late ClientChannel _channel;
  late BrongnalClient _stub;

  @override
  void initState() {
    super.initState();
    username = widget.username;
    _channel = ClientChannel('signal.brongan.com',
        port: 443,
        options:
            const ChannelOptions(credentials: ChannelCredentials.secure()));
    _stub = BrongnalClient(_channel,
        options: CallOptions(timeout: const Duration(seconds: 30)));
    listenForRegister();
    setupNotifications();
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

  void setupNotifications() async {
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

    print('User granted permission: ${settings.authorizationStatus}');
    String? token;
    if (settings.authorizationStatus == AuthorizationStatus.authorized) {
      token = await messaging.getToken();
      print('Firebase Token: $token');
      setState(() {
        fcmToken = token;
      });
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
      child = Register(stub: _stub, fcmToken: fcmToken);
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
