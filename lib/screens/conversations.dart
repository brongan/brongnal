import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/models/conversations.dart';
import 'package:brongnal_app/models/message.dart';
import 'package:brongnal_app/screens/chat.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class ConversationsScreen extends StatelessWidget {
  const ConversationsScreen({
    super.key,
    required this.self,
    required this.conversations,
  });
  final String self;
  final ConversationModel conversations;

  @override
  Widget build(BuildContext context) {
    final items = conversations.items;
    return ListView.builder(
        itemCount: items.length,
        itemBuilder: (context, i) {
          final peer = items.keys.elementAt(i);
          return Conversation(
            avatar: CircleAvatar(
                backgroundColor: Colors.primaries[i % Colors.primaries.length],
                radius: 36,
                child: Text(peer.substring(0, 2))),
            lastMessage: items.values.elementAt(i).last,
            self: self,
            peer: peer,
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
  final MessageModel lastMessage;
  final String self;
  final String peer;
  const Conversation({
    super.key,
    required this.avatar,
    required this.lastMessage,
    required this.self,
    required this.peer,
  });

  @override
  Widget build(BuildContext context) {
    final delta = DateTime.now().difference(lastMessage.time).inHours;
    final theme = Theme.of(context);

    var readIcon = Icon(
      getIcon(lastMessage.state),
      color: textColor,
      size: 18,
    );
    return TextButton(
      onPressed: () {
        Navigator.push(
          context,
          MaterialPageRoute<void>(
            builder: (context) => Consumer<ConversationModel>(
              builder: (context, conversationModel, child) {
                return ChatScreen(
                  self: self,
                  peer: peer,
                  messages: conversationModel.items[peer]!,
                );
              },
            ),
          ),
        );
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
