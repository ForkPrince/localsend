import 'package:localsend_isolates/rust/api/relay.dart';
import 'package:localsend_isolates/src/task/server/http_server.dart';
import 'package:refena_flutter/refena_flutter.dart';

final relayProvider = Provider((ref) => RelayService(ref));

/// Wraps the Rust relay client.
/// Only one relay connection can run at a time; it lives in the httpServer
/// isolate next to the server it feeds incoming sessions into.
class RelayService {
  final Ref _ref;

  RsRelayClient? _client;

  RelayService(this._ref);

  bool get running => _client != null;

  /// Starts the relay connection for the running HTTP server and returns the
  /// stream of relay events. Incoming relay sessions are automatically served
  /// by the HTTP server.
  ///
  /// The stream ends when the connection is lost or the relay is stopped.
  Future<Stream<RsRelayEvent>> start({
    required String url,
    required String roomSecret,
    required RsRelayInfo info,
  }) async {
    final server = _ref.read(httpServerProvider).server;
    if (server == null) {
      throw StateError('Server is not running');
    }
    if (_client != null) {
      throw StateError('Relay is already running');
    }

    final client = await server.startRelay(
      url: url,
      roomSecret: roomSecret,
      info: info,
    );
    _client = client;
    return client.listen();
  }

  /// Opens a relay session to the device [targetId] for every connection
  /// accepted on a local TCP listener and returns the local address to dial.
  Future<String> openProxy({required String targetId}) {
    return _requireClient().openProxy(targetId: targetId);
  }

  /// Stops the relay connection. The event stream returned by [start] will end.
  Future<void> stop() async {
    final client = _client;
    _client = null;
    await client?.stop();
  }

  RsRelayClient _requireClient() {
    final client = _client;
    if (client == null) {
      throw StateError('Relay is not running');
    }
    return client;
  }
}
