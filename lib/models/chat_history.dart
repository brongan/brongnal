import 'dart:collection';

import 'package:brongnal_app/src/bindings/bindings.dart';
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
  final Map<String, List<MessageModel>> conversations = {};
  ChatHistory({
    required this.username,
    required this.onMessageReceived,
  });
  UnmodifiableMapView<String, List<MessageModel>> get items =>
      UnmodifiableMapView(conversations);

  void add(MessageModel message) async {
    final peer = message.sender == username ? message.receiver : message.sender;
    conversations.putIfAbsent(peer, () => []);
    conversations[peer]!.add(message);
    notifyListeners();
    final recvTime =
        DateTime.fromMillisecondsSinceEpoch(1000 * message.dbRecvTime.toInt());

    if (message.receiver == username &&
        recvTime.isAfter(DateTime.now().subtract(Duration(seconds: 1)))) {
      await onMessageReceived(message);
    }
  }
}
