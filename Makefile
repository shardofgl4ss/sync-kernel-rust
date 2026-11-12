include			config.mk


PROJ_DIR		:= $(CURDIR)
SRC_DIR			:= kernel
IMG_DIR			:= img

UEFI_DIR		:= uefi
UEFI_CODE		:= $(UEFI_DIR)/OVMF_CODE.fd
UEFI_VARS		:= $(UEFI_DIR)/OVMF_VARS.fd


BUILD_TYPE		?= debug
RS_TARGET		?= x86_64-unknown-none
RS_RELOC		?= pic
RS_TARGET_DIR		:= target


DISK_IMAGE		:= $(IMG_DIR)/disk.img
KERNEL_ELF		:= $(IMG_DIR)/kernel.elf
GRUB_FILE		:= $(IMG_DIR)/BOOTX64.EFI


########## asm
ASM_ALL			:= $(sort $(shell find $(SRC_DIR) -type f \( -name '*.S' -o -name '*.s' \)))
ASM_SOURCES		:= $(filter-out $(SRC_DIR)/arch/%,$(ASM_ALL))
ASM_SOURCES		+= $(filter $(SRC_DIR)/arch/$(ARCH)/%,$(ASM_ALL))
 
OBJECTS			:= $(addsuffix .o,$(ASM_SOURCES))
DEPS			:= $(OBJECTS:.o=.d)

########## rust
ifeq ($(BUILD_TYPE), release)
	RS_PROFILE_DIR	:= release
	CARGO_FLAGS	:= --release
else
	RS_PROFILE_DIR	:= debug
	CARGO_FLAGS	:=
endif

RS_OUT_DIR		:= $(RS_TARGET_DIR)/$(RS_TARGET)/$(RS_PROFILE_DIR)
RS_LIBS			:= $(foreach p,$(RS_PACKAGES),$(RS_OUT_DIR)/lib$(subst -,_,$(p)).a)
KERNEL_AR		:= $(RS_OUT_DIR)/libsyos_kernel.a
RSFLAGS			:= -C relocation-model=$(RS_RELOC) -C code-model=kernel

# Tracked inputs: every .rs / Cargo.toml in the Rust dirs, plus workspace files.
RS_INPUTS		:= $(wildcard Cargo.toml Cargo.lock rust-toolchain.toml .cargo/config.toml)
RS_INPUTS		+= $(sort $(shell find $(SRC_DIR) -type f -name '*.rs'))


OBJECTS			:= $(addsuffix .o, $(ASM_SOURCES))
DEPS			:= $(OBJECTS:.o=.d)


CFLAGS			:= $(CFLAGS_COMMON) $(CFLAGS_STANDALONE) $(CFLAGS_KERNELSPACE)
CPPFLAGS		+= $(addprefix -I,$(INC_DIRS))
ASMFLAGS 		:= -I$(SRC_DIR) -I$(SRC_DIR)/include
DEPFLAGS		= -MT $@ -MMD -MP -MF $(@:.o=.d)


.PHONY: 		all image kernel clean distclean standalone run

all: 			image

image: 			$(DISK_IMAGE)-grub
kernel: 		$(KERNEL_ELF)
standalone:		$(GRUB_FILE)

QEMU_UEFI		:= -drive if=pflash,format=raw,readonly=on,file=$(UEFI_CODE) \
			   -drive if=pflash,format=raw,file=$(UEFI_VARS)

QEMU_DISK 		:= -drive format=raw,file=$(DISK_IMAGE)

QEMU_FLAGS		:= $(QEMU_UEFI) $(QEMU_DISK) -no-reboot -no-shutdown -d cpu_reset -debugcon stdio -global isa-debugcon.iobase=0xe9

QEMU_KVM		?= 0
QEMU_GDB_STOP		?= 1

ifeq ($(QEMU_KVM), 1)
	QEMU_FLAGS 	+= -cpu host -accel kvm
endif

ifeq ($(QEMU_GDB_STOP), 1)
	QEMU_FLAGS 	+= -s -S
endif

ifeq ($(BUILD_TYPE), release)
	CFLAGS += $(CFLAGS_RELEASE)
else
	CFLAGS += $(CFLAGS_DEBUG)
	CPPFLAGS += -DDEBUG
	LDFLAGS += -g
	ASMFLAGS += -g
	RSFLAGS += -Z emit-stack-sizes
endif


# KVM acceleration with host cpu make it impossible to debug with GDB.
# But it *is* closer to true hardware behavior then base QEMU.
run: image $(UEFI_CODE) $(UEFI_VARS)
	@qemu-system-x86_64 $(QEMU_FLAGS)


$(KERNEL_ELF): $(OBJECTS) $(KERNEL_AR) $(LINKER_FILE)
	@mkdir -p $(dir $@)
	@printf "  LD        %s\n" "$@"
	@$(LD) $(LDFLAGS) -o $@ $(OBJECTS) $(KERNEL_AR)

$(KERNEL_AR): $(RS_INPUTS)
	cargo rustc -v $(CARGO_FLAGS) --target $(RS_TARGET) -p syos-kernel --crate-type staticlib -- $(RSFLAGS)


$(DISK_IMAGE)-kernel: $(KERNEL_ELF) | $(DISK_IMAGE)
	@mkdir -p $(dir $@)
	@printf "  MCOPY     %s\t%s::/boot/kernel.elf\n" "$(KERNEL_ELF)" "$(DISK_IMAGE)"
	@mcopy -o -i $(DISK_IMAGE) $(KERNEL_ELF) "::/boot/kernel.elf"
	@touch $@
$(DISK_IMAGE)-grub: $(GRUB_FILE) | $(DISK_IMAGE)-kernel
	@mkdir -p $(dir $@)
	@printf "  MCOPY     %s\t%s::/EFI/BOOT/BOOTX64.EFI\n" "$(GRUB_FILE)" "$(DISK_IMAGE)"
	@mcopy -o -i $(DISK_IMAGE) $(GRUB_FILE) "::/EFI/BOOT/BOOTX64.EFI"
	@touch $@


$(DISK_IMAGE):
	@mkdir -p $(dir $@)
	@printf "  TR        %s\n" "$@"
	@truncate -s 64M $@
	@printf "  MKFS      %s\t%s\n" "$@" "FAT32"
	@mkfs.fat -F 32 -n 'SYOS' $@ >/dev/null 2>&1
	@mmd -i $@ "::/boot"
	@mmd -i $@ "::/EFI"
	@mmd -i $@ "::/EFI/BOOT"
	@rm -f $(DISK_IMAGE)-kernel $(DISK_IMAGE)-grub


$(GRUB_FILE): grub.cfg
	@mkdir -p $(dir $@)
	@printf "  BOOTLDR   %s\n" "$@"
	@grub-mkstandalone \
		-O x86_64-efi \
		-o $@ \
		--modules="fat part_gpt multiboot2" \
		'boot/grub/grub.cfg=grub.cfg'


%.S.o: %.S
	$(CC) $(CPPFLAGS) $(CFLAGS) $(DEPFLAGS) -c -o $@ $<

# @printf "  CC        %s\n" "$@"
%.s.o: %.s
	$(AS) $(ASMFLAGS) -c -o $@ $<

# @printf "  CC        %s\n" "$@"

-include $(DEPS)

clean:
	@rm -f $(OBJECTS)
	@rm -f $(DEPS)
	@rm -f $(KERNEL_ELF)
	@cargo clean
	@printf "cleaned build artifacts\n"


distclean: clean
	@rm -f $(DISK_IMAGE)
	@rm -f $(DISK_IMAGE)-kernel $(DISK_IMAGE)-grub
	@rm -f $(GRUB_FILE)
	@printf "cleaned boot and disk files\n"

