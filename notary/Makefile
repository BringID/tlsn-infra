build:
	docker build -t notary-service .

setup:
	openssl genpkey -algorithm EC -out eckey.pem -pkeyopt ec_paramgen_curve:secp256k1 -pkeyopt ec_param_enc:named_curve

setup-dev:
	openssl genpkey -algorithm EC -out eckey.dev.pem -pkeyopt ec_paramgen_curve:secp256k1 -pkeyopt ec_param_enc:named_curve

run:
	docker run \
		--init \
		-p 127.0.0.1:7047:7047 \
		-v ./config.yaml:/root/.notary/config.yaml \
		-v ./eckey.pem:/root/.notary/eckey.pem \
		ghcr.io/tlsnotary/tlsn/notary-server:v0.1.0-alpha.12 \
		--config /root/.notary/config.yaml

dev:
	docker run \
		--init \
		-p 127.0.0.1:7047:7047 \
		-v ./config.dev.yaml:/root/.notary/config.yaml \
		-v ./eckey.dev.pem:/root/.notary/eckey.pem \
		ghcr.io/tlsnotary/tlsn/notary-server:v0.1.0-alpha.12 \
		--config /root/.notary/config.yaml