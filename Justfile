default:
	@just --list

install:
	pnpm install

dev:
	@if [ ! -d node_modules ]; then just install; fi
	@-pkill -x cortado 2>/dev/null
	pnpm exec tauri dev --config src-tauri/tauri.dev.conf.json

lint:
	@if [ ! -d node_modules ]; then just install; fi
	pnpm exec tsc --noEmit
	cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --no-default-features -- -D warnings
	just -f plugins/opencode/justfile check
	bash -n plugins/copilot/cortado-hook.sh

format:
	cargo fmt --manifest-path src-tauri/Cargo.toml --all

test:
	cargo test --manifest-path src-tauri/Cargo.toml --no-default-features
	just -f plugins/opencode/justfile test

check:
	just format
	just lint
	just test

build:
	@if [ ! -d node_modules ]; then just install; fi
	pnpm exec tauri build

build-signed:
	@if [ ! -d node_modules ]; then just install; fi
	set -a && . ./.env && set +a && pnpm exec tauri build

e2e:
	cargo test --manifest-path src-tauri/Cargo.toml --no-default-features -- --ignored feed::harness::e2e 2>&1

test-focus:
	cargo test --manifest-path src-tauri/Cargo.toml --no-default-features -- --ignored terminal_focus::e2e 2>&1
