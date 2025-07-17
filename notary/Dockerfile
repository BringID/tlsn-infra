FROM alpine:latest as key-generator

RUN apk add --no-cache openssl

WORKDIR /keys

RUN openssl genpkey -algorithm EC -out eckey.pem -pkeyopt ec_paramgen_curve:secp256k1 -pkeyopt ec_param_enc:named_curve

FROM ghcr.io/tlsnotary/tlsn/notary-server:v0.1.0-alpha.12

COPY --from=key-generator /keys/eckey.pem /root/.notary/eckey.pem

COPY config.yaml /root/.notary/config.yaml

EXPOSE 7047

CMD ["--config", "/root/.notary/config.yaml"]