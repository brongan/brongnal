import 'dart:collection';
import 'package:brongnal_app/common/core.dart';
import 'package:brongnal_app/src/rust/bridge.dart'
    show MessageModel, MessageState;
import 'package:flutter/material.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';

IconData getIcon(MessageState messageState) {
  switch (messageState) {
    case MessageState.sending:
      return Icons.radio_button_unchecked_outlined;
    case MessageState.sent:
      return Icons.check_circle;
    case MessageState.delivered:
      return Icons.check_circle_outline_outlined;
    case MessageState.read:
      return Icons.check_circle_outline_outlined;
  }
}

NotificationDetails toNotification(MessageModel model) {
  const String groupKey = 'com.android.brongnal_app.DecryptedMessage';
  const String groupChannelId = '1';
  const String groupChannelName = 'Chats';
  const String groupChannelDescription = 'Messages.';
  const AndroidNotificationDetails androidNotificationDetails =
      AndroidNotificationDetails(groupChannelId, groupChannelName,
          channelDescription: groupChannelDescription,
          importance: Importance.max,
          priority: Priority.high,
          groupKey: groupKey,
          ticker: 'Chats',
          setAsGroupSummary: false);
  return NotificationDetails(android: androidNotificationDetails);
}

class ChatHistory extends ChangeNotifier {
  final String username;
  final Future<void> Function(MessageModel message) onMessageReceived;
  final BrongnalCore core;
  final Map<String, List<MessageModel>> _conversations = {};

  ChatHistory({
    required this.username,
    required this.onMessageReceived,
    required this.core,
  }) {
    _init();
  }

  Future<void> _init() async {
    // 1. Load history
    try {
      final history = await core.getAllMessages();
      addHistory(history);
    } catch (e) {
      debugPrint("Failed to load message history: $e");
    }

    // 2. Subscribe to new messages
    core.subscribeMessages().listen((message) {
      addMessage(message);
    }, onError: (e) {
      debugPrint("subscribeMessages stream error: $e");
    });
  }

  UnmodifiableMapView<String, List<MessageModel>> get items =>
      UnmodifiableMapView(_conversations);

  void addHistory(List<MessageModel> messages) {
    for (final message in messages) {
      final peer = message.sender == username ? message.receiver : message.sender;
      _conversations.putIfAbsent(peer, () => []);
      _conversations[peer]!.add(message);
    }
    notifyListeners();
  }

  void addMessage(MessageModel message) async {
    final peer = message.sender == username ? message.receiver : message.sender;
    _conversations.putIfAbsent(peer, () => []);
    _conversations[peer]!.add(message);
    notifyListeners();

    if (message.receiver == username) {
      await onMessageReceived(message);
    }
  }

  Future<void> sendMessage(String recipient, String text) async {
    try {
      final msg = await core.sendMessage(recipient: recipient, text: text);
      addMessage(msg);
    } catch (e) {
      debugPrint("Failed to send message: $e");
      rethrow;
    }
  }
}
