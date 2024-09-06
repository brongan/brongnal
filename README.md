# Brongnal

I took [Going Bark: A Furry’s Guide to End-to-End Encryption](https://soatok.blog/2020/11/14/going-bark-a-furrys-guide-to-end-to-end-encryption/) as a great idea for a project.

X3DH and Double-Ratchet are implemented in Rust in the [protocol](./native/protocol/) directory.

## Warning
DO NOT USE THIS.

ASSUME IT IS INSECURE.

USE [Signal](https://signal.org/) instead.

This is for fun and learning :)

## Instructions

### App

To run and build this app, you need to have installed:
* [Rust toolchain](https://www.rust-lang.org/tools/install)
* [Flutter SDK](https://docs.flutter.dev/get-started/install)
* [rinf](https://rinf.cunarist.com/)
* [protoc](https://grpc.io/docs/protoc-installation/)

You can check that your system is ready with the commands below.
Note that all the Flutter subcomponents should be installed.

```bash
rustc --version
flutter doctor
cargo install rinf
```

Generated schema for messages between Dart and Rust are not commited and must be recreated.
If you have newly cloned the project repository
or made changes to the `.proto` files in the `./messages` directory,
run the following command:

```bash
rinf message
```

Now you can run and build this app just like any other Flutter projects.

```bash
flutter run
```

### Backend

To run and build the backend, you need to have installed:
* [Rust toolchain](https://www.rust-lang.org/tools/install)
* [protoc](https://grpc.io/docs/protoc-installation/)

```bash
cargo r -p server
```

### Client

```bash
cargo r -p client $USER http://localhost:8080
```

### Server Release

For me to install the server,
* [flyctl](https://fly.io/docs/hands-on/install-flyctl/)
* [just](https://github.com/casey/just)
* [nix](https://nixos.org/download/) 

I use [Justfile](./Justfile) for hard to remember commands.

```bash
just deploy
```

Deploys the server to signal.brongan.com


[grpcurl](https://github.com/fullstorydev/grpcurl) is a cool way to see the exposed rpcs from the [proto directory](./native/server/proto/].
```bash
grpcurl signal.brongan.com:443 describe
```

