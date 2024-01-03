# Building Objects
TARGET := riscv64gc-unknown-none-elf
MODE := release
MODE_ARGS := --release
KERNEL_ELF := target/$(TARGET)/$(MODE)/kernel
KERNEL_BIN := $(KERNEL_ELF).bin

# Board
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER = ../bootloader/$(SBI)-$(BOARD).bin

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# Kernel Entry
KERNEL_ENTRY := 0x80200000

# Building Kernel
kernel:
	@cargo build $(MODE_ARGS)

env: 
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

build: env $(KERNEL_BIN)

QEMU_ARGS := -machine virt \
			 -nographic \
			 -bios $(BOOTLOADER) \
			 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY)

run-inner: build
	@qemu-system-riscv64 $(QEMU_ARGS)

run: run-inner

clean:
	@cargo clean

.PHONY: clean build env kernel run run-inner