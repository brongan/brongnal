import 'package:flutter/material.dart';
import 'dart:math' as math;
import 'theme.dart';

Color randomColor() {
  final random = math.Random();
  return Color.fromRGBO(random.nextInt(256), random.nextInt(256),
      random.nextInt(256), 1.0); // 1.0 for full opacity
}

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
        size: 36,
      ),
      tooltip: name,
      onPressed: () {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text(name)));
      },
    );
  }
}
