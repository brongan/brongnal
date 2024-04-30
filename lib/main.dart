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
        theme: ThemeData.light(),
        darkTheme: ThemeData.dark(),
        themeMode: ThemeMode.system,
        home: const HomePage(),
      ),
    );
  }
}

/*
IconButton(
              icon: const Icon(Icons.navigate_next),
              tooltip: 'Go to the next page',
              onPressed: () {
                Navigator.push(context, MaterialPageRoute<void>(
                  builder: (BuildContext context) {
                    return Scaffold(
                      appBar: AppBar(
                        title: const Text('Next page'),
                      ),
                      body: const Center(
                        child: Text(
                          'This is the next page',
                          style: TextStyle(fontSize: 24),
                        ),
                      ),
                    );
                  },
                ));
              },
            ),

*/

enum SampleItem {
  newGroup,
  markAllRead,
  inviteFriends,
  settings,
}

enum MessageState {
  sending,
  sent,
  read,
}

class HomePage extends StatefulWidget {
  const HomePage({
    super.key,
  });

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  String selectedPage = '';

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(
          title: const Text('Brongnal'),
          leading: Builder(
            builder: (BuildContext context) {
              return IconButton(
                icon: const Icon(Icons.menu),
                onPressed: () {
                  Scaffold.of(context).openDrawer();
                },
                tooltip: MaterialLocalizations.of(context).openAppDrawerTooltip,
              );
            },
          ),
          backgroundColor: theme.primaryColorDark,
          actions: <Widget>[
            IconButton(
              icon: const Icon(Icons.search_outlined),
              tooltip: 'Search',
              onPressed: () {
                ScaffoldMessenger.of(context)
                    .showSnackBar(const SnackBar(content: Text('Search')));
              },
            ),
            PopupMenuButton<SampleItem>(
              onSelected: (SampleItem item) {},
              itemBuilder: (BuildContext context) =>
                  <PopupMenuEntry<SampleItem>>[
                const PopupMenuItem<SampleItem>(
                  value: SampleItem.newGroup,
                  child: Text('New Group'),
                ),
                const PopupMenuItem<SampleItem>(
                  value: SampleItem.markAllRead,
                  child: Text('Mark All Read'),
                ),
                const PopupMenuItem<SampleItem>(
                  value: SampleItem.inviteFriends,
                  child: Text('Invite Friends'),
                ),
                const PopupMenuItem<SampleItem>(
                  value: SampleItem.settings,
                  child: Text('Settings'),
                ),
              ],
            ),
          ],
        ),
        drawer: Drawer(
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
                    name: "Brennan",
                    username: "brongan.69",
                  )
                ]),
              ),
              ListTile(
                leading: const Icon(Icons.account_circle),
                title: const Text('Account'),
                onTap: () {
                  setState(() {
                    selectedPage = 'Account';
                  });
                },
              ),
              ListTile(
                leading: const Icon(Icons.message),
                title: const Text('Appearance'),
                onTap: () {
                  setState(() {
                    selectedPage = 'Appearance';
                  });
                },
              ),
            ],
          ),
        ),
        body: const ConversationsList(),
        floatingActionButton: const BrongnalFloatingActionButtons(),
        bottomNavigationBar: BottomNavigationBar(
          selectedItemColor: theme.bottomNavigationBarTheme.selectedItemColor,
          unselectedItemColor:
              theme.bottomNavigationBarTheme.unselectedItemColor,
          backgroundColor: theme.bottomNavigationBarTheme.backgroundColor,
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
        ),
      ),
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

class ConversationsList extends StatelessWidget {
  const ConversationsList({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Conversation(
          avatar: CircleAvatar(
            backgroundColor: randomColor(),
            child: const Text('Al'),
          ),
          name: "Alice",
          lastMessage: "Hello Brennan.",
          lastMessageTime: DateTime.utc(2024, 4, 30),
          messageState: MessageState.sent,
        ),
        Conversation(
          avatar: CircleAvatar(
            backgroundColor: randomColor(),
            child: const Text('Al'),
          ),
          name: "Alice",
          lastMessage: "Hi Alice",
          lastMessageTime: DateTime.utc(2024, 4, 30),
          messageState: MessageState.sending,
        ),
        Conversation(
          avatar: CircleAvatar(
            backgroundColor: randomColor(),
            child: const Text('MA'),
          ),
          name: "Madeleine Appelmans",
          lastMessage: "Bob please write better rust.",
          lastMessageTime: DateTime.utc(2024, 4, 29),
          messageState: MessageState.read,
        ),
      ],
    );
  }
}

IconData getIcon(MessageState messageState) {
  switch (messageState) {
    case MessageState.sending:
      return Icons.radio_button_unchecked_outlined;
    case MessageState.sent:
      return Icons.check_circle;
    case MessageState.read:
      return Icons.check_circle_outline_outlined;
  }
}

class Conversation extends StatelessWidget {
  final CircleAvatar avatar;
  final String name;
  final String lastMessage;
  final DateTime lastMessageTime;
  final MessageState messageState;
  const Conversation({
    super.key,
    required this.avatar,
    required this.name,
    required this.lastMessage,
    required this.lastMessageTime,
    required this.messageState,
  });

  @override
  Widget build(BuildContext context) {
    var delta = DateTime.now().difference(lastMessageTime).inHours;

    var readIcon = Icon(getIcon(messageState));
    return TextButton(
      onPressed: () {},
      onLongPress: null,
      child: SizedBox(
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
                      lastMessage,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ],
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(8.0),
              child: Column(
                children: [
                  Text('${delta}h'),
                  readIcon,
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class BrongnalFloatingActionButtons extends StatelessWidget {
  const BrongnalFloatingActionButtons({
    super.key,
  });

  @override
  // TODO remove outline when active.
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Column(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: FloatingActionButton(
            backgroundColor: theme.floatingActionButtonTheme.backgroundColor,
            onPressed: () {},
            child: const Icon(Icons.photo_camera_outlined),
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: FloatingActionButton(
              foregroundColor: theme.floatingActionButtonTheme.foregroundColor,
              onPressed: () {},
              child: const Icon(Icons.create_outlined)),
        ),
      ],
    );
  }
}
