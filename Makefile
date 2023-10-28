
token:
	echo -n $$(tr -dc A-Za-z0-9 </dev/urandom | head -c 50 ; echo '') > keys/token.txt
	cp keys/token.txt ./container/keys
	@# echo $(tr -dc A-Za-z0-9 </dev/urandom | head -c 50 ; echo '') > ./container/token.txt

key:
	rm -f keys/*.crt keys/*.srl keys/*.key keys/*.pem keys/*.csr
	openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
		-keyout keys/ca.key -out keys/ca.crt -config keys/ca_req.conf
	openssl genpkey -algorithm RSA -out keys/server.key
	openssl req -new -key keys/server.key -out keys/server.csr \
		-config keys/server_req.conf
	openssl x509 -req -in keys/server.csr -CA keys/ca.crt -CAkey keys/ca.key \
		-CAcreateserial -out keys/server.crt -days 365 -extfile keys/v3_end.cnf \
		-extensions v3_end
	cat keys/server.crt keys/server.key > keys/server.pem
	cp keys/server.pem ./container/keys
	cp keys/ca.crt ./container/keys

build:
	podman build -f container/build.containerfile -t sinnergasm/serve-build
	podman run --rm \
		-v /work/ProjectsForFun/rust-synergy/seperate:/build:ro \
		-v /work/ProjectsForFun/rust-synergy/seperate/container/target/:/build/target:rw \
		sinnergasm/serve-build
	# strip -v target/release/serve
	podman build -f container/serve.containerfile -t sinnergasm/serve container

bump_version:
	SINNERGY_SERVE_VERSION=$$(tar c simulator/ controller/ server/ common/ | sha1sum | sed 's/ .*//') && \
	echo -n $$SINNERGY_SERVE_VERSION > ./container/version.txt
	echo -n $$(tr -dc A-Za-z0-9 </dev/urandom | head -c 10 ; echo '') >> ./container/version.txt
	cat ./container/version.txt
	echo -n "0.0.2" > ./container/version.txt

	# echo "1.2.3" | awk -F. '{$NF++; print $1"."$2"."$NF}'


push_image: bump_version build
	aws ecr get-login-password --region us-west-1 | \
		podman login --username AWS --password-stdin 080899586278.dkr.ecr.us-west-1.amazonaws.com
	podman tag sinnergasm/serve 080899586278.dkr.ecr.us-west-1.amazonaws.com/sinnergy-serve:$$(cat ./container/version.txt)
	podman push 080899586278.dkr.ecr.us-west-1.amazonaws.com/sinnergy-serve:$$(cat ./container/version.txt)
	echo 080899586278.dkr.ecr.us-west-1.amazonaws.com/sinnergy-serve:$$(cat ./container/version.txt)

run:
  # podman run -d -p 50051:50051 --rm 080899586278.dkr.ecr.us-west-1.amazonaws.com/sinnergy-serve:latest
	podman run -p 50051:50051 --rm sinnergasm/serve # -d 

stop:
	podman container kill $$(podman ps -a | grep sinnergasm/serve | awk '{print $$1}')

restart:
	podman restart $$(podman ps -a | grep sinnergasm/serve | awk '{print $$1}')

# push_image: build
# 	export SINNERGY_SERVE_VERSION=$(tar c simulator/ controller/ server/ common/ | sha1sum | sed 's/ .*//')
# 	echo $SINNERGY_SERVE_VERSION > ./container/version.txt
# 	aws ecr get-login-password --region us-west-1 | \
# 		podman login --username AWS --password-stdin \
# 			080899586278.dkr.ecr.us-west-1.amazonaws.com
# 	podman tag sinnergasm/serve \
# 		080899586278.dkr.ecr.us-west-1.amazonaws.com/sinnergy-serve:$SINNERGY_SERVE_VERSION
# 	podman push \
# 		080899586278.dkr.ecr.us-west-1.amazonaws.com/sinnergy-serve:$SINNERGY_SERVE_VERSION

# restart:
# 	podman restart $(shell podman ps -a | grep sinnergasm/serve | awk '{print $$1}') 


