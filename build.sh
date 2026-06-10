cd build/
rm -f *

cd ../src/

nasm -f bin boot.asm -o boot.bin
nasm -f bin kernel.asm -o kernel.bin

mv boot.bin kernel.bin ../build/
cd ../build/

cat boot.bin kernel.bin > os.bin
qemu-system-x86_64 -drive format=raw,file=os.bin
