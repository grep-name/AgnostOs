bits 16
[org 0x7C00]

mov bp, 0x9000
mov sp, bp
mov [boot_drive], dl

kernel_start equ 0x1000

mov bx, msg
call print_string

jmp $

%include "disk.asm"
%include "print.asm"

msg db 'Hello from bootloader!', 0

load_kernel:
    mov bx, kernel_start
    mov dh, 2             ; number of sectors to read
    mov dl, [boot_drive]  ; where to read the sectors from
    call disk_load        ; call the actual function
    ret


boot_drive db 0

; padding
times 510 - ($-$$) db 0

; magic number
dw 0xaa55
