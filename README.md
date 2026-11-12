# SYOS (my "little" hobby project)

>This version is a rust version of sync-kernel because I want to, and its fun.

This is a personal project, so don't expect this to be done soon, nor am I an expert or professional by any means.

I'm quite busy with it, and also learning OS dev and kernel development on my own time, so it is also going to be slow,  it is just a personal project of mine.


I will personally try to make sure the kernel compiles properly without errors,
before every push to the repo. At the moment, the kernel is nowhere near even usable,
so do not expect pushes to have a usable kernel for a long while, it just will compile, it's 
still a major WIP.


At the moment I will not take contributions, but I will take feedback, especially code criticism.
If you are an expert in OS development, I would love to hear feedback or details of OS dev.


That being said, if you want to try it,

# Compiling
Required tools:

>GNU Make (my version is 4.4.1)

>x86_64-elf cross compiler (GCC) for assembly

>Rust cross-compiler toolchain (2024 Edition)

>basic linux corelib functions (mkfs, coreutil's truncate)

>mtools (for disk image)

>fat32 support in mkfs.fat, should already be present in your distro

>grub (bootloader, grub-mkstandalone)

>optional: QEMU to test it, it *may* run on bare hardware right now, but I would not recommend.

>optional: OVMF_CODE.fd and OVMF_VARS.fd, uefi implementations for QEMU. put in uefi/ yourself if you use QEMU.

This probably won't compile on Windows or Mac, obviously.

# Notes

SYOS is a temporary name. It may not stick.

Rust Edition: 2024

Arch: x86-64 only (maybe in the future I will accomodate ARM64)

ABI: SYSV (no-red-zone)

(MAYBE) POSIX complaint in the future.

Syscalls: When I get to them, I will attach a table of them here.
