import 'package:flutter/material.dart';
import 'util.dart';
import 'theme.dart';

class ConversationPage extends StatelessWidget {
  const ConversationPage({
    super.key,
    required this.name,
    required this.lastMessage,
  });

  final String name;
  final String lastMessage;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Scaffold(
      appBar: getConversationAppBar(context, name),
      body: Center(
        child: Text(
          lastMessage,
          style: theme.textTheme.bodySmall,
        ),
      ),
    );
  }
}

enum ConversationPopupItem { search }

AppBar getConversationAppBar(BuildContext context, String name) {
  final theme = Theme.of(context);
  return AppBar(
    title: Row(
      children: [
        CircleAvatar(
          backgroundColor: randomColor(),
          child: Text(name.substring(0, 2)),
        ),
        Padding(
          padding: const EdgeInsets.all(16.0),
          child: Text(name),
        ),
        const Icon(
          Icons.account_circle,
        ),
      ],
    ),
    foregroundColor: textColor,
    backgroundColor: theme.colorScheme.background,
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
