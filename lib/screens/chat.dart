import 'dart:math' as math;
import 'package:brongnal_app/common/util.dart';
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

enum Sender {
  other,
  self,
}

class Chat extends StatelessWidget {
  const Chat({
    super.key,
    required this.name,
    required this.lastMessage,
  });

  final String name;
  final String lastMessage;

  @override
  Widget build(BuildContext context) {
    final random = math.Random();
    return Scaffold(
      appBar: getConversationAppBar(context, name),
      body: Column(
        children: [
          Expanded(
            child: ListView.builder(
              itemBuilder: (context, index) {
                final start = random.nextInt(lastMessage.length);
                final end = random.nextInt(lastMessage.length - start) + start;
                return Message(
                  message: lastMessage.substring(start, end),
                  time: DateTime.now(),
                  sender: random.nextBool() ? Sender.other : Sender.self,
                );
              },
            ),
          ),
          const SendMessageWidget()
        ],
      ),
    );
  }
}

class SendMessageWidget extends StatelessWidget {
  const SendMessageWidget({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    TextEditingController messageInput = TextEditingController();
    return Row(
      children: [
        Expanded(
          child: Padding(
            padding: const EdgeInsets.all(8.0),
            child: TextField(
              controller: messageInput,
              decoration: InputDecoration(
                border:
                    OutlineInputBorder(borderRadius: BorderRadius.circular(20)),
                hintText: "Brongnal message",
                suffixIcon: const StubIconButton(
                  icon: Icons.photo_camera_outlined,
                  name: "Send a picture.",
                ),
              ),
            ),
          ),
        ),
        const StubIconButton(
            icon: Icons.add_circle, name: "Add an attachment."),
      ],
    );
  }
}

class Message extends StatelessWidget {
  const Message({
    super.key,
    required this.message,
    required this.time,
    required this.sender,
  });
  final String message;
  final DateTime time;
  final Sender sender;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final double leftPadding;
    final double rightPadding;
    final Color bubbleColor;
    final Alignment alignment;
    if (sender == Sender.self) {
      leftPadding = 36;
      rightPadding = 14;
      bubbleColor = Colors.indigoAccent.shade400;
      alignment = Alignment.centerRight;
    } else {
      leftPadding = 14;
      rightPadding = 36;
      bubbleColor = Colors.grey.shade800;
      alignment = Alignment.centerLeft;
    }
    return Align(
      alignment: alignment,
      child: Container(
        margin: EdgeInsets.only(
            left: leftPadding, right: rightPadding, top: 10, bottom: 10),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(20),
          color: bubbleColor,
        ),
        padding: const EdgeInsets.all(16),
        child: RichText(
          textAlign: TextAlign.left,
          text: TextSpan(
            text: message,
            style: theme.textTheme.bodySmall!.copyWith(
              color: Colors.white,
              fontSize: 24,
            ),
            children: [
              const TextSpan(text: ' '),
              TextSpan(
                text: DateFormat.jm().format(time),
                style: theme.textTheme.bodySmall,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

enum ConversationPopupItem { search }

AppBar getConversationAppBar(BuildContext context, String name) {
  final theme = AppBarTheme.of(context);
  return AppBar(
    title: Row(
      children: [
        Padding(
          padding: const EdgeInsets.only(right: 16.0),
          child: CircleAvatar(
            backgroundColor: theme.foregroundColor,
            child: Text(name.substring(0, 2)),
          ),
        ),
        Flexible(
            child: Text(name,
                overflow: TextOverflow.fade,
                style: theme.titleTextStyle!.copyWith(
                  fontSize: 30,
                  color: Colors.white,
                ))),
        const Icon(Icons.account_circle, size: 30),
      ],
    ),
    toolbarHeight: theme.toolbarHeight,
    backgroundColor: theme.backgroundColor,
    actions: <Widget>[
      const StubIconButton(icon: Icons.videocam_outlined, name: 'Video Call'),
      const StubIconButton(icon: Icons.phone_outlined, name: 'Call'),
      PopupMenuButton<ConversationPopupItem>(
        onSelected: (ConversationPopupItem item) {},
        iconSize: 36,
        itemBuilder: (BuildContext context) => [],
      ),
    ],
  );
}
