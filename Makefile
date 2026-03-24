GENERAL_ARGS = --release

.PHONY: \
	build \
	build-doc \
	lint-rustfmt \
	lint-clippy \
	test

ifndef VERBOSE
.SILENT:
else
GENERAL_ARGS += -v
endif

all: build

build:
	cargo build $(GENERAL_ARGS)

build-doc:
	cargo doc --workspace --all-features --no-deps

lint-clippy:
	cargo clippy --all-targets --all-features -- -D warnings

lint-rustfmt:
	cargo fmt --version
	cargo fmt
	git diff --exit-code

test:
	cargo test --all-features
	cargo test --no-default-features
