//
//  Generated code. Do not modify.
//  source: service.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

class SignedPreKey extends $pb.GeneratedMessage {
  factory SignedPreKey({
    $core.List<$core.int>? preKey,
    $core.List<$core.int>? signature,
  }) {
    final $result = create();
    if (preKey != null) {
      $result.preKey = preKey;
    }
    if (signature != null) {
      $result.signature = signature;
    }
    return $result;
  }
  SignedPreKey._() : super();
  factory SignedPreKey.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SignedPreKey.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SignedPreKey', package: const $pb.PackageName(_omitMessageNames ? '' : 'service'), createEmptyInstance: create)
    ..a<$core.List<$core.int>>(1, _omitFieldNames ? '' : 'preKey', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(2, _omitFieldNames ? '' : 'signature', $pb.PbFieldType.OY)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SignedPreKey clone() => SignedPreKey()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SignedPreKey copyWith(void Function(SignedPreKey) updates) => super.copyWith((message) => updates(message as SignedPreKey)) as SignedPreKey;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SignedPreKey create() => SignedPreKey._();
  SignedPreKey createEmptyInstance() => create();
  static $pb.PbList<SignedPreKey> createRepeated() => $pb.PbList<SignedPreKey>();
  @$core.pragma('dart2js:noInline')
  static SignedPreKey getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SignedPreKey>(create);
  static SignedPreKey? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get preKey => $_getN(0);
  @$pb.TagNumber(1)
  set preKey($core.List<$core.int> v) { $_setBytes(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasPreKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearPreKey() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get signature => $_getN(1);
  @$pb.TagNumber(2)
  set signature($core.List<$core.int> v) { $_setBytes(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasSignature() => $_has(1);
  @$pb.TagNumber(2)
  void clearSignature() => clearField(2);
}

class SignedPreKeys extends $pb.GeneratedMessage {
  factory SignedPreKeys({
    $core.Iterable<$core.List<$core.int>>? preKeys,
    $core.List<$core.int>? signature,
  }) {
    final $result = create();
    if (preKeys != null) {
      $result.preKeys.addAll(preKeys);
    }
    if (signature != null) {
      $result.signature = signature;
    }
    return $result;
  }
  SignedPreKeys._() : super();
  factory SignedPreKeys.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SignedPreKeys.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SignedPreKeys', package: const $pb.PackageName(_omitMessageNames ? '' : 'service'), createEmptyInstance: create)
    ..p<$core.List<$core.int>>(1, _omitFieldNames ? '' : 'preKeys', $pb.PbFieldType.PY)
    ..a<$core.List<$core.int>>(2, _omitFieldNames ? '' : 'signature', $pb.PbFieldType.OY)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SignedPreKeys clone() => SignedPreKeys()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SignedPreKeys copyWith(void Function(SignedPreKeys) updates) => super.copyWith((message) => updates(message as SignedPreKeys)) as SignedPreKeys;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SignedPreKeys create() => SignedPreKeys._();
  SignedPreKeys createEmptyInstance() => create();
  static $pb.PbList<SignedPreKeys> createRepeated() => $pb.PbList<SignedPreKeys>();
  @$core.pragma('dart2js:noInline')
  static SignedPreKeys getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SignedPreKeys>(create);
  static SignedPreKeys? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.List<$core.int>> get preKeys => $_getList(0);

  @$pb.TagNumber(2)
  $core.List<$core.int> get signature => $_getN(1);
  @$pb.TagNumber(2)
  set signature($core.List<$core.int> v) { $_setBytes(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasSignature() => $_has(1);
  @$pb.TagNumber(2)
  void clearSignature() => clearField(2);
}

class RegisterPreKeyBundleRequest extends $pb.GeneratedMessage {
  factory RegisterPreKeyBundleRequest({
    $core.String? identity,
    $core.List<$core.int>? ik,
    SignedPreKey? spk,
    SignedPreKeys? otkBundle,
  }) {
    final $result = create();
    if (identity != null) {
      $result.identity = identity;
    }
    if (ik != null) {
      $result.ik = ik;
    }
    if (spk != null) {
      $result.spk = spk;
    }
    if (otkBundle != null) {
      $result.otkBundle = otkBundle;
    }
    return $result;
  }
  RegisterPreKeyBundleRequest._() : super();
  factory RegisterPreKeyBundleRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory RegisterPreKeyBundleRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RegisterPreKeyBundleRequest', package: const $pb.PackageName(_omitMessageNames ? '' : 'service'), createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'identity')
    ..a<$core.List<$core.int>>(2, _omitFieldNames ? '' : 'ik', $pb.PbFieldType.OY)
    ..aOM<SignedPreKey>(3, _omitFieldNames ? '' : 'spk', subBuilder: SignedPreKey.create)
    ..aOM<SignedPreKeys>(4, _omitFieldNames ? '' : 'otkBundle', subBuilder: SignedPreKeys.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  RegisterPreKeyBundleRequest clone() => RegisterPreKeyBundleRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  RegisterPreKeyBundleRequest copyWith(void Function(RegisterPreKeyBundleRequest) updates) => super.copyWith((message) => updates(message as RegisterPreKeyBundleRequest)) as RegisterPreKeyBundleRequest;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RegisterPreKeyBundleRequest create() => RegisterPreKeyBundleRequest._();
  RegisterPreKeyBundleRequest createEmptyInstance() => create();
  static $pb.PbList<RegisterPreKeyBundleRequest> createRepeated() => $pb.PbList<RegisterPreKeyBundleRequest>();
  @$core.pragma('dart2js:noInline')
  static RegisterPreKeyBundleRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RegisterPreKeyBundleRequest>(create);
  static RegisterPreKeyBundleRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get identity => $_getSZ(0);
  @$pb.TagNumber(1)
  set identity($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasIdentity() => $_has(0);
  @$pb.TagNumber(1)
  void clearIdentity() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get ik => $_getN(1);
  @$pb.TagNumber(2)
  set ik($core.List<$core.int> v) { $_setBytes(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasIk() => $_has(1);
  @$pb.TagNumber(2)
  void clearIk() => clearField(2);

  @$pb.TagNumber(3)
  SignedPreKey get spk => $_getN(2);
  @$pb.TagNumber(3)
  set spk(SignedPreKey v) { setField(3, v); }
  @$pb.TagNumber(3)
  $core.bool hasSpk() => $_has(2);
  @$pb.TagNumber(3)
  void clearSpk() => clearField(3);
  @$pb.TagNumber(3)
  SignedPreKey ensureSpk() => $_ensure(2);

  @$pb.TagNumber(4)
  SignedPreKeys get otkBundle => $_getN(3);
  @$pb.TagNumber(4)
  set otkBundle(SignedPreKeys v) { setField(4, v); }
  @$pb.TagNumber(4)
  $core.bool hasOtkBundle() => $_has(3);
  @$pb.TagNumber(4)
  void clearOtkBundle() => clearField(4);
  @$pb.TagNumber(4)
  SignedPreKeys ensureOtkBundle() => $_ensure(3);
}

class RegisterPreKeyBundleResponse extends $pb.GeneratedMessage {
  factory RegisterPreKeyBundleResponse() => create();
  RegisterPreKeyBundleResponse._() : super();
  factory RegisterPreKeyBundleResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory RegisterPreKeyBundleResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RegisterPreKeyBundleResponse', package: const $pb.PackageName(_omitMessageNames ? '' : 'service'), createEmptyInstance: create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  RegisterPreKeyBundleResponse clone() => RegisterPreKeyBundleResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  RegisterPreKeyBundleResponse copyWith(void Function(RegisterPreKeyBundleResponse) updates) => super.copyWith((message) => updates(message as RegisterPreKeyBundleResponse)) as RegisterPreKeyBundleResponse;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RegisterPreKeyBundleResponse create() => RegisterPreKeyBundleResponse._();
  RegisterPreKeyBundleResponse createEmptyInstance() => create();
  static $pb.PbList<RegisterPreKeyBundleResponse> createRepeated() => $pb.PbList<RegisterPreKeyBundleResponse>();
  @$core.pragma('dart2js:noInline')
  static RegisterPreKeyBundleResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RegisterPreKeyBundleResponse>(create);
  static RegisterPreKeyBundleResponse? _defaultInstance;
}

class RequestPreKeysRequest extends $pb.GeneratedMessage {
  factory RequestPreKeysRequest({
    $core.String? identity,
  }) {
    final $result = create();
    if (identity != null) {
      $result.identity = identity;
    }
    return $result;
  }
  RequestPreKeysRequest._() : super();
  factory RequestPreKeysRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory RequestPreKeysRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RequestPreKeysRequest', package: const $pb.PackageName(_omitMessageNames ? '' : 'service'), createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'identity')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  RequestPreKeysRequest clone() => RequestPreKeysRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  RequestPreKeysRequest copyWith(void Function(RequestPreKeysRequest) updates) => super.copyWith((message) => updates(message as RequestPreKeysRequest)) as RequestPreKeysRequest;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RequestPreKeysRequest create() => RequestPreKeysRequest._();
  RequestPreKeysRequest createEmptyInstance() => create();
  static $pb.PbList<RequestPreKeysRequest> createRepeated() => $pb.PbList<RequestPreKeysRequest>();
  @$core.pragma('dart2js:noInline')
  static RequestPreKeysRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RequestPreKeysRequest>(create);
  static RequestPreKeysRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get identity => $_getSZ(0);
  @$pb.TagNumber(1)
  set identity($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasIdentity() => $_has(0);
  @$pb.TagNumber(1)
  void clearIdentity() => clearField(1);
}

class PreKeyBundle extends $pb.GeneratedMessage {
  factory PreKeyBundle({
    $core.List<$core.int>? identityKey,
    $core.List<$core.int>? otk,
    SignedPreKey? spk,
  }) {
    final $result = create();
    if (identityKey != null) {
      $result.identityKey = identityKey;
    }
    if (otk != null) {
      $result.otk = otk;
    }
    if (spk != null) {
      $result.spk = spk;
    }
    return $result;
  }
  PreKeyBundle._() : super();
  factory PreKeyBundle.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory PreKeyBundle.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'PreKeyBundle', package: const $pb.PackageName(_omitMessageNames ? '' : 'service'), createEmptyInstance: create)
    ..a<$core.List<$core.int>>(1, _omitFieldNames ? '' : 'identityKey', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(2, _omitFieldNames ? '' : 'otk', $pb.PbFieldType.OY)
    ..aOM<SignedPreKey>(3, _omitFieldNames ? '' : 'spk', subBuilder: SignedPreKey.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  PreKeyBundle clone() => PreKeyBundle()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  PreKeyBundle copyWith(void Function(PreKeyBundle) updates) => super.copyWith((message) => updates(message as PreKeyBundle)) as PreKeyBundle;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PreKeyBundle create() => PreKeyBundle._();
  PreKeyBundle createEmptyInstance() => create();
  static $pb.PbList<PreKeyBundle> createRepeated() => $pb.PbList<PreKeyBundle>();
  @$core.pragma('dart2js:noInline')
  static PreKeyBundle getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PreKeyBundle>(create);
  static PreKeyBundle? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get identityKey => $_getN(0);
  @$pb.TagNumber(1)
  set identityKey($core.List<$core.int> v) { $_setBytes(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasIdentityKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearIdentityKey() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get otk => $_getN(1);
  @$pb.TagNumber(2)
  set otk($core.List<$core.int> v) { $_setBytes(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasOtk() => $_has(1);
  @$pb.TagNumber(2)
  void clearOtk() => clearField(2);

  @$pb.TagNumber(3)
  SignedPreKey get spk => $_getN(2);
  @$pb.TagNumber(3)
  set spk(SignedPreKey v) { setField(3, v); }
  @$pb.TagNumber(3)
  $core.bool hasSpk() => $_has(2);
  @$pb.TagNumber(3)
  void clearSpk() => clearField(3);
  @$pb.TagNumber(3)
  SignedPreKey ensureSpk() => $_ensure(2);
}

class X3DHMessage extends $pb.GeneratedMessage {
  factory X3DHMessage({
    $core.List<$core.int>? senderIk,
    $core.List<$core.int>? ephemeralKey,
    $core.List<$core.int>? otk,
    $core.List<$core.int>? ciphertext,
  }) {
    final $result = create();
    if (senderIk != null) {
      $result.senderIk = senderIk;
    }
    if (ephemeralKey != null) {
      $result.ephemeralKey = ephemeralKey;
    }
    if (otk != null) {
      $result.otk = otk;
    }
    if (ciphertext != null) {
      $result.ciphertext = ciphertext;
    }
    return $result;
  }
  X3DHMessage._() : super();
  factory X3DHMessage.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory X3DHMessage.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'X3DHMessage', package: const $pb.PackageName(_omitMessageNames ? '' : 'service'), createEmptyInstance: create)
    ..a<$core.List<$core.int>>(1, _omitFieldNames ? '' : 'senderIk', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(2, _omitFieldNames ? '' : 'ephemeralKey', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(3, _omitFieldNames ? '' : 'otk', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(4, _omitFieldNames ? '' : 'ciphertext', $pb.PbFieldType.OY)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  X3DHMessage clone() => X3DHMessage()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  X3DHMessage copyWith(void Function(X3DHMessage) updates) => super.copyWith((message) => updates(message as X3DHMessage)) as X3DHMessage;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static X3DHMessage create() => X3DHMessage._();
  X3DHMessage createEmptyInstance() => create();
  static $pb.PbList<X3DHMessage> createRepeated() => $pb.PbList<X3DHMessage>();
  @$core.pragma('dart2js:noInline')
  static X3DHMessage getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<X3DHMessage>(create);
  static X3DHMessage? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get senderIk => $_getN(0);
  @$pb.TagNumber(1)
  set senderIk($core.List<$core.int> v) { $_setBytes(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasSenderIk() => $_has(0);
  @$pb.TagNumber(1)
  void clearSenderIk() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get ephemeralKey => $_getN(1);
  @$pb.TagNumber(2)
  set ephemeralKey($core.List<$core.int> v) { $_setBytes(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasEphemeralKey() => $_has(1);
  @$pb.TagNumber(2)
  void clearEphemeralKey() => clearField(2);

  @$pb.TagNumber(3)
  $core.List<$core.int> get otk => $_getN(2);
  @$pb.TagNumber(3)
  set otk($core.List<$core.int> v) { $_setBytes(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasOtk() => $_has(2);
  @$pb.TagNumber(3)
  void clearOtk() => clearField(3);

  @$pb.TagNumber(4)
  $core.List<$core.int> get ciphertext => $_getN(3);
  @$pb.TagNumber(4)
  set ciphertext($core.List<$core.int> v) { $_setBytes(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasCiphertext() => $_has(3);
  @$pb.TagNumber(4)
  void clearCiphertext() => clearField(4);
}

class SendMessageRequest extends $pb.GeneratedMessage {
  factory SendMessageRequest({
    $core.String? recipientIdentity,
    X3DHMessage? message,
  }) {
    final $result = create();
    if (recipientIdentity != null) {
      $result.recipientIdentity = recipientIdentity;
    }
    if (message != null) {
      $result.message = message;
    }
    return $result;
  }
  SendMessageRequest._() : super();
  factory SendMessageRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SendMessageRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SendMessageRequest', package: const $pb.PackageName(_omitMessageNames ? '' : 'service'), createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'recipientIdentity')
    ..aOM<X3DHMessage>(2, _omitFieldNames ? '' : 'message', subBuilder: X3DHMessage.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SendMessageRequest clone() => SendMessageRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SendMessageRequest copyWith(void Function(SendMessageRequest) updates) => super.copyWith((message) => updates(message as SendMessageRequest)) as SendMessageRequest;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SendMessageRequest create() => SendMessageRequest._();
  SendMessageRequest createEmptyInstance() => create();
  static $pb.PbList<SendMessageRequest> createRepeated() => $pb.PbList<SendMessageRequest>();
  @$core.pragma('dart2js:noInline')
  static SendMessageRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SendMessageRequest>(create);
  static SendMessageRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get recipientIdentity => $_getSZ(0);
  @$pb.TagNumber(1)
  set recipientIdentity($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasRecipientIdentity() => $_has(0);
  @$pb.TagNumber(1)
  void clearRecipientIdentity() => clearField(1);

  @$pb.TagNumber(2)
  X3DHMessage get message => $_getN(1);
  @$pb.TagNumber(2)
  set message(X3DHMessage v) { setField(2, v); }
  @$pb.TagNumber(2)
  $core.bool hasMessage() => $_has(1);
  @$pb.TagNumber(2)
  void clearMessage() => clearField(2);
  @$pb.TagNumber(2)
  X3DHMessage ensureMessage() => $_ensure(1);
}

class SendMessageResponse extends $pb.GeneratedMessage {
  factory SendMessageResponse() => create();
  SendMessageResponse._() : super();
  factory SendMessageResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SendMessageResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'SendMessageResponse', package: const $pb.PackageName(_omitMessageNames ? '' : 'service'), createEmptyInstance: create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SendMessageResponse clone() => SendMessageResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SendMessageResponse copyWith(void Function(SendMessageResponse) updates) => super.copyWith((message) => updates(message as SendMessageResponse)) as SendMessageResponse;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SendMessageResponse create() => SendMessageResponse._();
  SendMessageResponse createEmptyInstance() => create();
  static $pb.PbList<SendMessageResponse> createRepeated() => $pb.PbList<SendMessageResponse>();
  @$core.pragma('dart2js:noInline')
  static SendMessageResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SendMessageResponse>(create);
  static SendMessageResponse? _defaultInstance;
}

class RetrieveMessagesRequest extends $pb.GeneratedMessage {
  factory RetrieveMessagesRequest({
    $core.String? identity,
  }) {
    final $result = create();
    if (identity != null) {
      $result.identity = identity;
    }
    return $result;
  }
  RetrieveMessagesRequest._() : super();
  factory RetrieveMessagesRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory RetrieveMessagesRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RetrieveMessagesRequest', package: const $pb.PackageName(_omitMessageNames ? '' : 'service'), createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'identity')
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  RetrieveMessagesRequest clone() => RetrieveMessagesRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  RetrieveMessagesRequest copyWith(void Function(RetrieveMessagesRequest) updates) => super.copyWith((message) => updates(message as RetrieveMessagesRequest)) as RetrieveMessagesRequest;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RetrieveMessagesRequest create() => RetrieveMessagesRequest._();
  RetrieveMessagesRequest createEmptyInstance() => create();
  static $pb.PbList<RetrieveMessagesRequest> createRepeated() => $pb.PbList<RetrieveMessagesRequest>();
  @$core.pragma('dart2js:noInline')
  static RetrieveMessagesRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RetrieveMessagesRequest>(create);
  static RetrieveMessagesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get identity => $_getSZ(0);
  @$pb.TagNumber(1)
  set identity($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasIdentity() => $_has(0);
  @$pb.TagNumber(1)
  void clearIdentity() => clearField(1);
}

class RetrieveMessagesResponse extends $pb.GeneratedMessage {
  factory RetrieveMessagesResponse({
    $core.Iterable<X3DHMessage>? messages,
  }) {
    final $result = create();
    if (messages != null) {
      $result.messages.addAll(messages);
    }
    return $result;
  }
  RetrieveMessagesResponse._() : super();
  factory RetrieveMessagesResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory RetrieveMessagesResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(_omitMessageNames ? '' : 'RetrieveMessagesResponse', package: const $pb.PackageName(_omitMessageNames ? '' : 'service'), createEmptyInstance: create)
    ..pc<X3DHMessage>(1, _omitFieldNames ? '' : 'messages', $pb.PbFieldType.PM, subBuilder: X3DHMessage.create)
    ..hasRequiredFields = false
  ;

  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  RetrieveMessagesResponse clone() => RetrieveMessagesResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  RetrieveMessagesResponse copyWith(void Function(RetrieveMessagesResponse) updates) => super.copyWith((message) => updates(message as RetrieveMessagesResponse)) as RetrieveMessagesResponse;

  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RetrieveMessagesResponse create() => RetrieveMessagesResponse._();
  RetrieveMessagesResponse createEmptyInstance() => create();
  static $pb.PbList<RetrieveMessagesResponse> createRepeated() => $pb.PbList<RetrieveMessagesResponse>();
  @$core.pragma('dart2js:noInline')
  static RetrieveMessagesResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RetrieveMessagesResponse>(create);
  static RetrieveMessagesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<X3DHMessage> get messages => $_getList(0);
}


const _omitFieldNames = $core.bool.fromEnvironment('protobuf.omit_field_names');
const _omitMessageNames = $core.bool.fromEnvironment('protobuf.omit_message_names');
