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
        debugShowCheckedModeBanner: false,
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

enum SelectedDestination {
  chats,
  calls,
  stories,
}

class _HomePageState extends State<HomePage> {
  SelectedDestination destination = SelectedDestination.chats;
  String name = "Brennan";
  String username = "brongan.69";
  bool registered = false;
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

    WidgetsBinding.instance.addPostFrameCallback((_) async {
      TextEditingController usernameInput = TextEditingController();
      final registrationInfo = await showDialog(
        context: context,
        barrierDismissible: false,
        builder: (BuildContext context) => Center(
          child: AlertDialog(
            title: const Text('Register'),
            insetPadding: const EdgeInsets.all(20),
            backgroundColor: backgroundColor,
            contentPadding: const EdgeInsets.all(0),
            content: Padding(
              padding: const EdgeInsets.all(20.0),
              child: TextField(
                controller: usernameInput,
                decoration: const InputDecoration(hintText: "name"),
              ),
            ),
            actions: <Widget>[
              ElevatedButton(
                child: const Text('OK', style: conversationNameStyle),
                onPressed: () {
                  Navigator.pop(context, usernameInput.text);
                },
              ),
            ],
          ),
        ),
      );
      _register(registrationInfo);
    });
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final Widget body;
    if (destination == SelectedDestination.chats) {
      body = const ConversationsList();
    } else {
      body = Text("TODO", style: theme.textTheme.bodyMedium);
    }

    return Scaffold(
        appBar: getHomeAppBar(context),
        drawer: getHomeDrawer(context),
        backgroundColor: theme.colorScheme.background,
        body: body,
        floatingActionButton:
            BrongnalFloatingActionButtons(destination: destination),
        bottomNavigationBar: NavigationBar(
          backgroundColor: theme.bottomNavigationBarTheme.backgroundColor,
          indicatorColor: theme.navigationBarTheme.indicatorColor,
          height: 150,
          animationDuration: const Duration(milliseconds: 1000),
          destinations: [
            NavigationDestination(
              icon: Icon(Icons.chat_bubble_outline_outlined,
                  size: theme.iconTheme.size),
              selectedIcon: Icon(Icons.chat_bubble, size: theme.iconTheme.size),
              label: 'Chats',
            ),
            NavigationDestination(
              icon: Icon(Icons.call_outlined, size: theme.iconTheme.size),
              selectedIcon: Icon(Icons.call, size: theme.iconTheme.size),
              label: 'Calls',
            ),
            NavigationDestination(
              icon:
                  Icon(Icons.amp_stories_outlined, size: theme.iconTheme.size),
              selectedIcon: Icon(Icons.amp_stories, size: theme.iconTheme.size),
              label: 'Stories',
            ),
          ],
          selectedIndex: destination.index,
          onDestinationSelected: (int index) {
            setState(() {
              if (index == 1) {
                destination = SelectedDestination.calls;
              } else if (index == 2) {
                destination = SelectedDestination.stories;
              } else {
                destination = SelectedDestination.chats;
              }
            });
          },
        ));
  }

  void _register(String name) async {
    if (registered) {
      return;
    }
    try {
      final _ = await _stub.registerPreKeyBundle(
          RegisterPreKeyBundleRequest(identity: name),
          options: CallOptions(timeout: const Duration(seconds: 5)));
    } catch (e) {
      print('Caught error: $e');
    }
    setState(() {
      registered = true;
      name = name;
      username = "${name.toLowerCase()}.69";
    });
  }

  Drawer getHomeDrawer(BuildContext context) {
    final theme = DrawerTheme.of(context);
    return Drawer(
      child: ListView(
        padding: EdgeInsets.zero,
        children: <Widget>[
          DrawerHeader(
            decoration: BoxDecoration(
              color: theme.backgroundColor,
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
                  backgroundColor: AppBarTheme.of(context).foregroundColor,
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
                  return Scaffold(
                      appBar: AppBar(
                        title: const Text("Account"),
                      ),
                      body: const Text("TODO"));
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
                  return Scaffold(
                      appBar: AppBar(title: const Text("Appearance")),
                      body: const Text("TODO"));
                },
              ));
            },
          ),
        ],
      ),
    );
  }

  AppBar getHomeAppBar(BuildContext context) {
    final theme = AppBarTheme.of(context);
    return AppBar(
      title: Text('Brongnal', style: theme.titleTextStyle),
      toolbarHeight: theme.toolbarHeight,
      leading: Builder(
        builder: (BuildContext context) {
          return IconButton(
            icon: CircleAvatar(
              radius: 20,
              backgroundColor: theme.foregroundColor,
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
      backgroundColor: theme.backgroundColor,
      actions: <Widget>[
        const StubIconButton(icon: Icons.search_outlined, name: 'Search'),
        PopupMenuButton<HomepagePopupItem>(
          onSelected: (HomepagePopupItem item) {},
          iconColor: theme.foregroundColor,
          iconSize: appbarIconThemeSize,
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
  final SelectedDestination destination;
  const BrongnalFloatingActionButtons({super.key, required this.destination});

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: FloatingActionButton.large(
            backgroundColor: const Color.fromRGBO(47, 49, 51, 1.0),
            onPressed: () {},
            heroTag: "btn1",
            child: const Icon(Icons.photo_camera_outlined,
                color: textColor, size: 40),
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: FloatingActionButton.large(
              backgroundColor: const Color.fromRGBO(70, 75, 92, 1.0),
              onPressed: () {},
              heroTag: "btn2",
              child: const Icon(Icons.create_outlined,
                  color: textColor, size: 40)),
        ),
      ],
    );
  }
}
