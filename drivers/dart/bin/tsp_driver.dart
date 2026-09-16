import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:tsp_driver_dart/driver.dart';

/// Speaks the TSP conformance driver protocol v1 on stdin/stdout: one JSON
/// request per line in, exactly one JSON response per line out, in order.
Future<void> main() async {
  final driver = Driver();
  final lines = stdin.transform(utf8.decoder).transform(const LineSplitter());
  await for (final line in lines) {
    if (line.trim().isEmpty) continue;
    final response = await driver.handleLine(line);
    stdout.writeln(jsonEncode(response));
    await stdout.flush();
  }
  exit(0);
}
