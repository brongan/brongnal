import 'package:flutter/material.dart';

/// Runs [start] and shows a spinner until it completes. On success it shows
/// [builder] with the result. On failure it shows [errorBuilder], which can
/// call `retry` to run [start] again.
class StartupGate<T> extends StatefulWidget {
  const StartupGate({
    super.key,
    required this.start,
    required this.builder,
    required this.errorBuilder,
  });
  final Future<T> Function() start;
  final Widget Function(BuildContext context, T value) builder;
  final Widget Function(
    BuildContext context,
    Object error,
    bool retrying,
    VoidCallback retry,
  )
  errorBuilder;

  @override
  State<StartupGate<T>> createState() => _StartupGateState<T>();
}

class _StartupGateState<T> extends State<StartupGate<T>> {
  late Future<T> _future = widget.start();

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<T>(
      future: _future,
      builder: (context, snapshot) {
        if (snapshot.hasError) {
          return widget.errorBuilder(
            context,
            snapshot.error!,
            snapshot.connectionState == ConnectionState.waiting,
            () => setState(() {
              _future = widget.start();
            }),
          );
        }
        if (snapshot.connectionState != ConnectionState.done) {
          return const Center(child: CircularProgressIndicator());
        }
        return widget.builder(context, snapshot.data as T);
      },
    );
  }
}
