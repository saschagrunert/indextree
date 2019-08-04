GENERAL_ARGS = --release

.PHONY: \
	build \
	build-doc \
	coverage \
	lint-rustfmt \
	lint-clippy

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

coverage:
	cargo kcov --features="par_iter"

lint-clippy:
	cargo clippy -- -D warnings

lint-rustfmt:
	cargo fmt
	git diff --exit-code
