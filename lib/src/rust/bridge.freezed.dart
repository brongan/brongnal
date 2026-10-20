// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'bridge.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

/// @nodoc
mixin _$BridgeError {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) registrationFailed,
    required TResult Function(String field0) initializationFailed,
    required TResult Function(String field0) messageSendFailed,
    required TResult Function() serverUnreachable,
    required TResult Function() serverError,
    required TResult Function() registrationRejected,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? registrationFailed,
    TResult? Function(String field0)? initializationFailed,
    TResult? Function(String field0)? messageSendFailed,
    TResult? Function()? serverUnreachable,
    TResult? Function()? serverError,
    TResult? Function()? registrationRejected,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? registrationFailed,
    TResult Function(String field0)? initializationFailed,
    TResult Function(String field0)? messageSendFailed,
    TResult Function()? serverUnreachable,
    TResult Function()? serverError,
    TResult Function()? registrationRejected,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(BridgeError_RegistrationFailed value)
    registrationFailed,
    required TResult Function(BridgeError_InitializationFailed value)
    initializationFailed,
    required TResult Function(BridgeError_MessageSendFailed value)
    messageSendFailed,
    required TResult Function(BridgeError_ServerUnreachable value)
    serverUnreachable,
    required TResult Function(BridgeError_ServerError value) serverError,
    required TResult Function(BridgeError_RegistrationRejected value)
    registrationRejected,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult? Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult? Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult? Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult? Function(BridgeError_ServerError value)? serverError,
    TResult? Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult Function(BridgeError_ServerError value)? serverError,
    TResult Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $BridgeErrorCopyWith<$Res> {
  factory $BridgeErrorCopyWith(
    BridgeError value,
    $Res Function(BridgeError) then,
  ) = _$BridgeErrorCopyWithImpl<$Res, BridgeError>;
}

/// @nodoc
class _$BridgeErrorCopyWithImpl<$Res, $Val extends BridgeError>
    implements $BridgeErrorCopyWith<$Res> {
  _$BridgeErrorCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$BridgeError_RegistrationFailedImplCopyWith<$Res> {
  factory _$$BridgeError_RegistrationFailedImplCopyWith(
    _$BridgeError_RegistrationFailedImpl value,
    $Res Function(_$BridgeError_RegistrationFailedImpl) then,
  ) = __$$BridgeError_RegistrationFailedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class __$$BridgeError_RegistrationFailedImplCopyWithImpl<$Res>
    extends
        _$BridgeErrorCopyWithImpl<$Res, _$BridgeError_RegistrationFailedImpl>
    implements _$$BridgeError_RegistrationFailedImplCopyWith<$Res> {
  __$$BridgeError_RegistrationFailedImplCopyWithImpl(
    _$BridgeError_RegistrationFailedImpl _value,
    $Res Function(_$BridgeError_RegistrationFailedImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null}) {
    return _then(
      _$BridgeError_RegistrationFailedImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc

class _$BridgeError_RegistrationFailedImpl
    extends BridgeError_RegistrationFailed {
  const _$BridgeError_RegistrationFailedImpl(this.field0) : super._();

  @override
  final String field0;

  @override
  String toString() {
    return 'BridgeError.registrationFailed(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$BridgeError_RegistrationFailedImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$BridgeError_RegistrationFailedImplCopyWith<
    _$BridgeError_RegistrationFailedImpl
  >
  get copyWith =>
      __$$BridgeError_RegistrationFailedImplCopyWithImpl<
        _$BridgeError_RegistrationFailedImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) registrationFailed,
    required TResult Function(String field0) initializationFailed,
    required TResult Function(String field0) messageSendFailed,
    required TResult Function() serverUnreachable,
    required TResult Function() serverError,
    required TResult Function() registrationRejected,
  }) {
    return registrationFailed(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? registrationFailed,
    TResult? Function(String field0)? initializationFailed,
    TResult? Function(String field0)? messageSendFailed,
    TResult? Function()? serverUnreachable,
    TResult? Function()? serverError,
    TResult? Function()? registrationRejected,
  }) {
    return registrationFailed?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? registrationFailed,
    TResult Function(String field0)? initializationFailed,
    TResult Function(String field0)? messageSendFailed,
    TResult Function()? serverUnreachable,
    TResult Function()? serverError,
    TResult Function()? registrationRejected,
    required TResult orElse(),
  }) {
    if (registrationFailed != null) {
      return registrationFailed(field0);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(BridgeError_RegistrationFailed value)
    registrationFailed,
    required TResult Function(BridgeError_InitializationFailed value)
    initializationFailed,
    required TResult Function(BridgeError_MessageSendFailed value)
    messageSendFailed,
    required TResult Function(BridgeError_ServerUnreachable value)
    serverUnreachable,
    required TResult Function(BridgeError_ServerError value) serverError,
    required TResult Function(BridgeError_RegistrationRejected value)
    registrationRejected,
  }) {
    return registrationFailed(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult? Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult? Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult? Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult? Function(BridgeError_ServerError value)? serverError,
    TResult? Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
  }) {
    return registrationFailed?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult Function(BridgeError_ServerError value)? serverError,
    TResult Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
    required TResult orElse(),
  }) {
    if (registrationFailed != null) {
      return registrationFailed(this);
    }
    return orElse();
  }
}

abstract class BridgeError_RegistrationFailed extends BridgeError {
  const factory BridgeError_RegistrationFailed(final String field0) =
      _$BridgeError_RegistrationFailedImpl;
  const BridgeError_RegistrationFailed._() : super._();

  String get field0;

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$BridgeError_RegistrationFailedImplCopyWith<
    _$BridgeError_RegistrationFailedImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$BridgeError_InitializationFailedImplCopyWith<$Res> {
  factory _$$BridgeError_InitializationFailedImplCopyWith(
    _$BridgeError_InitializationFailedImpl value,
    $Res Function(_$BridgeError_InitializationFailedImpl) then,
  ) = __$$BridgeError_InitializationFailedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class __$$BridgeError_InitializationFailedImplCopyWithImpl<$Res>
    extends
        _$BridgeErrorCopyWithImpl<$Res, _$BridgeError_InitializationFailedImpl>
    implements _$$BridgeError_InitializationFailedImplCopyWith<$Res> {
  __$$BridgeError_InitializationFailedImplCopyWithImpl(
    _$BridgeError_InitializationFailedImpl _value,
    $Res Function(_$BridgeError_InitializationFailedImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null}) {
    return _then(
      _$BridgeError_InitializationFailedImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc

class _$BridgeError_InitializationFailedImpl
    extends BridgeError_InitializationFailed {
  const _$BridgeError_InitializationFailedImpl(this.field0) : super._();

  @override
  final String field0;

  @override
  String toString() {
    return 'BridgeError.initializationFailed(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$BridgeError_InitializationFailedImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$BridgeError_InitializationFailedImplCopyWith<
    _$BridgeError_InitializationFailedImpl
  >
  get copyWith =>
      __$$BridgeError_InitializationFailedImplCopyWithImpl<
        _$BridgeError_InitializationFailedImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) registrationFailed,
    required TResult Function(String field0) initializationFailed,
    required TResult Function(String field0) messageSendFailed,
    required TResult Function() serverUnreachable,
    required TResult Function() serverError,
    required TResult Function() registrationRejected,
  }) {
    return initializationFailed(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? registrationFailed,
    TResult? Function(String field0)? initializationFailed,
    TResult? Function(String field0)? messageSendFailed,
    TResult? Function()? serverUnreachable,
    TResult? Function()? serverError,
    TResult? Function()? registrationRejected,
  }) {
    return initializationFailed?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? registrationFailed,
    TResult Function(String field0)? initializationFailed,
    TResult Function(String field0)? messageSendFailed,
    TResult Function()? serverUnreachable,
    TResult Function()? serverError,
    TResult Function()? registrationRejected,
    required TResult orElse(),
  }) {
    if (initializationFailed != null) {
      return initializationFailed(field0);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(BridgeError_RegistrationFailed value)
    registrationFailed,
    required TResult Function(BridgeError_InitializationFailed value)
    initializationFailed,
    required TResult Function(BridgeError_MessageSendFailed value)
    messageSendFailed,
    required TResult Function(BridgeError_ServerUnreachable value)
    serverUnreachable,
    required TResult Function(BridgeError_ServerError value) serverError,
    required TResult Function(BridgeError_RegistrationRejected value)
    registrationRejected,
  }) {
    return initializationFailed(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult? Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult? Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult? Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult? Function(BridgeError_ServerError value)? serverError,
    TResult? Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
  }) {
    return initializationFailed?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult Function(BridgeError_ServerError value)? serverError,
    TResult Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
    required TResult orElse(),
  }) {
    if (initializationFailed != null) {
      return initializationFailed(this);
    }
    return orElse();
  }
}

abstract class BridgeError_InitializationFailed extends BridgeError {
  const factory BridgeError_InitializationFailed(final String field0) =
      _$BridgeError_InitializationFailedImpl;
  const BridgeError_InitializationFailed._() : super._();

  String get field0;

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$BridgeError_InitializationFailedImplCopyWith<
    _$BridgeError_InitializationFailedImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$BridgeError_MessageSendFailedImplCopyWith<$Res> {
  factory _$$BridgeError_MessageSendFailedImplCopyWith(
    _$BridgeError_MessageSendFailedImpl value,
    $Res Function(_$BridgeError_MessageSendFailedImpl) then,
  ) = __$$BridgeError_MessageSendFailedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class __$$BridgeError_MessageSendFailedImplCopyWithImpl<$Res>
    extends _$BridgeErrorCopyWithImpl<$Res, _$BridgeError_MessageSendFailedImpl>
    implements _$$BridgeError_MessageSendFailedImplCopyWith<$Res> {
  __$$BridgeError_MessageSendFailedImplCopyWithImpl(
    _$BridgeError_MessageSendFailedImpl _value,
    $Res Function(_$BridgeError_MessageSendFailedImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null}) {
    return _then(
      _$BridgeError_MessageSendFailedImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc

class _$BridgeError_MessageSendFailedImpl
    extends BridgeError_MessageSendFailed {
  const _$BridgeError_MessageSendFailedImpl(this.field0) : super._();

  @override
  final String field0;

  @override
  String toString() {
    return 'BridgeError.messageSendFailed(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$BridgeError_MessageSendFailedImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$BridgeError_MessageSendFailedImplCopyWith<
    _$BridgeError_MessageSendFailedImpl
  >
  get copyWith =>
      __$$BridgeError_MessageSendFailedImplCopyWithImpl<
        _$BridgeError_MessageSendFailedImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) registrationFailed,
    required TResult Function(String field0) initializationFailed,
    required TResult Function(String field0) messageSendFailed,
    required TResult Function() serverUnreachable,
    required TResult Function() serverError,
    required TResult Function() registrationRejected,
  }) {
    return messageSendFailed(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? registrationFailed,
    TResult? Function(String field0)? initializationFailed,
    TResult? Function(String field0)? messageSendFailed,
    TResult? Function()? serverUnreachable,
    TResult? Function()? serverError,
    TResult? Function()? registrationRejected,
  }) {
    return messageSendFailed?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? registrationFailed,
    TResult Function(String field0)? initializationFailed,
    TResult Function(String field0)? messageSendFailed,
    TResult Function()? serverUnreachable,
    TResult Function()? serverError,
    TResult Function()? registrationRejected,
    required TResult orElse(),
  }) {
    if (messageSendFailed != null) {
      return messageSendFailed(field0);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(BridgeError_RegistrationFailed value)
    registrationFailed,
    required TResult Function(BridgeError_InitializationFailed value)
    initializationFailed,
    required TResult Function(BridgeError_MessageSendFailed value)
    messageSendFailed,
    required TResult Function(BridgeError_ServerUnreachable value)
    serverUnreachable,
    required TResult Function(BridgeError_ServerError value) serverError,
    required TResult Function(BridgeError_RegistrationRejected value)
    registrationRejected,
  }) {
    return messageSendFailed(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult? Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult? Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult? Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult? Function(BridgeError_ServerError value)? serverError,
    TResult? Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
  }) {
    return messageSendFailed?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult Function(BridgeError_ServerError value)? serverError,
    TResult Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
    required TResult orElse(),
  }) {
    if (messageSendFailed != null) {
      return messageSendFailed(this);
    }
    return orElse();
  }
}

abstract class BridgeError_MessageSendFailed extends BridgeError {
  const factory BridgeError_MessageSendFailed(final String field0) =
      _$BridgeError_MessageSendFailedImpl;
  const BridgeError_MessageSendFailed._() : super._();

  String get field0;

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$BridgeError_MessageSendFailedImplCopyWith<
    _$BridgeError_MessageSendFailedImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$BridgeError_ServerUnreachableImplCopyWith<$Res> {
  factory _$$BridgeError_ServerUnreachableImplCopyWith(
    _$BridgeError_ServerUnreachableImpl value,
    $Res Function(_$BridgeError_ServerUnreachableImpl) then,
  ) = __$$BridgeError_ServerUnreachableImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$BridgeError_ServerUnreachableImplCopyWithImpl<$Res>
    extends _$BridgeErrorCopyWithImpl<$Res, _$BridgeError_ServerUnreachableImpl>
    implements _$$BridgeError_ServerUnreachableImplCopyWith<$Res> {
  __$$BridgeError_ServerUnreachableImplCopyWithImpl(
    _$BridgeError_ServerUnreachableImpl _value,
    $Res Function(_$BridgeError_ServerUnreachableImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$BridgeError_ServerUnreachableImpl
    extends BridgeError_ServerUnreachable {
  const _$BridgeError_ServerUnreachableImpl() : super._();

  @override
  String toString() {
    return 'BridgeError.serverUnreachable()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$BridgeError_ServerUnreachableImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) registrationFailed,
    required TResult Function(String field0) initializationFailed,
    required TResult Function(String field0) messageSendFailed,
    required TResult Function() serverUnreachable,
    required TResult Function() serverError,
    required TResult Function() registrationRejected,
  }) {
    return serverUnreachable();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? registrationFailed,
    TResult? Function(String field0)? initializationFailed,
    TResult? Function(String field0)? messageSendFailed,
    TResult? Function()? serverUnreachable,
    TResult? Function()? serverError,
    TResult? Function()? registrationRejected,
  }) {
    return serverUnreachable?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? registrationFailed,
    TResult Function(String field0)? initializationFailed,
    TResult Function(String field0)? messageSendFailed,
    TResult Function()? serverUnreachable,
    TResult Function()? serverError,
    TResult Function()? registrationRejected,
    required TResult orElse(),
  }) {
    if (serverUnreachable != null) {
      return serverUnreachable();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(BridgeError_RegistrationFailed value)
    registrationFailed,
    required TResult Function(BridgeError_InitializationFailed value)
    initializationFailed,
    required TResult Function(BridgeError_MessageSendFailed value)
    messageSendFailed,
    required TResult Function(BridgeError_ServerUnreachable value)
    serverUnreachable,
    required TResult Function(BridgeError_ServerError value) serverError,
    required TResult Function(BridgeError_RegistrationRejected value)
    registrationRejected,
  }) {
    return serverUnreachable(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult? Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult? Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult? Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult? Function(BridgeError_ServerError value)? serverError,
    TResult? Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
  }) {
    return serverUnreachable?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult Function(BridgeError_ServerError value)? serverError,
    TResult Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
    required TResult orElse(),
  }) {
    if (serverUnreachable != null) {
      return serverUnreachable(this);
    }
    return orElse();
  }
}

abstract class BridgeError_ServerUnreachable extends BridgeError {
  const factory BridgeError_ServerUnreachable() =
      _$BridgeError_ServerUnreachableImpl;
  const BridgeError_ServerUnreachable._() : super._();
}

/// @nodoc
abstract class _$$BridgeError_ServerErrorImplCopyWith<$Res> {
  factory _$$BridgeError_ServerErrorImplCopyWith(
    _$BridgeError_ServerErrorImpl value,
    $Res Function(_$BridgeError_ServerErrorImpl) then,
  ) = __$$BridgeError_ServerErrorImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$BridgeError_ServerErrorImplCopyWithImpl<$Res>
    extends _$BridgeErrorCopyWithImpl<$Res, _$BridgeError_ServerErrorImpl>
    implements _$$BridgeError_ServerErrorImplCopyWith<$Res> {
  __$$BridgeError_ServerErrorImplCopyWithImpl(
    _$BridgeError_ServerErrorImpl _value,
    $Res Function(_$BridgeError_ServerErrorImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$BridgeError_ServerErrorImpl extends BridgeError_ServerError {
  const _$BridgeError_ServerErrorImpl() : super._();

  @override
  String toString() {
    return 'BridgeError.serverError()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$BridgeError_ServerErrorImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) registrationFailed,
    required TResult Function(String field0) initializationFailed,
    required TResult Function(String field0) messageSendFailed,
    required TResult Function() serverUnreachable,
    required TResult Function() serverError,
    required TResult Function() registrationRejected,
  }) {
    return serverError();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? registrationFailed,
    TResult? Function(String field0)? initializationFailed,
    TResult? Function(String field0)? messageSendFailed,
    TResult? Function()? serverUnreachable,
    TResult? Function()? serverError,
    TResult? Function()? registrationRejected,
  }) {
    return serverError?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? registrationFailed,
    TResult Function(String field0)? initializationFailed,
    TResult Function(String field0)? messageSendFailed,
    TResult Function()? serverUnreachable,
    TResult Function()? serverError,
    TResult Function()? registrationRejected,
    required TResult orElse(),
  }) {
    if (serverError != null) {
      return serverError();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(BridgeError_RegistrationFailed value)
    registrationFailed,
    required TResult Function(BridgeError_InitializationFailed value)
    initializationFailed,
    required TResult Function(BridgeError_MessageSendFailed value)
    messageSendFailed,
    required TResult Function(BridgeError_ServerUnreachable value)
    serverUnreachable,
    required TResult Function(BridgeError_ServerError value) serverError,
    required TResult Function(BridgeError_RegistrationRejected value)
    registrationRejected,
  }) {
    return serverError(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult? Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult? Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult? Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult? Function(BridgeError_ServerError value)? serverError,
    TResult? Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
  }) {
    return serverError?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult Function(BridgeError_ServerError value)? serverError,
    TResult Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
    required TResult orElse(),
  }) {
    if (serverError != null) {
      return serverError(this);
    }
    return orElse();
  }
}

abstract class BridgeError_ServerError extends BridgeError {
  const factory BridgeError_ServerError() = _$BridgeError_ServerErrorImpl;
  const BridgeError_ServerError._() : super._();
}

/// @nodoc
abstract class _$$BridgeError_RegistrationRejectedImplCopyWith<$Res> {
  factory _$$BridgeError_RegistrationRejectedImplCopyWith(
    _$BridgeError_RegistrationRejectedImpl value,
    $Res Function(_$BridgeError_RegistrationRejectedImpl) then,
  ) = __$$BridgeError_RegistrationRejectedImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$BridgeError_RegistrationRejectedImplCopyWithImpl<$Res>
    extends
        _$BridgeErrorCopyWithImpl<$Res, _$BridgeError_RegistrationRejectedImpl>
    implements _$$BridgeError_RegistrationRejectedImplCopyWith<$Res> {
  __$$BridgeError_RegistrationRejectedImplCopyWithImpl(
    _$BridgeError_RegistrationRejectedImpl _value,
    $Res Function(_$BridgeError_RegistrationRejectedImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$BridgeError_RegistrationRejectedImpl
    extends BridgeError_RegistrationRejected {
  const _$BridgeError_RegistrationRejectedImpl() : super._();

  @override
  String toString() {
    return 'BridgeError.registrationRejected()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$BridgeError_RegistrationRejectedImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) registrationFailed,
    required TResult Function(String field0) initializationFailed,
    required TResult Function(String field0) messageSendFailed,
    required TResult Function() serverUnreachable,
    required TResult Function() serverError,
    required TResult Function() registrationRejected,
  }) {
    return registrationRejected();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? registrationFailed,
    TResult? Function(String field0)? initializationFailed,
    TResult? Function(String field0)? messageSendFailed,
    TResult? Function()? serverUnreachable,
    TResult? Function()? serverError,
    TResult? Function()? registrationRejected,
  }) {
    return registrationRejected?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? registrationFailed,
    TResult Function(String field0)? initializationFailed,
    TResult Function(String field0)? messageSendFailed,
    TResult Function()? serverUnreachable,
    TResult Function()? serverError,
    TResult Function()? registrationRejected,
    required TResult orElse(),
  }) {
    if (registrationRejected != null) {
      return registrationRejected();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(BridgeError_RegistrationFailed value)
    registrationFailed,
    required TResult Function(BridgeError_InitializationFailed value)
    initializationFailed,
    required TResult Function(BridgeError_MessageSendFailed value)
    messageSendFailed,
    required TResult Function(BridgeError_ServerUnreachable value)
    serverUnreachable,
    required TResult Function(BridgeError_ServerError value) serverError,
    required TResult Function(BridgeError_RegistrationRejected value)
    registrationRejected,
  }) {
    return registrationRejected(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult? Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult? Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult? Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult? Function(BridgeError_ServerError value)? serverError,
    TResult? Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
  }) {
    return registrationRejected?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult Function(BridgeError_InitializationFailed value)?
    initializationFailed,
    TResult Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    TResult Function(BridgeError_ServerUnreachable value)? serverUnreachable,
    TResult Function(BridgeError_ServerError value)? serverError,
    TResult Function(BridgeError_RegistrationRejected value)?
    registrationRejected,
    required TResult orElse(),
  }) {
    if (registrationRejected != null) {
      return registrationRejected(this);
    }
    return orElse();
  }
}

abstract class BridgeError_RegistrationRejected extends BridgeError {
  const factory BridgeError_RegistrationRejected() =
      _$BridgeError_RegistrationRejectedImpl;
  const BridgeError_RegistrationRejected._() : super._();
}
