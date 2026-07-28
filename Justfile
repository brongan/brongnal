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

# build a signed release AAB for Play; upload key is decrypted to tmpfs
# (touch YubiKey twice), used, then wiped. versionCode is the build
# timestamp, so uploads never collide.
release:
	#!/usr/bin/env bash
	set -euo pipefail
	aab=build/app/outputs/bundle/release/app-release.aab
	# build unsigned (no YubiKey prompt), then sign with the upload key held
	# on the YubiKey via PKCS#11 -- enter PIN, then touch.
	flutter build appbundle --release --build-number="$(date +%s)"
	cfg="$(mktemp)"; trap 'rm -f "$cfg"' EXIT
	printf 'name = YubiKeyPIV\nlibrary = %s\n' \
		"$(dirname "$(command -v yubico-piv-tool)")/../lib/libykcs11.so" > "$cfg"
	jarsigner -keystore NONE -storetype PKCS11 \
		-providerClass sun.security.pkcs11.SunPKCS11 -providerArg "$cfg" \
		"$aab" "X.509 Certificate for Digital Signature"
	echo "signed $aab"
