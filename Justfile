# print recipes
_default:
	just --list

# build server and apk
build:
	nix build .#myServer .#apk

# nix build and pipe into podman
container:
	nix build .#dockerImage
	./result | podman load

# linters!
format:
	nix run .#format

test: build
	nix run .#test

# run this before pushing a commit!
precommit: format test build container 
	
# push server to fly.io
deploy: container
	podman push brongnal docker://registry.fly.io/brongnal:latest
	flyctl deploy -i registry.fly.io/brongnal:latest

# generate flutter_rust_bridge bindings
codegen:
	nix run .#codegen
