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
        themeMode: ThemeMode.dark, // Default mode
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
                child: const Text(
                  'Settings',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 24,
                  ),
                ),
              ),
              ListTile(
                leading: const Icon(Icons.message),
                title: const Text('Messages'),
                onTap: () {
                  setState(() {
                    selectedPage = 'Messages';
                  });
                },
              ),
              ListTile(
                leading: const Icon(Icons.account_circle),
                title: const Text('Profile'),
                onTap: () {
                  setState(() {
                    selectedPage = 'Profile';
                  });
                },
              ),
              ListTile(
                leading: const Icon(Icons.settings),
                title: const Text('Settings'),
                onTap: () {
                  setState(() {
                    selectedPage = 'Settings';
                  });
                },
              ),
            ],
          ),
        ),
        body: Conversation(
          avatar: CircleAvatar(
            backgroundColor: Colors.brown.shade800,
            child: const Text('AH'),
          ),
          name: "Alice",
          lastMessage: "Hello Bob.",
          lastMessageTime: DateTime.utc(1970, 0, 0),
          read: true,
        ),
        // TODO make text white so this works.
        // backgroundColor: theme.colorScheme.background,
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

class Conversation extends StatelessWidget {
  final CircleAvatar avatar;
  final String name;
  final String lastMessage;
  final DateTime lastMessageTime;
  final bool read;
  const Conversation({
    super.key,
    required this.avatar,
    required this.name,
    required this.lastMessage,
    required this.lastMessageTime,
    required this.read,
  });

  @override
  Widget build(BuildContext context) {
    // TODO Implement conversation rendering.
    return Row(
      children: [
        avatar,
        Text(name),
        Text(lastMessage),
        Text('$lastMessageTime'),
      ],
    );
  }
}

class ConversationsList extends StatelessWidget {
  const ConversationsList({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return const Text("hello world");
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
            onPressed: () {
              // TODO create message modal.
            },
            child: const Icon(Icons.photo_camera_outlined),
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: FloatingActionButton(
              foregroundColor: theme.floatingActionButtonTheme.foregroundColor,
              onPressed: () {
                // TODO send image.
              },
              child: const Icon(Icons.create_outlined)),
        ),
      ],
    );
  }
}
