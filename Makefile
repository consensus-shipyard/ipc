NETWORK ?= localnet

deploy-ipc:
	./ops/deploy.sh $(NETWORK)

.PHONY: deploy-ipc