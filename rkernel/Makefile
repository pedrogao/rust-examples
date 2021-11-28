default: build

.PHONY: build

build:
	@echo "build for x86_64 kernel"
	@rustup component add llvm-tools-preview
	@cargo install bootimage
	@cargo bootimage
	@cargo run
