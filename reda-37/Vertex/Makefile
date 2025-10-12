SHELL := /bin/bash

.PHONY: build test ui sdk devnet-deploy demo gate ts-test

build:
	PATH="$(PWD)/scripts/bin:$$PATH" anchor build | tee /tmp/vertex_anchor_build_final.log

test:
	cargo test -p keystone-fee-router -- --nocapture | tee /tmp/vertex_fee_router_rust_tests.log

ui:
	@[ ! -d apps/ui/node_modules ] || rm -r apps/ui/node_modules
	@[ ! -f apps/ui/package-lock.json ] || rm apps/ui/package-lock.json
	npm --prefix apps/ui install | tee /tmp/vertex_ui_install.log
	npm --prefix apps/ui run build | tee /tmp/vertex_ui_build.log
	tail -30 /tmp/vertex_ui_build.log > /tmp/vertex_ui_build_tail.log

sdk:
	npm --prefix sdk/ts run build | tee /tmp/vertex_sdk_build.log
	npm --prefix sdk/ts run pack | tee /tmp/vertex_sdk_pack.log
	ls -1 sdk/ts/keystone-sdk-*.tgz | tail -n 1 > /tmp/vertex_sdk_tarball.txt

ts-test:
	npm -w tests/ts run test | tee /tmp/vertex_ts_tests.log

devnet-deploy:
	./scripts/deploy_anchor.sh

demo:
	solana-test-validator --reset --limit-ledger-size 4096 > /tmp/validator.log 2>&1 &
	sleep 5
	anchor deploy
	npm --prefix apps/ui run dev

gate:
	bash scripts/acceptance_gate.sh | tee /tmp/vertex_gate.log
