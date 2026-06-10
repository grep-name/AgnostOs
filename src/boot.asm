bits 16
[org 0x7C00]

mov bp, 0x9000
mov sp, bp
mov [boot_drive], dl

call load_kernel
jmp kernel_start

load_kernel:
    mov ax, 0
    mov es, ax
    mov bx, kernel_start
    mov dh, 1
    mov dl, 0x80
    call disk_load
    ret

boot_drive db 0
kernel_start equ 0x1000

%include "disk.asm"
%include "print.asm"

times 510 - ($-$$) db 0
dw 0xaa55
