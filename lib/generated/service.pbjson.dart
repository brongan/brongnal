//
//  Generated code. Do not modify.
//  source: service.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use signedPreKeyDescriptor instead')
const SignedPreKey$json = {
  '1': 'SignedPreKey',
  '2': [
    {'1': 'pre_key', '3': 1, '4': 1, '5': 12, '10': 'preKey'},
    {'1': 'signature', '3': 2, '4': 1, '5': 12, '10': 'signature'},
  ],
};

/// Descriptor for `SignedPreKey`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List signedPreKeyDescriptor = $convert.base64Decode(
    'CgxTaWduZWRQcmVLZXkSFwoHcHJlX2tleRgBIAEoDFIGcHJlS2V5EhwKCXNpZ25hdHVyZRgCIA'
    'EoDFIJc2lnbmF0dXJl');

@$core.Deprecated('Use signedPreKeysDescriptor instead')
const SignedPreKeys$json = {
  '1': 'SignedPreKeys',
  '2': [
    {'1': 'pre_keys', '3': 1, '4': 3, '5': 12, '10': 'preKeys'},
    {'1': 'signature', '3': 2, '4': 1, '5': 12, '10': 'signature'},
  ],
};

/// Descriptor for `SignedPreKeys`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List signedPreKeysDescriptor = $convert.base64Decode(
    'Cg1TaWduZWRQcmVLZXlzEhkKCHByZV9rZXlzGAEgAygMUgdwcmVLZXlzEhwKCXNpZ25hdHVyZR'
    'gCIAEoDFIJc2lnbmF0dXJl');

@$core.Deprecated('Use registerPreKeyBundleRequestDescriptor instead')
const RegisterPreKeyBundleRequest$json = {
  '1': 'RegisterPreKeyBundleRequest',
  '2': [
    {'1': 'identity', '3': 1, '4': 1, '5': 9, '10': 'identity'},
    {'1': 'ik', '3': 2, '4': 1, '5': 12, '10': 'ik'},
    {
      '1': 'spk',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.service.SignedPreKey',
      '10': 'spk'
    },
    {
      '1': 'otk_bundle',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.service.SignedPreKeys',
      '10': 'otkBundle'
    },
  ],
};

/// Descriptor for `RegisterPreKeyBundleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerPreKeyBundleRequestDescriptor = $convert.base64Decode(
    'ChtSZWdpc3RlclByZUtleUJ1bmRsZVJlcXVlc3QSGgoIaWRlbnRpdHkYASABKAlSCGlkZW50aX'
    'R5Eg4KAmlrGAIgASgMUgJpaxInCgNzcGsYAyABKAsyFS5zZXJ2aWNlLlNpZ25lZFByZUtleVID'
    'c3BrEjUKCm90a19idW5kbGUYBCABKAsyFi5zZXJ2aWNlLlNpZ25lZFByZUtleXNSCW90a0J1bm'
    'RsZQ==');

@$core.Deprecated('Use registerPreKeyBundleResponseDescriptor instead')
const RegisterPreKeyBundleResponse$json = {
  '1': 'RegisterPreKeyBundleResponse',
};

/// Descriptor for `RegisterPreKeyBundleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerPreKeyBundleResponseDescriptor =
    $convert.base64Decode('ChxSZWdpc3RlclByZUtleUJ1bmRsZVJlc3BvbnNl');

@$core.Deprecated('Use requestPreKeysRequestDescriptor instead')
const RequestPreKeysRequest$json = {
  '1': 'RequestPreKeysRequest',
  '2': [
    {'1': 'identity', '3': 1, '4': 1, '5': 9, '10': 'identity'},
  ],
};

/// Descriptor for `RequestPreKeysRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List requestPreKeysRequestDescriptor =
    $convert.base64Decode(
        'ChVSZXF1ZXN0UHJlS2V5c1JlcXVlc3QSGgoIaWRlbnRpdHkYASABKAlSCGlkZW50aXR5');

@$core.Deprecated('Use preKeyBundleDescriptor instead')
const PreKeyBundle$json = {
  '1': 'PreKeyBundle',
  '2': [
    {'1': 'identity_key', '3': 1, '4': 1, '5': 12, '10': 'identityKey'},
    {'1': 'otk', '3': 2, '4': 1, '5': 12, '10': 'otk'},
    {
      '1': 'spk',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.service.SignedPreKey',
      '10': 'spk'
    },
  ],
};

/// Descriptor for `PreKeyBundle`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List preKeyBundleDescriptor = $convert.base64Decode(
    'CgxQcmVLZXlCdW5kbGUSIQoMaWRlbnRpdHlfa2V5GAEgASgMUgtpZGVudGl0eUtleRIQCgNvdG'
    'sYAiABKAxSA290axInCgNzcGsYAyABKAsyFS5zZXJ2aWNlLlNpZ25lZFByZUtleVIDc3Br');

@$core.Deprecated('Use x3DHMessageDescriptor instead')
const X3DHMessage$json = {
  '1': 'X3DHMessage',
  '2': [
    {'1': 'sender_ik', '3': 1, '4': 1, '5': 12, '10': 'senderIk'},
    {'1': 'ephemeral_key', '3': 2, '4': 1, '5': 12, '10': 'ephemeralKey'},
    {'1': 'otk', '3': 3, '4': 1, '5': 12, '10': 'otk'},
    {'1': 'ciphertext', '3': 4, '4': 1, '5': 12, '10': 'ciphertext'},
  ],
};

/// Descriptor for `X3DHMessage`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List x3DHMessageDescriptor = $convert.base64Decode(
    'CgtYM0RITWVzc2FnZRIbCglzZW5kZXJfaWsYASABKAxSCHNlbmRlcklrEiMKDWVwaGVtZXJhbF'
    '9rZXkYAiABKAxSDGVwaGVtZXJhbEtleRIQCgNvdGsYAyABKAxSA290axIeCgpjaXBoZXJ0ZXh0'
    'GAQgASgMUgpjaXBoZXJ0ZXh0');

@$core.Deprecated('Use sendMessageRequestDescriptor instead')
const SendMessageRequest$json = {
  '1': 'SendMessageRequest',
  '2': [
    {
      '1': 'recipient_identity',
      '3': 1,
      '4': 1,
      '5': 9,
      '10': 'recipientIdentity'
    },
    {
      '1': 'message',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.service.X3DHMessage',
      '10': 'message'
    },
  ],
};

/// Descriptor for `SendMessageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sendMessageRequestDescriptor = $convert.base64Decode(
    'ChJTZW5kTWVzc2FnZVJlcXVlc3QSLQoScmVjaXBpZW50X2lkZW50aXR5GAEgASgJUhFyZWNpcG'
    'llbnRJZGVudGl0eRIuCgdtZXNzYWdlGAIgASgLMhQuc2VydmljZS5YM0RITWVzc2FnZVIHbWVz'
    'c2FnZQ==');

@$core.Deprecated('Use sendMessageResponseDescriptor instead')
const SendMessageResponse$json = {
  '1': 'SendMessageResponse',
};

/// Descriptor for `SendMessageResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sendMessageResponseDescriptor =
    $convert.base64Decode('ChNTZW5kTWVzc2FnZVJlc3BvbnNl');

@$core.Deprecated('Use retrieveMessagesRequestDescriptor instead')
const RetrieveMessagesRequest$json = {
  '1': 'RetrieveMessagesRequest',
  '2': [
    {'1': 'identity', '3': 1, '4': 1, '5': 9, '10': 'identity'},
  ],
};

/// Descriptor for `RetrieveMessagesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List retrieveMessagesRequestDescriptor =
    $convert.base64Decode(
        'ChdSZXRyaWV2ZU1lc3NhZ2VzUmVxdWVzdBIaCghpZGVudGl0eRgBIAEoCVIIaWRlbnRpdHk=');

@$core.Deprecated('Use retrieveMessagesResponseDescriptor instead')
const RetrieveMessagesResponse$json = {
  '1': 'RetrieveMessagesResponse',
  '2': [
    {
      '1': 'messages',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.service.X3DHMessage',
      '10': 'messages'
    },
  ],
};

/// Descriptor for `RetrieveMessagesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List retrieveMessagesResponseDescriptor =
    $convert.base64Decode(
        'ChhSZXRyaWV2ZU1lc3NhZ2VzUmVzcG9uc2USMAoIbWVzc2FnZXMYASADKAsyFC5zZXJ2aWNlLl'
        'gzREhNZXNzYWdlUghtZXNzYWdlcw==');
