-include .env
export

.PHONY: dev-api dev-cms dev-player dev-db build lint fmt fmt-check test check clean

dev-db:
	podman compose up -d postgres

dev-api: dev-db
	cargo run -p civico-api

dev-cms:
	cd apps/cms-web && npm run dev

dev-player:
	cd apps/player-web && npm run dev

build:
	cargo build --workspace
	cd apps/cms-web && npm run build
	cd apps/player-web && npm run build

lint:
	cargo clippy --workspace -- -D warnings
	cd apps/cms-web && npm run lint
	cd apps/player-web && npm run lint

fmt:
	cargo fmt --all
	cd apps/cms-web && npm run format
	cd apps/player-web && npm run format

fmt-check:
	cargo fmt --all -- --check
	cd apps/cms-web && npm run format:check
	cd apps/player-web && npm run format:check

test:
	cargo test --workspace

check: fmt-check lint test
	@echo "All checks passed."

clean:
	cargo clean
	cd apps/cms-web && rm -rf dist node_modules
	cd apps/player-web && rm -rf dist node_modules
