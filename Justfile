_default:
  just --list

container:
	nix build .#dockerImage
	./result | podman load

deploy: container
  podman push brongnal docker://registry.fly.io/brongnal:latest
  flyctl deploy -i registry.fly.io/brongnal:latest

build:
  cargo b --all
  just container
  flutter build

precommit: container
  dart analyze flutter_package --fatal-infos
  dart format .
  cargo fmt
  cargo clippy --fix --allow-dirty
  flutter build apk

