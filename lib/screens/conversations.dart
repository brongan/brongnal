import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/screens/chat.dart';
import 'package:random_name_generator/random_name_generator.dart';
import 'package:flutter/material.dart';

const String loremIpsum =
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.';

enum MessageState {
  sending,
  sent,
  read,
}

class Conversations extends StatelessWidget {
  const Conversations({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final randomNames = RandomNames(Zone.us);
    return ListView.builder(itemBuilder: (context, index) {
      final name = randomNames.fullName();
      return Conversation(
        avatar: CircleAvatar(
          backgroundColor: Colors.primaries[index % Colors.primaries.length],
          radius: 36,
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
    final delta = DateTime.now().difference(lastMessageTime).inHours;
    final theme = Theme.of(context);

    var readIcon = Icon(
      getIcon(messageState),
      color: textColor,
      size: 18,
    );
    return TextButton(
      onPressed: () {
        Navigator.push(context, MaterialPageRoute<void>(
          builder: (BuildContext context) {
            return Chat(name: name, lastMessage: lastMessage);
          },
        ));
      },
      onLongPress: null,
      child: SizedBox(
        child: Row(
          children: [
            Padding(
              padding: const EdgeInsets.all(24.0),
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
                      style: theme.textTheme.bodyMedium,
                    ),
                    Text(
                      lastMessage,
                      overflow: TextOverflow.ellipsis,
                      style: theme.textTheme.bodySmall,
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
                    style: theme.textTheme.bodySmall,
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
