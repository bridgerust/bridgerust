.PHONY: build-all build-cli build-python build-node clean

# CLI Targets
build-cli-linux-x64:
	cross build --manifest-path cli/embex-cli/Cargo.toml --target x86_64-unknown-linux-gnu --release

build-cli-linux-arm64:
	cross build --manifest-path cli/embex-cli/Cargo.toml --target aarch64-unknown-linux-gnu --release

build-cli-windows:
	cross build --manifest-path cli/embex-cli/Cargo.toml --target x86_64-pc-windows-gnu --release

build-cli: build-cli-linux-x64 build-cli-linux-arm64 build-cli-windows

# Python Targets
build-py-linux-x64:
	cd bindings/python/embex && maturin build --release --target x86_64-unknown-linux-gnu

build-py-linux-arm64:
	cd bindings/python/embex && maturin build --release --target aarch64-unknown-linux-gnu

build-py-windows:
	cd bindings/python/embex && maturin build --release --target x86_64-pc-windows-gnu

build-py-mac-x64:
	cd bindings/python/embex && maturin build --release --target x86_64-apple-darwin

build-py-mac-arm64:
	cd bindings/python/embex && maturin build --release --target aarch64-apple-darwin

build-python: build-py-linux-x64 build-py-linux-arm64 build-py-windows build-py-mac-x64 build-py-mac-arm64

# Node.js Targets
build-node-linux-x64-musl:
	cd bindings/node/@bridgerust/embex && npm install && npx napi build --platform --release --target x86_64-unknown-linux-musl

build-node-linux-arm64-musl:
	cd bindings/node/@bridgerust/embex && npm install && npx napi build --platform --release --target aarch64-unknown-linux-musl

build-node-windows-x64:
	cd bindings/node/@bridgerust/embex && npm install && npx napi build --platform --release --target x86_64-pc-windows-msvc

build-node-mac-x64:
	cd bindings/node/@bridgerust/embex && npm install && npx napi build --platform --release --target x86_64-apple-darwin

build-node-mac-arm64:
	cd bindings/node/@bridgerust/embex && npm install && npx napi build --platform --release --target aarch64-apple-darwin

build-node: build-node-linux-x64-musl build-node-linux-arm64-musl build-node-windows-x64 build-node-mac-x64 build-node-mac-arm64

build-all: build-cli build-python build-node
