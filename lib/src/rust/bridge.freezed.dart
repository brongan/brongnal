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
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$BridgeError {
  String get field0 => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) registrationFailed,
    required TResult Function(String field0) initializationFailed,
    required TResult Function(String field0) messageSendFailed,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? registrationFailed,
    TResult? Function(String field0)? initializationFailed,
    TResult? Function(String field0)? messageSendFailed,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? registrationFailed,
    TResult Function(String field0)? initializationFailed,
    TResult Function(String field0)? messageSendFailed,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(BridgeError_RegistrationFailed value)
        registrationFailed,
    required TResult Function(BridgeError_InitializationFailed value)
        initializationFailed,
    required TResult Function(BridgeError_MessageSendFailed value)
        messageSendFailed,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult? Function(BridgeError_InitializationFailed value)?
        initializationFailed,
    TResult? Function(BridgeError_MessageSendFailed value)? messageSendFailed,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(BridgeError_RegistrationFailed value)? registrationFailed,
    TResult Function(BridgeError_InitializationFailed value)?
        initializationFailed,
    TResult Function(BridgeError_MessageSendFailed value)? messageSendFailed,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $BridgeErrorCopyWith<BridgeError> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $BridgeErrorCopyWith<$Res> {
  factory $BridgeErrorCopyWith(
          BridgeError value, $Res Function(BridgeError) then) =
      _$BridgeErrorCopyWithImpl<$Res, BridgeError>;
  @useResult
  $Res call({String field0});
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
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_value.copyWith(
      field0: null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$BridgeError_RegistrationFailedImplCopyWith<$Res>
    implements $BridgeErrorCopyWith<$Res> {
  factory _$$BridgeError_RegistrationFailedImplCopyWith(
          _$BridgeError_RegistrationFailedImpl value,
          $Res Function(_$BridgeError_RegistrationFailedImpl) then) =
      __$$BridgeError_RegistrationFailedImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String field0});
}

/// @nodoc
class __$$BridgeError_RegistrationFailedImplCopyWithImpl<$Res>
    extends _$BridgeErrorCopyWithImpl<$Res,
        _$BridgeError_RegistrationFailedImpl>
    implements _$$BridgeError_RegistrationFailedImplCopyWith<$Res> {
  __$$BridgeError_RegistrationFailedImplCopyWithImpl(
      _$BridgeError_RegistrationFailedImpl _value,
      $Res Function(_$BridgeError_RegistrationFailedImpl) _then)
      : super(_value, _then);

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$BridgeError_RegistrationFailedImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
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
          _$BridgeError_RegistrationFailedImpl>
      get copyWith => __$$BridgeError_RegistrationFailedImplCopyWithImpl<
          _$BridgeError_RegistrationFailedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) registrationFailed,
    required TResult Function(String field0) initializationFailed,
    required TResult Function(String field0) messageSendFailed,
  }) {
    return registrationFailed(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? registrationFailed,
    TResult? Function(String field0)? initializationFailed,
    TResult? Function(String field0)? messageSendFailed,
  }) {
    return registrationFailed?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? registrationFailed,
    TResult Function(String field0)? initializationFailed,
    TResult Function(String field0)? messageSendFailed,
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

  @override
  String get field0;

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$BridgeError_RegistrationFailedImplCopyWith<
          _$BridgeError_RegistrationFailedImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$BridgeError_InitializationFailedImplCopyWith<$Res>
    implements $BridgeErrorCopyWith<$Res> {
  factory _$$BridgeError_InitializationFailedImplCopyWith(
          _$BridgeError_InitializationFailedImpl value,
          $Res Function(_$BridgeError_InitializationFailedImpl) then) =
      __$$BridgeError_InitializationFailedImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String field0});
}

/// @nodoc
class __$$BridgeError_InitializationFailedImplCopyWithImpl<$Res>
    extends _$BridgeErrorCopyWithImpl<$Res,
        _$BridgeError_InitializationFailedImpl>
    implements _$$BridgeError_InitializationFailedImplCopyWith<$Res> {
  __$$BridgeError_InitializationFailedImplCopyWithImpl(
      _$BridgeError_InitializationFailedImpl _value,
      $Res Function(_$BridgeError_InitializationFailedImpl) _then)
      : super(_value, _then);

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$BridgeError_InitializationFailedImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
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
          _$BridgeError_InitializationFailedImpl>
      get copyWith => __$$BridgeError_InitializationFailedImplCopyWithImpl<
          _$BridgeError_InitializationFailedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) registrationFailed,
    required TResult Function(String field0) initializationFailed,
    required TResult Function(String field0) messageSendFailed,
  }) {
    return initializationFailed(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? registrationFailed,
    TResult? Function(String field0)? initializationFailed,
    TResult? Function(String field0)? messageSendFailed,
  }) {
    return initializationFailed?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? registrationFailed,
    TResult Function(String field0)? initializationFailed,
    TResult Function(String field0)? messageSendFailed,
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

  @override
  String get field0;

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$BridgeError_InitializationFailedImplCopyWith<
          _$BridgeError_InitializationFailedImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$BridgeError_MessageSendFailedImplCopyWith<$Res>
    implements $BridgeErrorCopyWith<$Res> {
  factory _$$BridgeError_MessageSendFailedImplCopyWith(
          _$BridgeError_MessageSendFailedImpl value,
          $Res Function(_$BridgeError_MessageSendFailedImpl) then) =
      __$$BridgeError_MessageSendFailedImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String field0});
}

/// @nodoc
class __$$BridgeError_MessageSendFailedImplCopyWithImpl<$Res>
    extends _$BridgeErrorCopyWithImpl<$Res, _$BridgeError_MessageSendFailedImpl>
    implements _$$BridgeError_MessageSendFailedImplCopyWith<$Res> {
  __$$BridgeError_MessageSendFailedImplCopyWithImpl(
      _$BridgeError_MessageSendFailedImpl _value,
      $Res Function(_$BridgeError_MessageSendFailedImpl) _then)
      : super(_value, _then);

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
  }) {
    return _then(_$BridgeError_MessageSendFailedImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
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
          _$BridgeError_MessageSendFailedImpl>
      get copyWith => __$$BridgeError_MessageSendFailedImplCopyWithImpl<
          _$BridgeError_MessageSendFailedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String field0) registrationFailed,
    required TResult Function(String field0) initializationFailed,
    required TResult Function(String field0) messageSendFailed,
  }) {
    return messageSendFailed(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String field0)? registrationFailed,
    TResult? Function(String field0)? initializationFailed,
    TResult? Function(String field0)? messageSendFailed,
  }) {
    return messageSendFailed?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String field0)? registrationFailed,
    TResult Function(String field0)? initializationFailed,
    TResult Function(String field0)? messageSendFailed,
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

  @override
  String get field0;

  /// Create a copy of BridgeError
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$BridgeError_MessageSendFailedImplCopyWith<
          _$BridgeError_MessageSendFailedImpl>
      get copyWith => throw _privateConstructorUsedError;
}
