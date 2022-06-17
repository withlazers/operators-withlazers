FROM clux/muslrust:1.60.0 AS build
ARG OPERATOR_NAME=none

WORKDIR /src

COPY . .

WORKDIR ${OPERATOR_NAME}

RUN cargo install \
		--path . \
		--root /target \
		--bin ${OPERATOR_NAME}

WORKDIR /app

RUN cp /target/bin/${OPERATOR_NAME} . && \
	ln -s ${OPERATOR_NAME} operator



FROM gcr.io/distroless/static:nonroot
ARG OPERATOR_NAME=none

COPY --chown=nonroot:nonroot --from=build \
	/app /app

EXPOSE 8080

ENTRYPOINT ["/app/operator"]
