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

check:
	just format
	just lint
