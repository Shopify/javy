.PHONY: cli core test fmt clean
.DEFAULT_GOAL := cli

cli: core
		cd crates/cli && cargo build --release

core:
		cd crates/core && cargo build --release

tests: core
		cd crates/cli \
				&& cargo check --benches --release

fmt: fmt-quickjs-sys fmt-core fmt-cli

fmt-quickjs-sys:
		cd crates/quickjs-sys/ \
				&& cargo fmt -- --check \
				&& cargo clippy -- -D warnings \
				&& cd - \

fmt-core:
		cd crates/core/ \
				&& cargo fmt -- --check \
				&& cargo clippy -- -D warnings \
				&& cd - \

fmt-cli:
		cd crates/cli/ \
				&& cargo fmt -- --check \
				&& cargo clippy -- -D warnings \
				&& cd -

clean:
		cargo clean
