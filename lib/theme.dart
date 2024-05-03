import 'package:flutter/material.dart';

const Color textColor = Color.fromRGBO(190, 192, 197, 1.0);

const conversationNameStyle = TextStyle(
  fontStyle: FontStyle.normal,
  fontWeight: FontWeight.w500,
  fontFamily: 'Roboto',
  fontSize: 17,
  color: textColor,
);
const conversationMessageStyle = TextStyle(
  height: 1.15,
  fontStyle: FontStyle.normal,
  fontWeight: FontWeight.w400,
  fontFamily: 'Roboto',
  fontSize: 14,
  color: textColor,
);

ThemeData bronganlDarkTheme = ThemeData(
  useMaterial3: true,
  colorScheme: ColorScheme.fromSeed(
    seedColor: Colors.deepOrange,
    brightness: Brightness.dark,
  ).copyWith(background: const Color.fromRGBO(26, 28, 32, 1.0)),
  navigationBarTheme: const NavigationBarThemeData(
    backgroundColor: Color.fromRGBO(40, 43, 48, 1.0),
  ),
  iconTheme: const IconThemeData(color: textColor),
  textTheme: const TextTheme(
    displayLarge: TextStyle(
      color: textColor,
      fontFamily: 'Roboto',
      fontSize: 72,
      fontWeight: FontWeight.bold,
    ),
    titleLarge: TextStyle(
      color: textColor,
      fontFamily: 'Roboto',
      fontSize: 30,
    ),
    bodyMedium: conversationNameStyle,
    bodySmall: conversationMessageStyle,
  ),
);
