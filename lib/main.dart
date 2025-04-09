import 'dart:io' show Platform, Directory;
import 'dart:ui';

import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/database.dart';
import 'package:brongnal_app/generated/service.pbgrpc.dart';
import 'package:brongnal_app/messages/generated.dart';
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
import 'package:brongnal_app/messages/brongnal.pb.dart';

void main() async {
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
  late ClientChannel _channel;
  late BrongnalClient _stub;
  late final AppLifecycleListener _listener;

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

  @override
  Widget build(BuildContext context) {
    final Widget child;
    if (username == null) {
      child = Register(stub: _stub);
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
