# 1) prerequirement
# ./scripts/install_develop.sh
# 2) development
# ./scripts/config_rabbitms.sh
# ./scripts/speedup.sh
# make fmt
# make debug
# make test
# make bench
# make release
################################################################################
CARGO=RUSTFLAGS='-F warnings' cargo

debug:
	$(CARGO) build --all
	scripts/release.sh debug

release:
	$(CARGO) build --all  --release
	scripts/release.sh release

test:
	$(CARGO) test --all 2>&1 |tee target/test.log


test_ed25519_blake2b:
	sed -i 's/\["secp256k1"\]/\["ed25519"\]/g' share_libs/crypto/Cargo.toml
	sed -i 's/\["sha3hash"\]/\["blake2bhash"\]/g' share_libs/util/Cargo.toml
	$(CARGO) test  --all 2>&1 |tee target/test.log
	sed -i 's/\["ed25519"\]/\["secp256k1"\]/g' share_libs/crypto/Cargo.toml
	sed -i 's/\["blake2bhash"\]/\["sha3hash"\]/g' share_libs/util/Cargo.toml

bench:
	-rm target/bench.log
	cargo bench --all --no-run |tee target/bench.log
	cargo bench --all --jobs 1 |tee -a target/bench.log

fmt:
	cargo fmt --all  -- --write-mode diff

cov:
	cargo cov test --all
	cargo cov report --open

clean:
	rm -rf target/debug/
	rm -rf target/release/

clippy:
	cargo build --features clippy --all
