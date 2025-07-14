import 'dart:io' show Platform, Directory;
import 'dart:ui';

import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/firebase_options.dart';
import 'package:brongnal_app/models/chat_history.dart';
import 'package:brongnal_app/screens/home.dart';
import 'package:brongnal_app/screens/register.dart';
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

Future<void> _firebaseMessagingHandler(RemoteMessage message) async {
	await RustLib.init();
	debugPrint("Initialized Rust");
	final SharedPreferences prefs = await SharedPreferences.getInstance();
	final username = prefs.getString("username");
	debugPrint("Acquired Username");

	Directory databaseDirectory;
	try {
		databaseDirectory = Directory(p.join(dataHome.path, "brongnal"));
	} on StateError catch (_) {
		databaseDirectory = await getApplicationCacheDirectory();
	}
	debugPrint("Telling rust about database and username.");
	final User = await flutter_init_user(databaseDirectory.path, username);
	RustStartup(databaseDirectory: databaseDirectory.path, username: username)
		.sendSignalToRust();
	FlutterLocalNotificationsPlugin plugin = await createLocalNotifications();

	await for (final rustSignal in MessageModel.rustSignalStream) {
		final MessageModel message = rustSignal.message;
		await plugin.show(
				id++, message.sender, message.text, toNotification(message),
				payload: message.sender);
	}
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
	TODO make something happen when notification is pressed.
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
	setupWindow();
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
	await RustLib.init();

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

	runApp(BrongnalApp(username: username));
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
	const BrongnalApp({super.key, required this.username});
	final String? username;

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
					finalizeRust();  Shut down the async Rust runtime.
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

	@override
		Widget build(BuildContext context) {
			final Widget child;
			if (username == null) {
				child = const Register();
			} else {
				child = ChangeNotifierProvider(
						create: (context) => ChatHistory(
							username: username!, onMessageReceived: _onMessageReceived),
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

