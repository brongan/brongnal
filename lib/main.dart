import 'conversation_list.dart';
import 'package:brongnal_app/generated/service.pbgrpc.dart';
import 'package:flutter/material.dart';
import 'package:grpc/grpc.dart';
import 'package:provider/provider.dart';
import 'theme.dart';
import 'util.dart';

void main() {
  runApp(const BrongnalApp());
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
        darkTheme: bronganlDarkTheme,
        themeMode: ThemeMode.dark,
        home: const HomePage(),
      ),
    );
  }
}

enum HomepagePopupItem {
  newGroup,
  markAllRead,
  inviteFriends,
  settings,
}

class HomePage extends StatefulWidget {
  const HomePage({
    super.key,
  });

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  String name = "Brennan";
  String username = "brongan.69";
  late ClientChannel _channel;
  late BrongnalClient _stub;

  @override
  void initState() {
    super.initState();
    _channel = ClientChannel('signal.brongan.com',
        port: 443,
        options:
            const ChannelOptions(credentials: ChannelCredentials.secure()));
    _stub = BrongnalClient(_channel,
        options: CallOptions(timeout: const Duration(seconds: 30)));
  }

  void _register() async {
    await _stub
        .registerPreKeyBundle(RegisterPreKeyBundleRequest(identity: username));
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Scaffold(
      appBar: getHomeAppBar(context),
      drawer: getHomeDrawer(theme),
      backgroundColor: theme.colorScheme.background,
      body: const ConversationsList(),
      floatingActionButton: const BrongnalFloatingActionButtons(),
      bottomNavigationBar: getBottomNavBar(),
    );
  }

  BottomNavigationBar getBottomNavBar() {
    final theme = Theme.of(context);
    return BottomNavigationBar(
      backgroundColor: theme.navigationBarTheme.backgroundColor,
      items: const [
        BottomNavigationBarItem(
          icon: Icon(Icons.chat_bubble_outline_outlined),
          label: 'Chats',
        ),
        BottomNavigationBarItem(
          icon: Icon(Icons.call_outlined),
          label: 'Calls',
        ),
        BottomNavigationBarItem(
          icon: Icon(Icons.web_stories_outlined),
          label: 'Stories',
        ),
      ],
    );
  }

  Drawer getHomeDrawer(ThemeData theme) {
    return Drawer(
      child: ListView(
        padding: EdgeInsets.zero,
        children: <Widget>[
          DrawerHeader(
            decoration: BoxDecoration(
              color: theme.colorScheme.background,
            ),
            child: Column(children: [
              const Text(
                'Settings',
                style: TextStyle(
                  fontSize: 24,
                ),
              ),
              AccountInfo(
                avatar: CircleAvatar(
                  backgroundColor: randomColor(),
                  child: const Text('BR', style: TextStyle(fontSize: 24)),
                ),
                name: name,
                username: username,
              )
            ]),
          ),
          ListTile(
            leading: const Icon(Icons.account_circle),
            title: const Text('Account'),
            onTap: () {
              Navigator.push(context, MaterialPageRoute<void>(
                builder: (BuildContext context) {
                  return const Scaffold(body: Text("TODO"));
                },
              ));
            },
          ),
          ListTile(
            leading: const Icon(Icons.message),
            title: const Text('Appearance'),
            onTap: () {
              Navigator.push(context, MaterialPageRoute<void>(
                builder: (BuildContext context) {
                  return const Scaffold(body: Text("TODO"));
                },
              ));
            },
          ),
        ],
      ),
    );
  }

  AppBar getHomeAppBar(BuildContext context) {
    final theme = Theme.of(context);
    return AppBar(
      title: const Text('Brongnal'),
      leading: Builder(
        builder: (BuildContext context) {
          return IconButton(
            icon: CircleAvatar(
              radius: 16,
              backgroundColor: randomColor(),
              child: const Text(
                'BR',
                style: TextStyle(fontSize: 16),
              ),
            ),
            onPressed: () {
              Scaffold.of(context).openDrawer();
            },
            tooltip: MaterialLocalizations.of(context).openAppDrawerTooltip,
          );
        },
      ),
      backgroundColor: theme.colorScheme.background,
      actions: <Widget>[
        const StubIconButton(icon: Icons.search_outlined, name: 'Search'),
        PopupMenuButton<HomepagePopupItem>(
          onSelected: (HomepagePopupItem item) {},
          iconColor: textColor,
          iconSize: 24,
          itemBuilder: (BuildContext context) =>
              <PopupMenuEntry<HomepagePopupItem>>[
            const PopupMenuItem<HomepagePopupItem>(
              value: HomepagePopupItem.newGroup,
              child: Text('New Group'),
            ),
            const PopupMenuItem<HomepagePopupItem>(
              value: HomepagePopupItem.markAllRead,
              child: Text('Mark All Read'),
            ),
            const PopupMenuItem<HomepagePopupItem>(
              value: HomepagePopupItem.inviteFriends,
              child: Text('Invite Friends'),
            ),
            const PopupMenuItem<HomepagePopupItem>(
              value: HomepagePopupItem.settings,
              child: Text('Settings'),
            ),
          ],
        ),
      ],
    );
  }
}

class AccountInfo extends StatelessWidget {
  final CircleAvatar avatar;
  final String name;
  final String username;

  const AccountInfo({
    super.key,
    required this.avatar,
    required this.name,
    required this.username,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: 60,
      child: Row(
        children: [
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: avatar,
          ),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.all(4.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Text(
                    name,
                    overflow: TextOverflow.ellipsis,
                  ),
                  Text(
                    username,
                    overflow: TextOverflow.ellipsis,
                  ),
                ],
              ),
            ),
          ),
          const Padding(
            padding: EdgeInsets.all(8.0),
            child: Icon(Icons.qr_code),
          ),
        ],
      ),
    );
  }
}

class BrongnalFloatingActionButtons extends StatelessWidget {
  const BrongnalFloatingActionButtons({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: FloatingActionButton(
            backgroundColor: const Color.fromRGBO(47, 49, 51, 1.0),
            onPressed: () {},
            heroTag: "btn1",
            child: const Icon(Icons.photo_camera_outlined, color: textColor),
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: FloatingActionButton(
              backgroundColor: const Color.fromRGBO(70, 75, 92, 1.0),
              onPressed: () {},
              heroTag: "btn2",
              child: const Icon(Icons.create_outlined, color: textColor)),
        ),
      ],
    );
  }
}
