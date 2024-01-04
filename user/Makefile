TARGET_PLATFORM := riscv64gc-unknown-none-elf
MODE := release
APPS_DIR := src/apps
TARGET_DIR := target/$(TARGET_PLATFORM)/$(MODE)
APPS_SRC := $(wildcard $(APPS_DIR)/*.rs)
APPS_ELF := $(patsubst $(APPS_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS_SRC))

OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

elf:
	@cargo build --release

binary: elf
	@$(foreach elf, $(APPS_ELF), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf));)

build: binary