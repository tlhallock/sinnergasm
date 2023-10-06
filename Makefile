


build:
  podman build -f container/build.containerfile -t sinnergasm/serve-build container
	podman run --rm \
		-v /work/ProjectsForFun/rust-synergy/seperate:/build:ro \
		-v /work/ProjectsForFun/rust-synergy/seperate/container/target/:/build/target:rw \
		sinnergasm/serve-build
	podman build -f container/serve.containerfile -t sinnergasm/serve container

run:
	podman run -d -p 50051:50051 --rm sinnergasm/serve

restart:
	podman restart $(shell podman ps -a | grep sinnergasm/serve | awk '{print $$1}') 