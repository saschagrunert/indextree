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
	cargo doc --all --no-deps

lint-clippy:
	cargo clippy -- -D warnings

lint-rustfmt:
	cargo fmt --version
	cargo fmt
	git diff --exit-code

test:
	cargo test --features="par_iter"
