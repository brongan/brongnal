import 'package:drift/drift.dart';
import 'dart:io';
import 'package:drift/native.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as p;
import 'package:sqlite3/sqlite3.dart';
import 'package:sqlite3_flutter_libs/sqlite3_flutter_libs.dart';

part 'database.g.dart';

@UseRowClass(MessageModel)
class Messages extends Table {
  IntColumn get id => integer().autoIncrement()();
  TextColumn get sender => text()();
  TextColumn get receiver => text()();
  TextColumn get message => text()();
  DateTimeColumn get time => dateTime()();
  IntColumn get state => intEnum<MessageState>()();

  @override
  Set<Column<Object>>? get primaryKey => {id};
}

class MessageModel {
  const MessageModel({
    required this.id,
    required this.sender,
    required this.receiver,
    required this.message,
    required this.time,
    required this.state,
  });
  final int id;
  final String message;
  final DateTime time;
  final String sender;
  final String receiver;
  final MessageState state;
}

enum MessageState {
  sending,
  sent,
  read,
}

@DriftDatabase(tables: [Messages])
class AppDatabase extends _$AppDatabase {
  AppDatabase() : super(_openConnection());

  @override
  int get schemaVersion => 1;
}

LazyDatabase _openConnection() {
  return LazyDatabase(() async {
    final dbFolder = await getApplicationDocumentsDirectory();
    final file = File(p.join(dbFolder.path, 'chat_history.sqlite'));
    if (Platform.isAndroid) {
      await applyWorkaroundToOpenSqlite3OnOldAndroidVersions();
    }
    final cachebase = (await getTemporaryDirectory()).path;
    sqlite3.tempDirectory = cachebase;
    return NativeDatabase.createInBackground(file);
  });
}
