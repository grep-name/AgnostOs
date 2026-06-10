[bits 16]
[org 0x1000]

mov bx, kernel_msg 
call print_string

%include "print.asm"

jmp $
kernel_msg db 'Hello from the kernel!', 0
