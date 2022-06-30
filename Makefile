######################################################################
# @author      : Enno Boland (mail@eboland.de)
# @file        : Makefile
# @created     : Wednesday Jun 29, 2022 07:58:39 CEST
######################################################################

REGISTRY=127.0.0.1:5000
OPERATOR_NAME=NONE

.PHONY: build push all

all: build

build:
	mkdir -p /tmp/cargo_cache
	podman build \
		--build-arg OPERATOR_NAME=$(OPERATOR_NAME) \
		-t $(REGISTRY)/$(OPERATOR_NAME) \
		-v /tmp/cargo_cache:/cargo .

push: build
	podman push $(REGISTRY)/$(OPERATOR_NAME)

deploy:
	nk helm upgrade -n dev-operators \
		--install \
		--create-namespace \
		--reset-values \
		--set image.repository=$(REGISTRY)/$(OPERATOR_NAME) \
		--set image.tag=latest \
		--set image.pullPolicy=Always \
		$(OPERATOR_NAME) charts/$(OPERATOR_NAME)

undeploy:
	nk helm delete -n dev-operators $(OPERATOR_NAME)
