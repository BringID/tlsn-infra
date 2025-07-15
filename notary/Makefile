setup:
	openssl genpkey -algorithm EC -out eckey.pem -pkeyopt ec_paramgen_curve:secp256k1 -pkeyopt ec_param_enc:named_curve

dev:
	docker run \
		--init \
		-p 127.0.0.1:7047:7047 \
		-v ./config.yaml:/root/.notary/config.yaml \
		-v ./whitelist.dev.csv:/root/.notary/whitelist.csv \
		-v ./eckey.dev.pem:/root/.notary/eckey.pem \
		ghcr.io/tlsnotary/tlsn/notary-server:v0.1.0-alpha.12 \
		--config /root/.notary/config.yaml

run:
	docker run \
		--init \
		-p 127.0.0.1:7047:7047 \
		-v ./config.yaml:/root/.notary/config.yaml \
		-v ./whitelist.csv:/root/.notary/whitelist.csv \
		-v ./eckey.pem:/root/.notary/eckey.pem \
		ghcr.io/tlsnotary/tlsn/notary-server:v0.1.0-alpha.12 \
		--config /root/.notary/config.yaml