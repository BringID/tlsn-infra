build:
	docker build -t proxy-service .

build-prod:
	docker build --platform=linux/amd64 -t proxy-service-prod .

run:
	docker run \
		--rm \
		-p 55688:80 \
		proxy-service

dev:
	docker run \
		--rm \
		-it \
		-p 55688:80 \
		-v ./config:/app/websockify_config \
		-v ./requirements.txt:/root/requirements.txt \
		proxy-service