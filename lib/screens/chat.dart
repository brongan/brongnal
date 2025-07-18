import 'package:brongnal_app/common/theme.dart';
import 'package:brongnal_app/common/util.dart';
import 'package:brongnal_app/src/bindings/bindings.dart';
import 'package:brongnal_app/models/chat_history.dart';
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';

enum Sender {
  other,
  self,
}

class ChatScreen extends StatelessWidget {
  const ChatScreen({
    super.key,
    required this.self,
    required this.peer,
  });

  final String self;
  final String peer;

  @override
  Widget build(BuildContext context) {
    var conversationModel = context.watch<ChatHistory>();
    final messages = conversationModel.items[peer] ?? [];
    ScrollController scrollController = ScrollController();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      scrollController.jumpTo(scrollController.position.maxScrollExtent);
    });
    return Scaffold(
      appBar: getConversationAppBar(context, peer),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: ListView.builder(
                itemCount: messages.length,
                controller: scrollController,
                itemBuilder: (context, i) {
                  return MessageWidget(
                      message: messages[i].text,
                      time: DateTime.fromMillisecondsSinceEpoch(
                          1000 * messages[i].dbRecvTime),
                      sender: messages[i].sender == self
                          ? Sender.self
                          : Sender.other,
                      // TODO: only add this icon for the last message in a set from a given sender.
                      state: messages[i].state);
                },
              ),
            ),
            SendMessageWidget(
              self: self,
              peer: peer,
              conversationModel: conversationModel,
            ),
          ],
        ),
      ),
    );
  }
}

class SendMessageWidget extends StatefulWidget {
  const SendMessageWidget({
    super.key,
    required this.self,
    required this.peer,
    required this.conversationModel,
  });
  final String self;
  final String peer;
  final ChatHistory conversationModel;

  @override
  State<SendMessageWidget> createState() => _SendMessageWidgetState();
}

class _SendMessageWidgetState extends State<SendMessageWidget> {
  late FocusNode myFocusNode;

  @override
  void initState() {
    super.initState();
    myFocusNode = FocusNode();
  }

  @override
  void dispose() {
    myFocusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    TextEditingController messageInput = TextEditingController();
    return Row(
      children: [
        Expanded(
          child: Padding(
            padding: const EdgeInsets.all(8.0),
            child: TextField(
              focusNode: myFocusNode,
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
              textInputAction: TextInputAction.send,
              onSubmitted: (value) {
                SendMessage(
                        sender: widget.self,
                        recipient: widget.peer,
                        message: messageInput.text)
                    .sendSignalToRust();
                messageInput.clear();
                myFocusNode.requestFocus();
              },
            ),
          ),
        ),
        const StubIconButton(
            icon: Icons.add_circle, name: "Add an attachment."),
      ],
    );
  }
}

class MessageWidget extends StatelessWidget {
  const MessageWidget({
    super.key,
    required this.message,
    required this.time,
    required this.sender,
    required this.state,
  });
  final String message;
  final DateTime time;
  final Sender sender;
  final MessageState state;

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

    var readIcon = Icon(
      getIcon(state),
      color: textColor,
      size: 18,
    );

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
            ),
            children: [
              const TextSpan(text: ' '),
              TextSpan(
                text: DateFormat.jm().format(time),
                style: theme.textTheme.bodySmall,
              ),
              WidgetSpan(child: readIcon),
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
                  fontSize: 20,
                  color: Colors.white,
                ))),
        Icon(Icons.account_circle, size: 16),
      ],
    ),
    toolbarHeight: theme.toolbarHeight,
    backgroundColor: theme.backgroundColor,
    leadingWidth: 36,
    actions: <Widget>[
      const StubIconButton(icon: Icons.videocam_outlined, name: 'Video Call'),
      const StubIconButton(icon: Icons.phone_outlined, name: 'Call'),
      PopupMenuButton<ConversationPopupItem>(
        onSelected: (ConversationPopupItem item) {},
        iconSize: theme.iconTheme!.size!,
        itemBuilder: (BuildContext context) => [],
      ),
    ],
  );
}
