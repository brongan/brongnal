import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/screens/chat.dart';
import 'package:flutter/material.dart';

class Conversations extends StatelessWidget {
  const Conversations({
    super.key,
    required this.conversations,
  });
  final Map<String, List<MessageModel>> conversations;

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
        itemCount: conversations.length,
        itemBuilder: (context, i) {
          final peer = conversations.keys.elementAt(i);
          return Conversation(
            avatar: CircleAvatar(
                backgroundColor: Colors.primaries[i % Colors.primaries.length],
                radius: 36,
                child: Text(peer.substring(0, 2))),
            peer: peer,
            messages: conversations.values.elementAt(i),
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
  final String peer;
  final List<MessageModel> messages;
  const Conversation({
    super.key,
    required this.avatar,
    required this.peer,
    required this.messages,
  });

  @override
  Widget build(BuildContext context) {
    final lastMessage = messages.last;
    final delta = DateTime.now().difference(lastMessage.time).inHours;
    final theme = Theme.of(context);

    var readIcon = Icon(
      getIcon(lastMessage.state),
      color: textColor,
      size: 18,
    );
    return TextButton(
      onPressed: () {
        Navigator.push(context, MaterialPageRoute<void>(
          builder: (BuildContext context) {
            return Chat(name: peer, messages: messages);
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
                      peer,
                      overflow: TextOverflow.ellipsis,
                      style: theme.textTheme.bodyMedium,
                    ),
                    Text(
                      lastMessage.message,
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
