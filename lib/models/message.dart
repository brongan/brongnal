enum MessageState {
  sending,
  sent,
  read,
}

class MessageModel {
  const MessageModel({
    required this.message,
    required this.time,
    required this.sender,
    required this.state,
  });
  final String message;
  final DateTime time;
  final String sender;
  final MessageState state;
}
