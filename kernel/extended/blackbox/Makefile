include $(LITEOSTOPDIR)/config.mk

MODULE_NAME := $(notdir $(shell pwd))

LOCAL_SRCS := $(wildcard *.c)

LOCAL_INCLUDE := -I $(LITEOSTOPDIR)/syscall

LOCAL_FLAGS := $(LOCAL_INCLUDE)

BLACKBOX_USE_RUST ?= $(LOSCFG_BLACKBOX_USE_RUST)
RUSTC ?= rustc
RUSTFLAGS ?= -C panic=abort -C opt-level=2 -C relocation-model=static
RUST_TARGET ?= $(if $(LOSCFG_LLVM_TARGET),$(LOSCFG_LLVM_TARGET:"%"=%),$(LITEOSTOPDIR)/targets/armv7a-none-eabi.json)

BLACKBOX_RUST_SRC := $(LITEOSTOPDIR)/kernel/extended/blackbox/rust/lib.rs
BLACKBOX_RUST_OBJ := $(OUT)/obj/kernel/extended/blackbox/rust/blackbox_rust.o

ifeq ($(BLACKBOX_USE_RUST),y)
LOCAL_CFLAGS += -DBLACKBOX_USE_RUST
LOCAL_EXTRA_OBJS += $(BLACKBOX_RUST_OBJ)
endif

include $(MODULE)

ifeq ($(BLACKBOX_USE_RUST),y)
$(BLACKBOX_RUST_OBJ): $(BLACKBOX_RUST_SRC)
	$(HIDE)$(RUSTC) --crate-type=lib --emit=obj -o $@ $(RUSTFLAGS) --target $(RUST_TARGET) $<
endif
