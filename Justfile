# print recipes
_default:
	just --list

# build server and apk
build:
	cargo b --all
	flutter build apk

# nix build and pipe into podman
container:
	nix build .#dockerImage
	./result | podman load

# linters!
format:
	dart analyze --fatal-infos
	dart format .
	cargo fmt
	cargo clippy --fix --allow-dirty

test:
	cargo t


# run this before pushing a commit!
precommit: format test build container 
	
# push server to fly.io
deploy: container
	podman push brongnal docker://registry.fly.io/brongnal:latest
	flyctl deploy -i registry.fly.io/brongnal:latest

