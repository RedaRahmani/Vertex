SHELL := /bin/bash

.PHONY: build test ui devnet-deploy demo gate

build:
	anchor build

test:
	anchor test --skip-build

ui:
	cd apps/ui && pnpm i && pnpm dev

devnet-deploy:
	./scripts/deploy_anchor.sh

demo:
	./scripts/localnet.sh

gate:
	bash scripts/acceptance_gate.sh

