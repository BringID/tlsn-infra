build:
	docker build -t proxy-service .

release:
	docker build --platform=linux/amd64 -t ghcr.io/bringid/tlsn/proxy:latest .
	docker push ghcr.io/bringid/tlsn/proxy:latest

run:
	docker run \
		--rm \
		-p 3003:80 \
		proxy-service

dev:
	docker run \
		--rm \
		-p 3003:80 \
		-v ./config:/app/websockify_config \
		proxy-service