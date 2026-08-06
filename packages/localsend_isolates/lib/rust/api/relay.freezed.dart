// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'relay.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$RsRelayEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RsRelayEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RsRelayEvent()';
}


}

/// @nodoc
class $RsRelayEventCopyWith<$Res>  {
$RsRelayEventCopyWith(RsRelayEvent _, $Res Function(RsRelayEvent) __);
}


/// Adds pattern-matching-related methods to [RsRelayEvent].
extension RsRelayEventPatterns on RsRelayEvent {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( RsRelayEvent_Connected value)?  connected,TResult Function( RsRelayEvent_Peer value)?  peer,TResult Function( RsRelayEvent_PeerUpdate value)?  peerUpdate,TResult Function( RsRelayEvent_PeerLeft value)?  peerLeft,TResult Function( RsRelayEvent_Disconnected value)?  disconnected,required TResult orElse(),}){
final _that = this;
switch (_that) {
case RsRelayEvent_Connected() when connected != null:
return connected(_that);case RsRelayEvent_Peer() when peer != null:
return peer(_that);case RsRelayEvent_PeerUpdate() when peerUpdate != null:
return peerUpdate(_that);case RsRelayEvent_PeerLeft() when peerLeft != null:
return peerLeft(_that);case RsRelayEvent_Disconnected() when disconnected != null:
return disconnected(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( RsRelayEvent_Connected value)  connected,required TResult Function( RsRelayEvent_Peer value)  peer,required TResult Function( RsRelayEvent_PeerUpdate value)  peerUpdate,required TResult Function( RsRelayEvent_PeerLeft value)  peerLeft,required TResult Function( RsRelayEvent_Disconnected value)  disconnected,}){
final _that = this;
switch (_that) {
case RsRelayEvent_Connected():
return connected(_that);case RsRelayEvent_Peer():
return peer(_that);case RsRelayEvent_PeerUpdate():
return peerUpdate(_that);case RsRelayEvent_PeerLeft():
return peerLeft(_that);case RsRelayEvent_Disconnected():
return disconnected(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( RsRelayEvent_Connected value)?  connected,TResult? Function( RsRelayEvent_Peer value)?  peer,TResult? Function( RsRelayEvent_PeerUpdate value)?  peerUpdate,TResult? Function( RsRelayEvent_PeerLeft value)?  peerLeft,TResult? Function( RsRelayEvent_Disconnected value)?  disconnected,}){
final _that = this;
switch (_that) {
case RsRelayEvent_Connected() when connected != null:
return connected(_that);case RsRelayEvent_Peer() when peer != null:
return peer(_that);case RsRelayEvent_PeerUpdate() when peerUpdate != null:
return peerUpdate(_that);case RsRelayEvent_PeerLeft() when peerLeft != null:
return peerLeft(_that);case RsRelayEvent_Disconnected() when disconnected != null:
return disconnected(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String clientId)?  connected,TResult Function( RsRelayPeer peer)?  peer,TResult Function( RsRelayPeer peer)?  peerUpdate,TResult Function( String peerId)?  peerLeft,TResult Function( String? error)?  disconnected,required TResult orElse(),}) {final _that = this;
switch (_that) {
case RsRelayEvent_Connected() when connected != null:
return connected(_that.clientId);case RsRelayEvent_Peer() when peer != null:
return peer(_that.peer);case RsRelayEvent_PeerUpdate() when peerUpdate != null:
return peerUpdate(_that.peer);case RsRelayEvent_PeerLeft() when peerLeft != null:
return peerLeft(_that.peerId);case RsRelayEvent_Disconnected() when disconnected != null:
return disconnected(_that.error);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String clientId)  connected,required TResult Function( RsRelayPeer peer)  peer,required TResult Function( RsRelayPeer peer)  peerUpdate,required TResult Function( String peerId)  peerLeft,required TResult Function( String? error)  disconnected,}) {final _that = this;
switch (_that) {
case RsRelayEvent_Connected():
return connected(_that.clientId);case RsRelayEvent_Peer():
return peer(_that.peer);case RsRelayEvent_PeerUpdate():
return peerUpdate(_that.peer);case RsRelayEvent_PeerLeft():
return peerLeft(_that.peerId);case RsRelayEvent_Disconnected():
return disconnected(_that.error);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String clientId)?  connected,TResult? Function( RsRelayPeer peer)?  peer,TResult? Function( RsRelayPeer peer)?  peerUpdate,TResult? Function( String peerId)?  peerLeft,TResult? Function( String? error)?  disconnected,}) {final _that = this;
switch (_that) {
case RsRelayEvent_Connected() when connected != null:
return connected(_that.clientId);case RsRelayEvent_Peer() when peer != null:
return peer(_that.peer);case RsRelayEvent_PeerUpdate() when peerUpdate != null:
return peerUpdate(_that.peer);case RsRelayEvent_PeerLeft() when peerLeft != null:
return peerLeft(_that.peerId);case RsRelayEvent_Disconnected() when disconnected != null:
return disconnected(_that.error);case _:
  return null;

}
}

}

/// @nodoc


class RsRelayEvent_Connected extends RsRelayEvent {
  const RsRelayEvent_Connected({required this.clientId}): super._();
  

 final  String clientId;

/// Create a copy of RsRelayEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RsRelayEvent_ConnectedCopyWith<RsRelayEvent_Connected> get copyWith => _$RsRelayEvent_ConnectedCopyWithImpl<RsRelayEvent_Connected>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RsRelayEvent_Connected&&(identical(other.clientId, clientId) || other.clientId == clientId));
}


@override
int get hashCode => Object.hash(runtimeType,clientId);

@override
String toString() {
  return 'RsRelayEvent.connected(clientId: $clientId)';
}


}

/// @nodoc
abstract mixin class $RsRelayEvent_ConnectedCopyWith<$Res> implements $RsRelayEventCopyWith<$Res> {
  factory $RsRelayEvent_ConnectedCopyWith(RsRelayEvent_Connected value, $Res Function(RsRelayEvent_Connected) _then) = _$RsRelayEvent_ConnectedCopyWithImpl;
@useResult
$Res call({
 String clientId
});




}
/// @nodoc
class _$RsRelayEvent_ConnectedCopyWithImpl<$Res>
    implements $RsRelayEvent_ConnectedCopyWith<$Res> {
  _$RsRelayEvent_ConnectedCopyWithImpl(this._self, this._then);

  final RsRelayEvent_Connected _self;
  final $Res Function(RsRelayEvent_Connected) _then;

/// Create a copy of RsRelayEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? clientId = null,}) {
  return _then(RsRelayEvent_Connected(
clientId: null == clientId ? _self.clientId : clientId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class RsRelayEvent_Peer extends RsRelayEvent {
  const RsRelayEvent_Peer({required this.peer}): super._();
  

 final  RsRelayPeer peer;

/// Create a copy of RsRelayEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RsRelayEvent_PeerCopyWith<RsRelayEvent_Peer> get copyWith => _$RsRelayEvent_PeerCopyWithImpl<RsRelayEvent_Peer>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RsRelayEvent_Peer&&(identical(other.peer, peer) || other.peer == peer));
}


@override
int get hashCode => Object.hash(runtimeType,peer);

@override
String toString() {
  return 'RsRelayEvent.peer(peer: $peer)';
}


}

/// @nodoc
abstract mixin class $RsRelayEvent_PeerCopyWith<$Res> implements $RsRelayEventCopyWith<$Res> {
  factory $RsRelayEvent_PeerCopyWith(RsRelayEvent_Peer value, $Res Function(RsRelayEvent_Peer) _then) = _$RsRelayEvent_PeerCopyWithImpl;
@useResult
$Res call({
 RsRelayPeer peer
});




}
/// @nodoc
class _$RsRelayEvent_PeerCopyWithImpl<$Res>
    implements $RsRelayEvent_PeerCopyWith<$Res> {
  _$RsRelayEvent_PeerCopyWithImpl(this._self, this._then);

  final RsRelayEvent_Peer _self;
  final $Res Function(RsRelayEvent_Peer) _then;

/// Create a copy of RsRelayEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? peer = null,}) {
  return _then(RsRelayEvent_Peer(
peer: null == peer ? _self.peer : peer // ignore: cast_nullable_to_non_nullable
as RsRelayPeer,
  ));
}


}

/// @nodoc


class RsRelayEvent_PeerUpdate extends RsRelayEvent {
  const RsRelayEvent_PeerUpdate({required this.peer}): super._();
  

 final  RsRelayPeer peer;

/// Create a copy of RsRelayEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RsRelayEvent_PeerUpdateCopyWith<RsRelayEvent_PeerUpdate> get copyWith => _$RsRelayEvent_PeerUpdateCopyWithImpl<RsRelayEvent_PeerUpdate>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RsRelayEvent_PeerUpdate&&(identical(other.peer, peer) || other.peer == peer));
}


@override
int get hashCode => Object.hash(runtimeType,peer);

@override
String toString() {
  return 'RsRelayEvent.peerUpdate(peer: $peer)';
}


}

/// @nodoc
abstract mixin class $RsRelayEvent_PeerUpdateCopyWith<$Res> implements $RsRelayEventCopyWith<$Res> {
  factory $RsRelayEvent_PeerUpdateCopyWith(RsRelayEvent_PeerUpdate value, $Res Function(RsRelayEvent_PeerUpdate) _then) = _$RsRelayEvent_PeerUpdateCopyWithImpl;
@useResult
$Res call({
 RsRelayPeer peer
});




}
/// @nodoc
class _$RsRelayEvent_PeerUpdateCopyWithImpl<$Res>
    implements $RsRelayEvent_PeerUpdateCopyWith<$Res> {
  _$RsRelayEvent_PeerUpdateCopyWithImpl(this._self, this._then);

  final RsRelayEvent_PeerUpdate _self;
  final $Res Function(RsRelayEvent_PeerUpdate) _then;

/// Create a copy of RsRelayEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? peer = null,}) {
  return _then(RsRelayEvent_PeerUpdate(
peer: null == peer ? _self.peer : peer // ignore: cast_nullable_to_non_nullable
as RsRelayPeer,
  ));
}


}

/// @nodoc


class RsRelayEvent_PeerLeft extends RsRelayEvent {
  const RsRelayEvent_PeerLeft({required this.peerId}): super._();
  

 final  String peerId;

/// Create a copy of RsRelayEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RsRelayEvent_PeerLeftCopyWith<RsRelayEvent_PeerLeft> get copyWith => _$RsRelayEvent_PeerLeftCopyWithImpl<RsRelayEvent_PeerLeft>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RsRelayEvent_PeerLeft&&(identical(other.peerId, peerId) || other.peerId == peerId));
}


@override
int get hashCode => Object.hash(runtimeType,peerId);

@override
String toString() {
  return 'RsRelayEvent.peerLeft(peerId: $peerId)';
}


}

/// @nodoc
abstract mixin class $RsRelayEvent_PeerLeftCopyWith<$Res> implements $RsRelayEventCopyWith<$Res> {
  factory $RsRelayEvent_PeerLeftCopyWith(RsRelayEvent_PeerLeft value, $Res Function(RsRelayEvent_PeerLeft) _then) = _$RsRelayEvent_PeerLeftCopyWithImpl;
@useResult
$Res call({
 String peerId
});




}
/// @nodoc
class _$RsRelayEvent_PeerLeftCopyWithImpl<$Res>
    implements $RsRelayEvent_PeerLeftCopyWith<$Res> {
  _$RsRelayEvent_PeerLeftCopyWithImpl(this._self, this._then);

  final RsRelayEvent_PeerLeft _self;
  final $Res Function(RsRelayEvent_PeerLeft) _then;

/// Create a copy of RsRelayEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? peerId = null,}) {
  return _then(RsRelayEvent_PeerLeft(
peerId: null == peerId ? _self.peerId : peerId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class RsRelayEvent_Disconnected extends RsRelayEvent {
  const RsRelayEvent_Disconnected({this.error}): super._();
  

 final  String? error;

/// Create a copy of RsRelayEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RsRelayEvent_DisconnectedCopyWith<RsRelayEvent_Disconnected> get copyWith => _$RsRelayEvent_DisconnectedCopyWithImpl<RsRelayEvent_Disconnected>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RsRelayEvent_Disconnected&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,error);

@override
String toString() {
  return 'RsRelayEvent.disconnected(error: $error)';
}


}

/// @nodoc
abstract mixin class $RsRelayEvent_DisconnectedCopyWith<$Res> implements $RsRelayEventCopyWith<$Res> {
  factory $RsRelayEvent_DisconnectedCopyWith(RsRelayEvent_Disconnected value, $Res Function(RsRelayEvent_Disconnected) _then) = _$RsRelayEvent_DisconnectedCopyWithImpl;
@useResult
$Res call({
 String? error
});




}
/// @nodoc
class _$RsRelayEvent_DisconnectedCopyWithImpl<$Res>
    implements $RsRelayEvent_DisconnectedCopyWith<$Res> {
  _$RsRelayEvent_DisconnectedCopyWithImpl(this._self, this._then);

  final RsRelayEvent_Disconnected _self;
  final $Res Function(RsRelayEvent_Disconnected) _then;

/// Create a copy of RsRelayEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? error = freezed,}) {
  return _then(RsRelayEvent_Disconnected(
error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
