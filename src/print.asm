print_string:
    mov ah, 0x0e

.loop:
    mov al, [bx]
    cmp al, 0
    je .done
    int 0x10
    inc bx
    jmp .loop

.done:
    ret
    
