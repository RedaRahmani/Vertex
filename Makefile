SHELL := /bin/bash

.PHONY: build test ui sdk devnet-deploy demo gate

build:
	anchor build

test:
	cargo test -p keystone-fee-router -- --nocapture

ui:
	npm -w apps/ui install
	npm -w apps/ui run build

sdk:
	npm -w sdk/ts run build
	npm -w sdk/ts run pack

devnet-deploy:
	./scripts/deploy_anchor.sh

demo:
	solana-test-validator --reset --limit-ledger-size 4096 > /tmp/validator.log 2>&1 &
	sleep 5
	anchor deploy
	npm -w apps/ui run dev

gate:
	bash scripts/acceptance_gate.sh
