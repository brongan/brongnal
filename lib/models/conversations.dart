import 'dart:collection';

import 'package:brongnal_app/messages/brongnal.pb.dart';
import 'package:brongnal_app/models/message.dart';
import 'package:flutter/material.dart';

class ConversationModel extends ChangeNotifier {
  final Map<String, List<MessageModel>> _conversations = {};
  UnmodifiableMapView<String, List<MessageModel>> get items =>
      UnmodifiableMapView(_conversations);

  void add(ReceivedMessage message) {
    _conversations.putIfAbsent(message.sender, () => []);
    _conversations[message.sender]!.add(MessageModel(
        message: message.message,
        sender: message.sender,
        time: DateTime.now(),
        state: MessageState.sent));
    notifyListeners();
  }

  void compose(String peer) {
    _conversations.putIfAbsent(peer, () => []);
  }

  void addSentMessage(String message, String sender, String receiver) {
    _conversations.putIfAbsent(sender, () => []);
    _conversations[receiver]!.add(MessageModel(
        message: message,
        sender: sender,
        time: DateTime.now(),
        state: MessageState.sending));
    notifyListeners();
  }
}
