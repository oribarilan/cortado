default:
	@just --list

install:
	pnpm install

dev:
	@if [ ! -d node_modules ]; then just install; fi
	pnpm exec tauri dev

lint:
	@if [ ! -d node_modules ]; then just install; fi
	pnpm exec tsc --noEmit
	cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --no-default-features -- -D warnings

format:
	cargo fmt --manifest-path src-tauri/Cargo.toml --all

test:
	cargo test --manifest-path src-tauri/Cargo.toml --no-default-features

check:
	just format
	just lint
	just test

local-e2e:
	cargo test --manifest-path src-tauri/Cargo.toml --no-default-features -- --ignored terminal_focus::e2e 2>&1
