ARCH = x86_64-unknown-linux-gnu

.PHONY: build
build:
	cargo lambda build --release
	rm -rf ./build
	mkdir -p ./build
	cp -v ../target/lambda/nepospot-serve/bootstrap ./build/bootstrap
