import 'package:brongnal_app/models/chat_history.dart';
import 'package:brongnal_app/screens/chat.dart';
import 'package:brongnal_app/screens/conversations.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class ComposeMessage extends StatelessWidget {
  const ComposeMessage({super.key, required this.self});
  final String self;

  @override
  Widget build(BuildContext context) {
    final theme = AppBarTheme.of(context);
    TextEditingController usernameInput = TextEditingController();
    return Scaffold(
      appBar: AppBar(
        title: Row(
          children: [
            Text(
              "New message",
              overflow: TextOverflow.fade,
              style: theme.titleTextStyle!.copyWith(
                fontSize: 30,
                color: Colors.white,
              ),
            ),
          ],
        ),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(24.0),
              child: TextField(
                controller: usernameInput,
                decoration: InputDecoration(
                  border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(20)),
                  hintText: "Find by username",
                ),
                textInputAction: TextInputAction.send,
                onSubmitted: (value) {
                  Navigator.push(
                    context,
                    MaterialPageRoute<void>(
                      builder: (context) => ChatScreen(
                        self: self,
                        peer: usernameInput.value.text,
                      ),
                    ),
                  );
                },
              ),
            ),
            Expanded(
              child: Consumer<ChatHistory>(
                builder: (context, conversationModel, child) {
                  return ConversationsScreen(
                    self: self,
                    conversations: conversationModel.items,
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
