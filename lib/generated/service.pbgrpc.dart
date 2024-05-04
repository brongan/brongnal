//
//  Generated code. Do not modify.
//  source: service.proto
//
// @dart = 2.12

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_final_fields
// ignore_for_file: unnecessary_import, unnecessary_this, unused_import

import 'dart:async' as $async;
import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
import 'package:protobuf/protobuf.dart' as $pb;

import 'service.pb.dart' as $0;

export 'service.pb.dart';

@$pb.GrpcServiceName('service.Brongnal')
class BrongnalClient extends $grpc.Client {
  static final _$registerPreKeyBundle = $grpc.ClientMethod<$0.RegisterPreKeyBundleRequest, $0.RegisterPreKeyBundleResponse>(
      '/service.Brongnal/RegisterPreKeyBundle',
      ($0.RegisterPreKeyBundleRequest value) => value.writeToBuffer(),
      ($core.List<$core.int> value) => $0.RegisterPreKeyBundleResponse.fromBuffer(value));
  static final _$requestPreKeys = $grpc.ClientMethod<$0.RequestPreKeysRequest, $0.PreKeyBundle>(
      '/service.Brongnal/RequestPreKeys',
      ($0.RequestPreKeysRequest value) => value.writeToBuffer(),
      ($core.List<$core.int> value) => $0.PreKeyBundle.fromBuffer(value));
  static final _$sendMessage = $grpc.ClientMethod<$0.SendMessageRequest, $0.SendMessageResponse>(
      '/service.Brongnal/SendMessage',
      ($0.SendMessageRequest value) => value.writeToBuffer(),
      ($core.List<$core.int> value) => $0.SendMessageResponse.fromBuffer(value));
  static final _$retrieveMessages = $grpc.ClientMethod<$0.RetrieveMessagesRequest, $0.RetrieveMessagesResponse>(
      '/service.Brongnal/RetrieveMessages',
      ($0.RetrieveMessagesRequest value) => value.writeToBuffer(),
      ($core.List<$core.int> value) => $0.RetrieveMessagesResponse.fromBuffer(value));

  BrongnalClient($grpc.ClientChannel channel,
      {$grpc.CallOptions? options,
      $core.Iterable<$grpc.ClientInterceptor>? interceptors})
      : super(channel, options: options,
        interceptors: interceptors);

  $grpc.ResponseFuture<$0.RegisterPreKeyBundleResponse> registerPreKeyBundle($0.RegisterPreKeyBundleRequest request, {$grpc.CallOptions? options}) {
    return $createUnaryCall(_$registerPreKeyBundle, request, options: options);
  }

  $grpc.ResponseFuture<$0.PreKeyBundle> requestPreKeys($0.RequestPreKeysRequest request, {$grpc.CallOptions? options}) {
    return $createUnaryCall(_$requestPreKeys, request, options: options);
  }

  $grpc.ResponseFuture<$0.SendMessageResponse> sendMessage($0.SendMessageRequest request, {$grpc.CallOptions? options}) {
    return $createUnaryCall(_$sendMessage, request, options: options);
  }

  $grpc.ResponseFuture<$0.RetrieveMessagesResponse> retrieveMessages($0.RetrieveMessagesRequest request, {$grpc.CallOptions? options}) {
    return $createUnaryCall(_$retrieveMessages, request, options: options);
  }
}

@$pb.GrpcServiceName('service.Brongnal')
abstract class BrongnalServiceBase extends $grpc.Service {
  $core.String get $name => 'service.Brongnal';

  BrongnalServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.RegisterPreKeyBundleRequest, $0.RegisterPreKeyBundleResponse>(
        'RegisterPreKeyBundle',
        registerPreKeyBundle_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.RegisterPreKeyBundleRequest.fromBuffer(value),
        ($0.RegisterPreKeyBundleResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RequestPreKeysRequest, $0.PreKeyBundle>(
        'RequestPreKeys',
        requestPreKeys_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.RequestPreKeysRequest.fromBuffer(value),
        ($0.PreKeyBundle value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SendMessageRequest, $0.SendMessageResponse>(
        'SendMessage',
        sendMessage_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.SendMessageRequest.fromBuffer(value),
        ($0.SendMessageResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RetrieveMessagesRequest, $0.RetrieveMessagesResponse>(
        'RetrieveMessages',
        retrieveMessages_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.RetrieveMessagesRequest.fromBuffer(value),
        ($0.RetrieveMessagesResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.RegisterPreKeyBundleResponse> registerPreKeyBundle_Pre($grpc.ServiceCall call, $async.Future<$0.RegisterPreKeyBundleRequest> request) async {
    return registerPreKeyBundle(call, await request);
  }

  $async.Future<$0.PreKeyBundle> requestPreKeys_Pre($grpc.ServiceCall call, $async.Future<$0.RequestPreKeysRequest> request) async {
    return requestPreKeys(call, await request);
  }

  $async.Future<$0.SendMessageResponse> sendMessage_Pre($grpc.ServiceCall call, $async.Future<$0.SendMessageRequest> request) async {
    return sendMessage(call, await request);
  }

  $async.Future<$0.RetrieveMessagesResponse> retrieveMessages_Pre($grpc.ServiceCall call, $async.Future<$0.RetrieveMessagesRequest> request) async {
    return retrieveMessages(call, await request);
  }

  $async.Future<$0.RegisterPreKeyBundleResponse> registerPreKeyBundle($grpc.ServiceCall call, $0.RegisterPreKeyBundleRequest request);
  $async.Future<$0.PreKeyBundle> requestPreKeys($grpc.ServiceCall call, $0.RequestPreKeysRequest request);
  $async.Future<$0.SendMessageResponse> sendMessage($grpc.ServiceCall call, $0.SendMessageRequest request);
  $async.Future<$0.RetrieveMessagesResponse> retrieveMessages($grpc.ServiceCall call, $0.RetrieveMessagesRequest request);
}
