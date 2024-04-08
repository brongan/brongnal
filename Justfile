_default:
  just --list

container:
	nix build .#dockerImage
	./result | podman load

deploy: container
  podman push brongnal docker://registry.fly.io/brongnal:latest
  flyctl deploy -i registry.fly.io/brongnal:latest

