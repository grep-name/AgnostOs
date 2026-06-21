#!/bin/bash
set -e

cargo build --release
mkdir -p esp/EFI/BOOT
cp target/x86_64-unknown-uefi/release/agnostos.efi esp/EFI/BOOT/BOOTX64.EFI
qemu-system-x86_64 -bios ./bios/OVMF.4m.fd -drive format=raw,file=fat:rw:esp
