disk_load:
    pusha
    push dx

    mov ah, 0x02 ; tells bios we want to read sectors
    mov al, dh   ; dh is input parameter
    mov cl, 0x02 ; start from sector 2 as sector 1 is boot sector

    ; This comes from how old hard disks were made, this is where our bootloader and things after live
    mov ch, 0x00 
    mov dh, 0x00  

    int 0x13      ; BIOS interrupt to read disk
    jc disk_error ; check carry bit for error

    pop dx        ; We saved it before by pushing it on top (line 3)
    cmp al, dh    ; BIOS sets 'al' to the # of sectors actually read compare it with the number of sectors
                  ; we had to read from the arguement passed if they are unequal throw and error

    jne sectors_error
    popa

    ret

disk_error:
    mov bx, disk_error_msg
    call print_string
    jmp $

sectors_error:
    mov bx, sectors_error_msg
    call print_string
    jmp $

disk_error_msg   db 'Disk read error!', 0
sectors_error_msg db 'Wrong number of sectors read!', 0
