import 'dart:collection';
import 'package:brongnal_app/src/rust/bridge.dart' as bridge;
import 'package:brongnal_app/src/rust/bridge.dart'
    show MessageModel, MessageState, getAllMessages, subscribeMessages;
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
  final Map<String, List<MessageModel>> _conversations = {};

  ChatHistory({
    required this.username,
    required this.onMessageReceived,
  }) {
    _init();
  }

  Future<void> _init() async {
    // 1. Load history
    try {
      final history = await getAllMessages();
      for (final msg in history) {
        _addLocal(msg, notify: false);
      }
      notifyListeners();
    } catch (e) {
      debugPrint("Failed to load message history: $e");
    }

    // 2. Subscribe to new messages
    subscribeMessages().listen((message) {
      add(message);
    });
  }

  UnmodifiableMapView<String, List<MessageModel>> get items =>
      UnmodifiableMapView(_conversations);

  void _addLocal(MessageModel message, {bool notify = true}) {
    final peer = message.sender == username ? message.receiver : message.sender;
    _conversations.putIfAbsent(peer, () => []);
    _conversations[peer]!.add(message);
    if (notify) notifyListeners();
  }

  void add(MessageModel message) async {
    _addLocal(message);

    final recvTime =
        DateTime.fromMillisecondsSinceEpoch(1000 * message.dbRecvTime.toInt());

    if (message.receiver == username &&
        recvTime.isAfter(DateTime.now().subtract(const Duration(seconds: 1)))) {
      await onMessageReceived(message);
    }
  }

  Future<void> sendMessage(String recipient, String text) async {
    try {
      final msg = await bridge.sendMessage(recipient: recipient, text: text);
      add(msg);
    } catch (e) {
      debugPrint("Failed to send message: $e");
      rethrow;
    }
  }
}
