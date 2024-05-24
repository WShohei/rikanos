KERNEL = ./kernel/target/x86_64-unknown-rikanos/release/kernel
KERNEL_PATH = ~/workspace/rikanos/kernel
LOADER = ./bootloader/target/x86_64-unknown-uefi/release/rikan-loader.efi
LOADER_PATH = ~/workspace/rikanos/bootloader
BOOTIMAGE = rikan.img
TESTIMAGE = rikan-test.img

.PHONY: qemu
qemu: $(BOOTIMAGE)
	./qemu-run.sh

.PHONY: test
test: $(TESTIMAGE)
	./qemu-run.sh $(TESTIMAGE)

.PHONY: clean
clean: clean-loader clean-kernel
	rm -f $(BOOTIMAGE)

$(BOOTIMAGE): build
	./make-image.sh

$(TESTIMAGE): build-test-kernel $(LOADER)
	./make-image.sh $(TESTIMAGE)

.PHONY: build-test-kernel
build-test-kernel:; cd $(KERNEL_PATH) && cargo test --no-run --release

.PHONY: build
build: build-kernel build-loader

.PHONY: build-kernel
build-kernel:;	cd $(KERNEL_PATH) && cargo build --release

.PHONY: build-loader
build-loader:;	cd $(LOADER_PATH) && cargo build --release

.PHONY: clean-kernel
clean-kernel:; cd $(KERNEL_PATH) && cargo clean

.PHONY: clean-loader
clean-loader:; cd $(LOADER_PATH) && cargo clean
