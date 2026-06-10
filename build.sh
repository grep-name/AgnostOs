cd src
nasm -f bin boot.asm -o boot.bin
mv boot.bin ../build/
cd ../build/
qemu-system-x86_64 -drive format=raw,file=boot.bin
