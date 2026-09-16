import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:tsp_driver_dart/driver.dart';

final _fixture =
    jsonDecode(File('../../fixtures/spec-vectors.json').readAsStringSync())
        as Map<String, Object?>;
final _ids = (_fixture['identifiers']! as Map<String, Object?>)
    .cast<String, Map<String, Object?>>();
final _vectors = (_fixture['vectors']! as Map<String, Object?>)
    .cast<String, Map<String, Object?>>();

Map<String, Object?> _private(String n) => {..._ids[n]!}..remove('longForm');
Map<String, Object?> _public(String n) => _private(n)
  ..remove('skS')
  ..remove('skE');

void main() {
  test('stdin/stdout: hello and open direct-hpke-base', () async {
    final proc = await Process.start(Platform.resolvedExecutable, [
      'run',
      'bin/tsp_driver.dart',
    ]);
    final out = proc.stdout
        .transform(utf8.decoder)
        .transform(const LineSplitter());
    final lines = StreamIterator(out);
    final v = _vectors['direct-hpke-base']!;
    proc.stdin
      ..writeln(jsonEncode({'id': 1, 'op': 'hello'}))
      ..writeln(
        jsonEncode({
          'id': 2,
          'op': 'open',
          'receiver': _private('bob'),
          'sender': _public('alice'),
          'message': v['message'],
        }),
      );
    await proc.stdin.flush();

    expect(await lines.moveNext(), isTrue);
    final hello = jsonDecode(lines.current) as Map<String, Object?>;
    expect(hello['id'], 1);
    expect(hello['ok'], isTrue);
    final result = hello['result']! as Map<String, Object?>;
    expect(result['protocol'], 1);
    expect(result['language'], 'dart');
    expect(result['capabilities'], contains('hpke-base'));

    expect(await lines.moveNext(), isTrue);
    final open = jsonDecode(lines.current) as Map<String, Object?>;
    expect(open['id'], 2);
    expect(open['ok'], isTrue, reason: '$open');
    final opened = open['result']! as Map<String, Object?>;
    expect(opened['scheme'], 'hpke-base');
    expect(opened['version'], {'major': 0, 'minor': 2});
    final payload = opened['payload']! as Map<String, Object?>;
    expect(payload['type'], 'scs');
    expect(utf8.decode(base64Url.decode('${payload['data']}=')), 'hello world');
    expect(payload['payloadSender'], isNull);

    await proc.stdin.close();
    expect(await proc.exitCode, 0);
  });

  group('in-process', () {
    final driver = Driver();
    Future<Map<String, Object?>> call(Map<String, Object?> req) async {
      final r = await driver.handleLine(jsonEncode({'id': 0, ...req}));
      expect(r['ok'], isTrue, reason: '$req -> $r');
      return r['result']! as Map<String, Object?>;
    }

    test('every vector opens, and re-packs byte-exact where it can', () async {
      for (final entry in _vectors.entries) {
        final v = entry.value;
        final opened = await call({
          'op': 'open',
          'receiver': _private(v['receiver']! as String),
          'sender': _public(v['sender']! as String),
          'message': v['message'],
        });
        if (entry.key == 'direct-hpke-base-pq') {
          expect(opened['scheme'], 'hpke-pq');
          continue;
        }
        final payload = Map<String, Object?>.of(
          opened['payload']! as Map<String, Object?>,
        );
        final ephemeral = v['ikmE'] != null
            ? {'ikmE': v['ikmE']}
            : v['skEm'] != null
            ? {'skEm': v['skEm']}
            : null;
        final packed = await call({
          'op': 'pack',
          'scheme': opened['scheme'],
          'sender': _private(v['sender']! as String),
          'receiver': _public(v['receiver']! as String),
          'payload': payload,
          'ephemeral': ephemeral,
        });
        expect(packed['message'], v['message'], reason: entry.key);
      }
    });

    test('a tampered message is rejected with a code', () async {
      final v = _vectors['direct-hpke-base']!;
      final msg = v['message']! as String;
      final tampered =
          '${msg.substring(0, 200)}${msg[200] == 'A' ? 'B' : 'A'}${msg.substring(201)}';
      final r = await driver.handleLine(
        jsonEncode({
          'id': 9,
          'op': 'open',
          'receiver': _private('bob'),
          'sender': _public('alice'),
          'message': tampered,
        }),
      );
      expect(r['ok'], isFalse);
      expect((r['error']! as Map<String, Object?>)['code'], isNotNull);
    });

    test('endpoint relationship round trip', () async {
      final a = await call({
        'op': 'endpoint.create',
        'identities': [_private('alice')],
        'peers': [_public('bob')],
      });
      final b = await call({
        'op': 'endpoint.create',
        'identities': [_private('bob')],
        'peers': [_public('alice')],
      });
      final alice = _ids['alice']!['id'];
      final bob = _ids['bob']!['id'];
      final inv = await call({
        'op': 'endpoint.invite',
        'endpoint': a['endpoint'],
        'from': alice,
        'to': bob,
      });
      final got = await call({
        'op': 'endpoint.receive',
        'endpoint': b['endpoint'],
        'message': inv['message'],
      });
      expect(got['event'], 'invite');
      expect(got['digest'], inv['digest']);
      final acc = await call({
        'op': 'endpoint.accept',
        'endpoint': b['endpoint'],
        'from': bob,
        'to': alice,
      });
      expect(acc['digest'], inv['digest']);
      final got2 = await call({
        'op': 'endpoint.receive',
        'endpoint': a['endpoint'],
        'message': acc['message'],
      });
      expect(got2['event'], 'accept');
      expect(got2['replyDigest'], acc['replyDigest']);
      for (final (ep, l, r) in [(a, alice, bob), (b, bob, alice)]) {
        final s = await call({
          'op': 'endpoint.state',
          'endpoint': ep['endpoint'],
          'local': l,
          'remote': r,
        });
        expect(s, {
          'state': 'bidirectional',
          'digest': inv['digest'],
          'replyDigest': acc['replyDigest'],
        });
      }
    });

    test('unknown op is unsupported', () async {
      final r = await driver.handleLine('{"id": 3, "op": "frobnicate"}');
      expect((r['error']! as Map<String, Object?>)['code'], 'unsupported');
    });
  });
}
