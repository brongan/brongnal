import 'package:flutter/material.dart';
import 'theme.dart';

class StubIconButton extends StatelessWidget {
  final IconData icon;
  final String name;
  const StubIconButton({
    super.key,
    required this.icon,
    required this.name,
  });

  @override
  Widget build(BuildContext context) {
    return IconButton(
      icon: Icon(
        icon,
        color: textColor,
        size: appbarIconThemeSize,
      ),
      tooltip: name,
      onPressed: () {
        final messenger = ScaffoldMessenger.of(context);
        messenger.removeCurrentSnackBar();
        messenger.showSnackBar(SnackBar(content: Text(name)));
      },
    );
  }
}
