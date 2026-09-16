/// The request handler of the Dart TSP conformance driver.
///
/// A thin adapter: every byte it returns comes from `affinidi_tsp`.
library;

import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:affinidi_tsp/affinidi_tsp.dart';
import 'package:affinidi_tsp_pq/affinidi_tsp_pq.dart';

/// Capabilities advertised in `hello`. Every entry is exercised by the
/// library's own test suite against the Rev 3 vectors.
const List<String> capabilities = [
  'hpke-base',
  'signed-only',
  'sealed-box',
  'hpke-pq',
  'payload.scs',
  'payload.ctl',
  'payload.pad',
  'payload.rfi',
  'payload.rfa',
  'payload.rfd',
  'rfi.reply-path',
  'rfi.referral',
  'payload.hop',
  'padding',
  'payload-sender',
  'deterministic',
  'peek',
  'endpoint',
];

/// A request the driver refuses before reaching the library.
final class _Refusal implements Exception {
  _Refusal(this.code, this.message);
  final String code;
  final String message;
}

_Refusal _invalid(String m) => _Refusal('invalid-input', m);
_Refusal _unsupported(String m) => _Refusal('unsupported', m);

String _b64(List<int> bytes) => base64Url.encode(bytes).replaceAll('=', '');

Uint8List _unb64(Object? v, String field) {
  if (v is! String) throw _invalid('$field must be a base64url string');
  try {
    final s = v.replaceAll('=', '');
    if (s.length % 4 == 1) throw const FormatException('bad length');
    return base64Url.decode(s.padRight(s.length + (4 - s.length % 4) % 4, '='));
  } on FormatException {
    throw _invalid('$field is not valid base64url');
  }
}

Map<String, Object?> _obj(Object? v, String field) {
  if (v is Map<String, Object?>) return v;
  throw _invalid('$field must be an object');
}

String _str(Object? v, String field) {
  if (v is String) return v;
  throw _invalid('$field must be a string');
}

List<String> _strList(Object? v, String field) {
  if (v == null) return const [];
  if (v is List && v.every((e) => e is String)) return v.cast<String>();
  throw _invalid('$field must be an array of strings');
}

final class _SigningUnavailable implements TspSigningKey {
  const _SigningUnavailable(this.algorithm);
  @override
  final TspSignatureAlgorithm algorithm;
  @override
  Future<Uint8List> sign(Uint8List message) =>
      throw _invalid('identity has no skS');
}

/// Parses a driver-protocol Identity.
final class _Identity {
  _Identity(Map<String, Object?> json)
    : id = _str(json['id'], 'identity.id'),
      sigKeyType = _str(json['sigKeyType'], 'identity.sigKeyType'),
      encKeyType = _str(json['encKeyType'], 'identity.encKeyType'),
      _json = json;

  final String id;
  final String sigKeyType;
  final String encKeyType;
  final Map<String, Object?> _json;

  bool get isPostQuantumKem => encKeyType == TspKem.mlKem768X25519.wireName;

  TspVerificationKey verificationKey() {
    final pk = _unb64(_json['pkS'], 'identity.pkS');
    return switch (sigKeyType) {
      'Ed25519' => Ed25519VerificationKey(pk),
      'MlDsa65' => MlDsa65VerificationKey(pk),
      _ => throw _unsupported('sigKeyType $sigKeyType'),
    };
  }

  TspEncryptionKey encryptionKey() {
    final pk = _unb64(_json['pkE'], 'identity.pkE');
    return switch (encKeyType) {
      'X25519' => X25519EncryptionKey(pk),
      'MLKEM768-X25519' => MlKem768X25519EncryptionKey(pk),
      _ => throw _unsupported('encKeyType $encKeyType'),
    };
  }

  TspSigningKey signingKey() {
    final algorithm = switch (sigKeyType) {
      'Ed25519' => TspSignatureAlgorithm.ed25519,
      'MlDsa65' => TspSignatureAlgorithm.mlDsa65,
      _ => throw _unsupported('sigKeyType $sigKeyType'),
    };
    if (_json['skS'] == null) return _SigningUnavailable(algorithm);
    final sk = _unb64(_json['skS'], 'identity.skS');
    return switch (algorithm) {
      TspSignatureAlgorithm.ed25519 => Ed25519SigningKey.fromSeed(sk),
      TspSignatureAlgorithm.mlDsa65 => MlDsa65SigningKey(sk),
    };
  }

  TspDecryptionKey? decryptionKey() {
    if (_json['skE'] == null) return null;
    final sk = _unb64(_json['skE'], 'identity.skE');
    return switch (encKeyType) {
      'X25519' => X25519DecryptionKey.fromSecret(sk),
      'MLKEM768-X25519' => MlKem768X25519DecryptionKey.fromSeed(sk),
      _ => throw _unsupported('encKeyType $encKeyType'),
    };
  }

  PublicVid publicVid() => PublicVid(
    id: id,
    verificationKey: verificationKey(),
    encryptionKey: encryptionKey(),
  );

  PrivateVid privateVid() => PrivateVid(
    id: id,
    signingKey: signingKey(),
    decryptionKey: decryptionKey(),
  );
}

TspDigest _digestIn(
  Map<String, Object?> p,
  String field,
  TspDigestAlgorithm fallback,
) {
  final algName = p['digestAlg'];
  var alg = fallback;
  if (algName != null) {
    alg =
        TspDigestAlgorithm.byWireName(_str(algName, 'payload.digestAlg')) ??
        (throw _unsupported('digestAlg $algName'));
  }
  final bytes = _unb64(p[field], 'payload.$field');
  if (bytes.length != 32) throw _invalid('payload.$field must be 32 bytes');
  return TspDigest(bytes, alg);
}

TspPayload _payloadIn(Map<String, Object?> p, TspScheme scheme) {
  final padding = p['padding'] == null
      ? Uint8List(0)
      : _unb64(p['padding'], 'payload.padding');
  final schemeDigest = scheme == TspScheme.sealedBox
      ? TspDigestAlgorithm.blake2b256
      : TspDigestAlgorithm.sha256;
  final type = _str(p['type'], 'payload.type');
  Uint8List? nonce() =>
      p['nonce'] == null ? null : _unb64(p['nonce'], 'payload.nonce');
  switch (type) {
    case 'scs':
      return ScsPayload(_unb64(p['data'], 'payload.data'), padding: padding);
    case 'ctl':
      return CtlPayload(_unb64(p['data'], 'payload.data'), padding: padding);
    case 'pad':
      return PadPayload(nonce: nonce(), padding: padding);
    case 'rfi':
      final ref = p['referral'];
      Referral? referral;
      if (ref != null) {
        final r = _obj(ref, 'payload.referral');
        final vid = _str(r['vid'], 'referral.vid');
        if (r['skS'] != null) {
          // The runner hands over VID_new's private key so the library makes
          // Signature_new at pack time (it covers the invite digest).
          final sk = _unb64(r['skS'], 'referral.skS');
          referral = Referral.signWith(
            vid: vid,
            signingKey: sk.length == 32
                ? Ed25519SigningKey.fromSeed(sk)
                : MlDsa65SigningKey(sk),
          );
        } else {
          referral = Referral(
            vid: vid,
            signature: _unb64(r['signature'], 'referral.signature'),
          );
        }
      }
      return RfiPayload(
        nonce: nonce(),
        replyPath: _strList(p['replyPath'], 'payload.replyPath'),
        referral: referral,
        padding: padding,
      );
    case 'rfa':
      return RfaPayload(
        digest: _digestIn(p, 'digest', schemeDigest),
        padding: padding,
      );
    case 'rfd':
      return RfdPayload(
        digest: _digestIn(p, 'digest', TspDigestAlgorithm.sha256),
        padding: padding,
      );
    case 'hop':
      return HopPayload(
        hops: _strList(p['hops'], 'payload.hops'),
        inner: _unb64(p['inner'], 'payload.inner'),
        padding: padding,
      );
    default:
      throw _unsupported('payload type $type');
  }
}

Map<String, Object?> _payloadOut(TspMessage m) {
  final p = m.payload;
  final out = <String, Object?>{
    'padding': _b64(p.padding),
    'payloadSender': m.payloadSender,
  };
  switch (p) {
    case ScsPayload() || CtlPayload():
      final data = (p as StreamPayload).data;
      if (data == null) {
        throw _unsupported(
          'the payload stream is not a single Bytes primitive',
        );
      }
      out['type'] = p is ScsPayload ? 'scs' : 'ctl';
      out['data'] = _b64(data);
    case PadPayload(:final nonce):
      out['type'] = 'pad';
      out['nonce'] = _b64(nonce!);
    case RfiPayload(
      :final nonce,
      :final replyPath,
      :final referral,
      :final digest,
    ):
      out['type'] = 'rfi';
      out['nonce'] = _b64(nonce!);
      out['replyPath'] = replyPath;
      out['referral'] = referral == null
          ? null
          : {'vid': referral.vid, 'signature': _b64(referral.signature!)};
      out['digest'] = _b64(digest!.bytes);
      out['digestAlg'] = digest.algorithm.wireName;
    case RfaPayload(:final digest, :final replyDigest):
      out['type'] = 'rfa';
      out['digest'] = _b64(digest.bytes);
      out['replyDigest'] = _b64(replyDigest!.bytes);
      out['digestAlg'] = replyDigest.algorithm.wireName;
    case RfdPayload(:final digest):
      out['type'] = 'rfd';
      out['digest'] = _b64(digest.bytes);
      out['digestAlg'] = digest.algorithm.wireName;
    case HopPayload(:final hops, :final inner):
      out['type'] = 'hop';
      out['hops'] = hops;
      out['inner'] = _b64(inner);
  }
  return out;
}

Map<String, Object?> _version(TspVersion v) => {
  'major': v.major,
  'minor': v.minor,
};

String _schemeOut(TspScheme scheme, TspKem? kem) =>
    kem == TspKem.mlKem768X25519 ? 'hpke-pq' : scheme.wireName;

/// Handles driver-protocol requests.
final class Driver {
  final Map<String, TspEndpoint> _endpoints = {};
  var _nextEndpoint = 0;

  /// Handles one request line, always returning a response object.
  Future<Map<String, Object?>> handleLine(String line) async {
    Object? id;
    try {
      final decoded = jsonDecode(line);
      if (decoded is! Map<String, Object?>) {
        throw _invalid('request must be an object');
      }
      id = decoded['id'];
      final result = await handle(decoded);
      return {'id': id, 'ok': true, 'result': result};
    } on _Refusal catch (e) {
      return _error(id, e.code, e.message);
    } on TspException catch (e) {
      return _error(id, e.code.wireName, e.message);
    } on FormatException catch (e) {
      return _error(id, 'invalid-input', e.message);
    } on Object catch (e, st) {
      stderr.writeln('internal error: $e\n$st');
      return _error(id, 'internal', e.toString());
    }
  }

  Map<String, Object?> _error(Object? id, String code, String message) => {
    'id': id,
    'ok': false,
    'error': {'code': code, 'message': message},
  };

  /// Dispatches a decoded request.
  Future<Map<String, Object?>> handle(Map<String, Object?> req) async {
    final op = _str(req['op'], 'op');
    return switch (op) {
      'hello' => {
        'name': 'affinidi-tsp-dart',
        'version': '0.1.0',
        'language': 'dart',
        'protocol': 1,
        'capabilities': capabilities,
      },
      'pack' => _pack(req),
      'open' => _open(req),
      'peek' => _peek(req),
      'endpoint.create' => _endpointCreate(req),
      'endpoint.invite' => _endpointInvite(req),
      'endpoint.accept' => _endpointAccept(req),
      'endpoint.cancel' => _endpointCancel(req),
      'endpoint.send' => _endpointSend(req),
      'endpoint.receive' => _endpointReceive(req),
      'endpoint.state' => _endpointState(req),
      _ => throw _unsupported('op $op'),
    };
  }

  Future<Map<String, Object?>> _pack(Map<String, Object?> req) async {
    final schemeName = _str(req['scheme'], 'scheme');
    final sender = _Identity(_obj(req['sender'], 'sender'));
    final receiver = _Identity(_obj(req['receiver'], 'receiver'));
    final TspScheme scheme;
    switch (schemeName) {
      case 'hpke-base':
      case 'hpke-pq':
        scheme = TspScheme.hpkeBase;
        if (receiver.isPostQuantumKem != (schemeName == 'hpke-pq')) {
          throw _invalid(
            "scheme $schemeName does not match the receiver's encKeyType",
          );
        }
      case 'sealed-box':
        scheme = TspScheme.sealedBox;
      case 'signed-only':
        scheme = TspScheme.signedOnly;
      default:
        throw _unsupported('scheme $schemeName');
    }
    final payloadJson = _obj(req['payload'], 'payload');
    final payload = _payloadIn(payloadJson, scheme);

    var senderMode = PayloadSenderMode.schemeDefault;
    if (payloadJson.containsKey('payloadSender')) {
      final ps = payloadJson['payloadSender'];
      if (ps == null) {
        senderMode = PayloadSenderMode.nullVid;
      } else if (ps == sender.id) {
        senderMode = PayloadSenderMode.present;
      } else {
        throw _unsupported(
          'payloadSender other than the envelope sender or null',
        );
      }
    }

    Uint8List? ephemeral;
    final eph = req['ephemeral'];
    if (eph != null) {
      final e = _obj(eph, 'ephemeral');
      switch (scheme) {
        case TspScheme.hpkeBase:
          if (e['ikmE'] == null) {
            throw _invalid('HPKE packing takes ephemeral.ikmE');
          }
          ephemeral = _unb64(e['ikmE'], 'ephemeral.ikmE');
        case TspScheme.sealedBox:
          if (e['skEm'] == null) {
            throw _invalid('sealed-box packing takes ephemeral.skEm');
          }
          ephemeral = _unb64(e['skEm'], 'ephemeral.skEm');
        case TspScheme.signedOnly:
          break;
      }
    }

    final packed = await Tsp.pack(
      sender: sender.privateVid(),
      receiver: receiver.publicVid(),
      payload: payload,
      scheme: scheme,
      options: TspPackOptions(payloadSender: senderMode, ephemeral: ephemeral),
    );
    return {
      'message': _b64(packed.bytes),
      'digest': packed.digest == null ? null : _b64(packed.digest!.bytes),
      'digestAlg': packed.digest?.algorithm.wireName,
    };
  }

  Future<Map<String, Object?>> _open(Map<String, Object?> req) async {
    final receiver = _Identity(_obj(req['receiver'], 'receiver'));
    final sender = _Identity(_obj(req['sender'], 'sender'));
    final bytes = _unb64(req['message'], 'message');
    final m = await Tsp.open(
      bytes,
      receiver: receiver.privateVid(),
      sender: sender.publicVid(),
    );
    return {
      'version': _version(m.version),
      'envelopeSender': m.sender,
      'envelopeReceiver': m.receiver,
      'scheme': _schemeOut(m.scheme, m.kem),
      'payload': _payloadOut(m),
    };
  }

  Map<String, Object?> _peek(Map<String, Object?> req) {
    final info = Tsp.peek(_unb64(req['message'], 'message'));
    return {
      'version': _version(info.version),
      'envelopeSender': info.sender,
      'envelopeReceiver': info.receiver,
      'confidential': info.confidential,
    };
  }

  TspEndpoint _endpoint(Map<String, Object?> req) {
    final handle = _str(req['endpoint'], 'endpoint');
    return _endpoints[handle] ?? (throw _invalid('unknown endpoint $handle'));
  }

  Map<String, Object?> _endpointCreate(Map<String, Object?> req) {
    List<Map<String, Object?>> list(String f) {
      final v = req[f];
      if (v is! List) throw _invalid('$f must be an array');
      return [for (final e in v) _obj(e, f)];
    }

    final identities = [
      for (final j in list('identities')) _Identity(j).privateVid(),
    ];
    final peers = [for (final j in list('peers')) _Identity(j).publicVid()];
    final handle = 'ep-${_nextEndpoint++}';
    _endpoints[handle] = TspEndpoint(
      identities: identities,
      resolver: StaticVidResolver(peers),
    );
    return {'endpoint': handle};
  }

  Future<Map<String, Object?>> _endpointInvite(Map<String, Object?> req) async {
    final packed = await _endpoint(
      req,
    ).invite(from: _str(req['from'], 'from'), to: _str(req['to'], 'to'));
    return {
      'message': _b64(packed.bytes),
      'digest': _b64(packed.digest!.bytes),
    };
  }

  Future<Map<String, Object?>> _endpointAccept(Map<String, Object?> req) async {
    final ep = _endpoint(req);
    final from = _str(req['from'], 'from');
    final to = _str(req['to'], 'to');
    final packed = await ep.accept(from: from, to: to);
    final rel = await ep.relationship(from, to);
    return {
      'message': _b64(packed.bytes),
      'digest': _b64(rel.digest!.bytes),
      'replyDigest': _b64(packed.digest!.bytes),
    };
  }

  Future<Map<String, Object?>> _endpointCancel(Map<String, Object?> req) async {
    final packed = await _endpoint(
      req,
    ).cancel(from: _str(req['from'], 'from'), to: _str(req['to'], 'to'));
    return {'message': _b64(packed.bytes)};
  }

  Future<Map<String, Object?>> _endpointSend(Map<String, Object?> req) async {
    final packed = await _endpoint(req).send(
      from: _str(req['from'], 'from'),
      to: _str(req['to'], 'to'),
      data: _unb64(req['data'], 'data'),
    );
    return {'message': _b64(packed.bytes)};
  }

  Future<Map<String, Object?>> _endpointReceive(
    Map<String, Object?> req,
  ) async {
    final ev = await _endpoint(req).receive(_unb64(req['message'], 'message'));
    final out = <String, Object?>{
      'event': ev.kind.wireName,
      'from': ev.from,
      'to': ev.to,
    };
    switch (ev.message.payload) {
      case RfiPayload(:final digest):
        out['digest'] = _b64(digest!.bytes);
      case RfaPayload(:final digest, :final replyDigest):
        out['digest'] = _b64(digest.bytes);
        out['replyDigest'] = _b64(replyDigest!.bytes);
      case RfdPayload(:final digest):
        out['digest'] = _b64(digest.bytes);
      default:
        if (ev.data != null) out['data'] = _b64(ev.data!);
    }
    return out;
  }

  Future<Map<String, Object?>> _endpointState(Map<String, Object?> req) async {
    final rel = await _endpoint(
      req,
    ).relationship(_str(req['local'], 'local'), _str(req['remote'], 'remote'));
    return {
      'state': rel.state.wireName,
      if (rel.digest != null) 'digest': _b64(rel.digest!.bytes),
      if (rel.replyDigest != null) 'replyDigest': _b64(rel.replyDigest!.bytes),
    };
  }
}
