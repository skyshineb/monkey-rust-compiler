.PHONY: fmt fmt-check test compat check goldens-update release-check bench

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

test:
	cargo test

compat:
	cargo test compat_

check: fmt-check test

goldens-update:
	UPDATE_GOLDENS=1 cargo test compat_

release-check:
	./scripts/release-check.sh

bench:
	./scripts/bench.sh
