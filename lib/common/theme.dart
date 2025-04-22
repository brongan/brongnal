import 'package:flutter/material.dart';

const Color textColor = Color.fromRGBO(190, 192, 197, 1.0);
const backgroundColor = Color.fromRGBO(26, 28, 32, 1.0);
const double appbarIconThemeSize = 32;

const conversationNameStyle = TextStyle(
  fontStyle: FontStyle.normal,
  fontWeight: FontWeight.w400,
  fontFamily: 'Roboto',
  fontSize: 20,
  color: textColor,
);
const conversationMessageStyle = TextStyle(
  height: 1.15,
  fontStyle: FontStyle.normal,
  fontWeight: FontWeight.w300,
  fontFamily: 'Roboto',
  fontSize: 16,
  color: textColor,
);

ThemeData bronganlDarkTheme = ThemeData(
  useMaterial3: true,
  colorScheme: ColorScheme.fromSeed(
    seedColor: Colors.deepOrange,
    brightness: Brightness.dark,
  ).copyWith(surface: backgroundColor),
  dialogTheme: DialogThemeData(backgroundColor: backgroundColor),
  drawerTheme: const DrawerThemeData(backgroundColor: backgroundColor),
  appBarTheme: const AppBarTheme(
      backgroundColor: backgroundColor,
      iconTheme: IconThemeData(color: textColor, size: appbarIconThemeSize),
      toolbarHeight: 66,
      titleTextStyle: TextStyle(
        color: Color.fromRGBO(255, 255, 255, .95),
        fontFamily: 'Roboto',
        fontSize: 24,
        fontWeight: FontWeight.w300,
      )),
  bottomNavigationBarTheme: const BottomNavigationBarThemeData(
    backgroundColor: backgroundColor,
    selectedLabelStyle: conversationNameStyle,
    unselectedLabelStyle: conversationNameStyle,
  ),
  navigationBarTheme: const NavigationBarThemeData(
    backgroundColor: Color.fromRGBO(40, 43, 48, 1.0),
    indicatorColor: Color.fromRGBO(70, 75, 92, 1.0),
    labelTextStyle: WidgetStatePropertyAll(conversationMessageStyle),
  ),
  iconTheme: const IconThemeData(color: textColor, size: appbarIconThemeSize),
  textTheme: const TextTheme(
    bodyMedium: conversationNameStyle,
    bodySmall: conversationMessageStyle,
  ),
);
