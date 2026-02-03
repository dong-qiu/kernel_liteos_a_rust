include $(LITEOSTOPDIR)/config.mk

MODULE_NAME := $(notdir $(shell pwd))

LOCAL_SRCS := $(wildcard *.c)

HIDUMPER_USE_RUST ?= n
RUSTC ?= rustc
RUSTFLAGS ?= -C panic=abort -C opt-level=2 -C relocation-model=static
RUST_TARGET ?= $(if $(LOSCFG_LLVM_TARGET),$(LOSCFG_LLVM_TARGET:"%"=%),armv7a-none-eabi)

HIDUMPER_RUST_SRC := $(LITEOSTOPDIR)/kernel/extended/hidumper/rust/lib.rs
HIDUMPER_RUST_OBJ := $(OUT)/obj/kernel/extended/hidumper/rust/hidumper_rust.o

ifeq ($(HIDUMPER_USE_RUST),y)
LOCAL_CFLAGS += -DHIDUMPER_USE_RUST
LOCAL_EXTRA_OBJS += $(HIDUMPER_RUST_OBJ)
endif

include $(MODULE)

ifeq ($(HIDUMPER_USE_RUST),y)
$(HIDUMPER_RUST_OBJ): $(HIDUMPER_RUST_SRC)
	$(HIDE)$(RUSTC) --crate-type=lib --emit=obj -o $@ $(RUSTFLAGS) --target $(RUST_TARGET) $<
endif
