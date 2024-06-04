import 'package:brongnal_app/models/conversations.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

class ComposeMessage extends StatelessWidget {
  const ComposeMessage({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = AppBarTheme.of(context);
    TextEditingController usernameInput = TextEditingController();
    return Consumer<ConversationModel>(
        builder: (context, conversationModel, child) {
      final items = conversationModel.items;
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
              ListView.builder(itemBuilder: (context, i) {
                return const Text("TODO");
              })
            ],
          ),
        ),
        body: TextField(
          controller: usernameInput,
          decoration: InputDecoration(
            border: OutlineInputBorder(borderRadius: BorderRadius.circular(20)),
            hintText: "Find by username",
          ),
          textInputAction: TextInputAction.send,
          onSubmitted: (value) {
            // TODO Navigator.push conversation screen with replace.
          },
        ),
      );
    });
  }
}
