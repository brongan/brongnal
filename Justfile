_default:
	just --list

container:
	nix build .#dockerImage
	./result | podman load

build:
	cargo b --all
	flutter build apk

precommit: build container
	cargo t
	dart analyze --fatal-infos
	dart format .
	cargo fmt
	cargo clippy --fix --allow-dirty

deploy: container
	podman push brongnal docker://registry.fly.io/brongnal:latest
	flyctl deploy -i registry.fly.io/brongnal:latest

