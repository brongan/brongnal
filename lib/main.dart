import 'dart:io' show Platform, Directory;
import 'dart:ui';

import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/firebase_options.dart';
import 'package:brongnal_app/common/config.dart';
import 'package:brongnal_app/models/chat_history.dart';
import 'package:brongnal_app/screens/home.dart';
import 'package:brongnal_app/screens/register.dart';
import 'package:brongnal_app/common/core.dart';
import 'package:brongnal_app/src/rust/bridge.dart' show MessageModel;
import 'package:brongnal_app/src/rust/frb_generated.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:xdg_directories/xdg_directories.dart';

int id = 0;

Future<void> _onMessageReceived(MessageModel message) async {
  FlutterLocalNotificationsPlugin plugin = await createLocalNotifications();
  await plugin.show(id++, message.sender, message.text, toNotification(message),
      payload: message.sender);
}

Future<void> _firebaseMessagingHandler(RemoteMessage remoteMessage) async {
  if (!RustLib.instance.initialized) {
    await RustLib.init();
  }
  final SharedPreferences prefs = await SharedPreferences.getInstance();
  final username = prefs.getString("username");

  Directory databaseDirectory;
  try {
    databaseDirectory = Directory(p.join(dataHome.path, "brongnal"));
  } on StateError catch (_) {
    databaseDirectory = await getApplicationCacheDirectory();
  }

  final bridge = const RustBrongnalCore();
  await core.startHub(
    databaseDirectory: databaseDirectory.path,
    username: username,
    mailboxAddress: const String.fromEnvironment('MAILBOX_ADDR',
        defaultValue: 'https://signal.brongan.com:443'),
    identityAddress: const String.fromEnvironment('IDENTITY_ADDR',
        defaultValue: 'https://gossamer.brongan.com:443'),
  );

  FlutterLocalNotificationsPlugin plugin = await createLocalNotifications();

  core.subscribeMessages().listen((message) async {
    await plugin.show(
        id++, message.sender, message.text, toNotification(message),
        payload: message.sender);
  }, onError: (e) {
    debugPrint("subscribeMessages stream error: $e");
  });
}

@pragma('vm:entry-point')
Future<void> _firebaseMessagingBackgroundHandler(RemoteMessage message) async {
  debugPrint("_firebaseMessagingBackgroundHandler: ${message.messageId}");
  return _firebaseMessagingHandler(message);
}

void _firebaseMessagingForegroundHandler(RemoteMessage message) {
  debugPrint("_firebaseMessagingForegroundHandler: ${message.messageId}");
  _firebaseMessagingHandler(message);
}

void onDidReceiveNotificationResponse(
    NotificationResponse notificationResponse) async {
  debugPrint("onDidReceiveNotificationResponse: ${notificationResponse.id}");
  final String? payload = notificationResponse.payload;
  if (notificationResponse.payload != null) {
    debugPrint('notification payload: $payload');
  }
  // TODO make something happen when notification is pressed.
}

Future<FlutterLocalNotificationsPlugin> createLocalNotifications() async {
  FlutterLocalNotificationsPlugin flutterLocalNotificationsPlugin =
      FlutterLocalNotificationsPlugin();
  const AndroidInitializationSettings initializationSettingsAndroid =
      AndroidInitializationSettings('@mipmap/brongnal_launcher');
  final DarwinInitializationSettings initializationSettingsDarwin =
      DarwinInitializationSettings();
  final LinuxInitializationSettings initializationSettingsLinux =
      LinuxInitializationSettings(defaultActionName: 'Open notification');
  final WindowsInitializationSettings initializationSettingsWindows =
      WindowsInitializationSettings(
          appName: 'Flutter Local Notifications Example',
          appUserModelId: 'com.brongan.Brongnal',
          guid: '10759e20-4f29-4646-97cb-15acfd7fc208');
  final InitializationSettings initializationSettings = InitializationSettings(
      android: initializationSettingsAndroid,
      iOS: initializationSettingsDarwin,
      macOS: initializationSettingsDarwin,
      linux: initializationSettingsLinux,
      windows: initializationSettingsWindows);
  await flutterLocalNotificationsPlugin.initialize(initializationSettings,
      onDidReceiveNotificationResponse: onDidReceiveNotificationResponse);
  return flutterLocalNotificationsPlugin;
}

Future<String?> notificationsToken(FirebaseMessaging messaging) async {
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
    debugPrint('User granted permission: ${settings.authorizationStatus}');
    final String? token = await messaging.getToken();
    debugPrint('Firebase Token: $token');
    return token;
  }
  return null;
}

void main() async {
  await runBrongnalApp();
}

Future<void> runBrongnalApp({String? dbDirOverride}) async {
  setupWindow();

  final SharedPreferences prefs = await SharedPreferences.getInstance();
  final String? savedUsername = prefs.getString("username");

  final String? fcmToken;
  if (!Platform.isLinux) {
    await Firebase.initializeApp(
      options: DefaultFirebaseOptions.currentPlatform,
    );
    fcmToken = await notificationsToken(FirebaseMessaging.instance);
    FirebaseMessaging.onBackgroundMessage(_firebaseMessagingBackgroundHandler);
    FirebaseMessaging.onMessage.listen(_firebaseMessagingForegroundHandler);
  } else {
    fcmToken = null;
  }

  if (dbDirOverride != null) {
    AppConfig.setDatabaseOverride(dbDirOverride);
  }

  if (!RustLib.instance.initialized) {
    await RustLib.init();
  }

  // Determine database directory
  final String dbPath = await AppConfig.getDatabaseDirectory(override: dbDirOverride);

  final bridge = const RustBrongnalCore();
  if (savedUsername != null) {
    try {
      final watch = Stopwatch()..start();
      await core.startHub(
        databaseDirectory: dbPath,
        username: savedUsername,
        mailboxAddress: AppConfig.defaultMailboxAddr,
        identityAddress: AppConfig.defaultIdentityAddr,
      );
      watch.stop();
      debugPrint("Rust Hub initialized in ${watch.elapsedMilliseconds} ms");
    } catch (e) {
      debugPrint("Failed to start hub: $e");
    }
  }

  runApp(BrongnalApp(username: savedUsername, core: bridge));
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
  const BrongnalApp({super.key, required this.username, required this.core});
  final String? username;
  final BrongnalCore core;

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
    _listener = AppLifecycleListener(
      onExitRequested: () async {
        return AppExitResponse.exit;
      },
    );
  }

  @override
  void dispose() {
    _listener.dispose();
    super.dispose();
  }


  @override
  Widget build(BuildContext context) {
    final Widget child;
    if (username == null) {
      child = Register(
          core: widget.bridge,
          onRegister: (newUsername) {
            setState(() {
              username = newUsername;
            });
          });
    } else {
      child = ChangeNotifierProvider(
        create: (context) => ChatHistory(
            username: username!,
            onMessageReceived: _onMessageReceived,
            core: widget.bridge),
        child: Navigator(
          pages: [
            MaterialPage(child: Home(username: username!)),
          ],
          onDidRemovePage: (object) {},
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
