cargo build --release
mkdir -p esp/EFI/BOOT
cp target/x86_64-unknown-uefi/release/something.efi esp/EFI/BOOT/BOOTX64.EFI
qemu-system-x86_64 -bios /usr/share/edk2/x64/OVMF.4m.fd -drive format=raw,file=fat:rw:esp
