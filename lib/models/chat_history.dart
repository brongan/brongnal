import 'dart:collection';
import 'package:brongnal_app/src/bindings/bindings.dart';
import 'package:flutter/material.dart';

class ChatHistory extends ChangeNotifier {
  ChatHistory({
    required this.username,
  });
  final String username;
  final Map<String, List<MessageModel>> conversations = {};
  UnmodifiableMapView<String, List<MessageModel>> get items =>
      UnmodifiableMapView(conversations);

  void add(MessageModel message) async {
    final peer = message.sender == username ? message.receiver : message.sender;
    conversations.putIfAbsent(peer, () => []);
    conversations[peer]!.add(message);
    notifyListeners();
  }
}
