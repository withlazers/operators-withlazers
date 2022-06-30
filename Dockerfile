FROM clux/muslrust:1.60.0 AS build
ARG OPERATOR_NAME=none

WORKDIR /src

COPY . .

WORKDIR ${OPERATOR_NAME}

RUN \
	mkdir -p /cargo/cargo && \
	ln -sf $HOME/.cargo/config /cargo/cargo && \
	CARGO_HOME=/cargo/cargo \
	CARGO_TARGET_DIR=/cargo/target \
	cargo install \
		--path . \
		--root /app \
		--bin operator

FROM gcr.io/distroless/static:nonroot

COPY --from=build /app/bin/operator /

EXPOSE 8080

ENTRYPOINT ["/operator"]
