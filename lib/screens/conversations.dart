import 'dart:collection';
import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/src/bindings/bindings.dart';
import 'package:brongnal_app/screens/chat.dart';
import 'package:flutter/material.dart';
import 'package:timeago/timeago.dart' as timeago;

class ConversationsScreen extends StatelessWidget {
  const ConversationsScreen({
    super.key,
    required this.self,
    required this.conversations,
  });
  final String self;
  final UnmodifiableMapView<String, List<MessageModel>> conversations;

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
        itemCount: conversations.length,
        itemBuilder: (context, i) {
          final peer = conversations.keys.elementAt(i);
          return Conversation(
            avatar: CircleAvatar(
                backgroundColor: Colors.primaries[i % Colors.primaries.length],
                radius: 25,
                child: Text(peer.substring(0, 2))),
            lastMessage: conversations.values.elementAt(i).last,
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
    case MessageState.delivered:
      return Icons.check_circle_outline_outlined;
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
    final theme = Theme.of(context);
    final time = DateTime.fromMillisecondsSinceEpoch(
        1000 * lastMessage.dbRecvTime.toInt());

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
            builder: (context) => ChatScreen(
              self: self,
              peer: peer,
            ),
          ),
        );
      },
      onLongPress: null,
      child: SizedBox(
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
                      peer,
                      overflow: TextOverflow.ellipsis,
                      style: theme.textTheme.bodyMedium,
                    ),
                    Text(
                      lastMessage.text,
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
                    timeago.format(time, locale: 'en_short'),
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
