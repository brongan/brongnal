import 'package:flutter/material.dart';
import 'dart:math' as math;
import 'package:provider/provider.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:random_name_generator/random_name_generator.dart';

void main() {
  runApp(const BrongnalApp());
}

const Color background = Color.fromRGBO(26, 28, 32, 1.0);
const Color textColor = Color.fromRGBO(190, 192, 197, 1.0);
const String loremIpsum =
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.';

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
        theme: ThemeData(
          useMaterial3: true,
          colorScheme: ColorScheme.fromSeed(
            seedColor: Colors.purple,
            brightness: Brightness.dark,
          ),
          textTheme: TextTheme(
            displayLarge: const TextStyle(
              fontSize: 72,
              fontWeight: FontWeight.bold,
            ),
            titleLarge: GoogleFonts.oswald(
              fontSize: 30,
              fontStyle: FontStyle.italic,
            ),
            bodyMedium: GoogleFonts.merriweather(),
            bodySmall: GoogleFonts.roboto(),
          ),
        ),
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
        appBar: getAppBar(context),
        drawer: getDrawer(theme),
        backgroundColor: background,
        body: const ConversationsList(),
        floatingActionButton: const BrongnalFloatingActionButtons(),
        bottomNavigationBar: getBottomNavBar(),
      ),
    );
  }

  BottomNavigationBar getBottomNavBar() {
    return BottomNavigationBar(
      selectedItemColor: textColor,
      unselectedItemColor: textColor,
      backgroundColor: const Color.fromRGBO(40, 43, 48, 1.0),
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

  Drawer getDrawer(ThemeData theme) {
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
    );
  }

  AppBar getAppBar(BuildContext context) {
    return AppBar(
      title: const Text('Brongnal', style: TextStyle(color: textColor)),
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
      backgroundColor: background,
      actions: <Widget>[
        IconButton(
          icon: const Icon(
            Icons.search_outlined,
            color: textColor,
            size: 24,
          ),
          tooltip: 'Search',
          onPressed: () {
            ScaffoldMessenger.of(context)
                .showSnackBar(const SnackBar(content: Text('Search')));
          },
        ),
        PopupMenuButton<SampleItem>(
          onSelected: (SampleItem item) {},
          iconColor: textColor,
          iconSize: 24,
          itemBuilder: (BuildContext context) => <PopupMenuEntry<SampleItem>>[
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
    var randomNames = RandomNames(Zone.us);
    return ListView.builder(itemBuilder: (context, index) {
      var name = randomNames.fullName();
      return Conversation(
        avatar: CircleAvatar(
          backgroundColor: randomColor(),
          child: Text(name.substring(0, 2)),
        ),
        name: name,
        lastMessage: loremIpsum,
        lastMessageTime: DateTime.utc(2024, 4, 30),
        messageState: MessageState.sent,
      );
    });
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
    final theme = Theme.of(context);

    var readIcon = Icon(
      getIcon(messageState),
      color: textColor,
      size: 14,
    );
    return TextButton(
      onPressed: () {},
      onLongPress: null,
      child: SizedBox(
        height: 76,
        child: Row(
          children: [
            Padding(
              padding: const EdgeInsets.all(12.0),
              child: avatar,
            ),
            Expanded(
              child: Padding(
                padding: const EdgeInsets.all(8.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text(
                      name,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        fontStyle: FontStyle.normal,
                        fontWeight: FontWeight.w500,
                        fontFamily: 'Roboto',
                        fontSize: 17,
                        color: textColor,
                      ),
                    ),
                    Text(
                      lastMessage,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        height: 1.15,
                        fontStyle: FontStyle.normal,
                        fontWeight: FontWeight.w400,
                        fontFamily: 'Roboto',
                        fontSize: 14,
                        color: textColor,
                      ),
                      maxLines: 2,
                    ),
                  ],
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(8.0),
              child: Column(
                children: [
                  Text(
                    '${delta}h',
                    style: const TextStyle(color: textColor),
                  ),
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
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Column(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: FloatingActionButton(
            backgroundColor: const Color.fromRGBO(47, 49, 51, 1.0),
            onPressed: () {},
            child: const Icon(Icons.photo_camera_outlined, color: textColor),
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: FloatingActionButton(
              backgroundColor: const Color.fromRGBO(70, 75, 92, 1.0),
              onPressed: () {},
              child: const Icon(Icons.create_outlined, color: textColor)),
        ),
      ],
    );
  }
}
