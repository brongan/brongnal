import 'dart:collection';

import 'package:brongnal_app/database.dart';
import 'package:brongnal_app/src/bindings/bindings.dart';
import 'package:flutter/material.dart';

class ConversationModel extends ChangeNotifier {
  ConversationModel({
    required this.database,
    required this.conversations,
  });
  final AppDatabase database;
  final Map<String, List<MessageModel>> conversations;
  UnmodifiableMapView<String, List<MessageModel>> get items =>
      UnmodifiableMapView(conversations);

  void add(String self, ReceivedMessage message) async {
    final messageModel = await database.into(database.messages).insertReturning(
        MessagesCompanion.insert(
            sender: message.sender,
            receiver: self,
            message: message.message,
            time: DateTime.now(),
            state: MessageState.read));
    conversations.putIfAbsent(message.sender, () => []);
    conversations[message.sender]!.add(messageModel);
    notifyListeners();
  }

  void addSentMessage(String message, String self, String receiver) async {
    final messageModel = await database.into(database.messages).insertReturning(
        MessagesCompanion.insert(
            sender: self,
            receiver: receiver,
            message: message,
            time: DateTime.now(),
            state: MessageState.sending));
    conversations.putIfAbsent(receiver, () => []);
    conversations[receiver]!.add(messageModel);
    notifyListeners();
  }
}
