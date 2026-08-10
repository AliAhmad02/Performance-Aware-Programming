global Read_32x8

section .text

Read_32x8:
    xor r9, r9
    mov rax, rsi
	align 64

.loop:
    ; Read 256 bytes
    vmovdqu ymm0, [rax]
    vmovdqu ymm0, [rax + 0x20]
    vmovdqu ymm0, [rax + 0x40]
    vmovdqu ymm0, [rax + 0x60]
    vmovdqu ymm0, [rax + 0x80]
    vmovdqu ymm0, [rax + 0xa0]
    vmovdqu ymm0, [rax + 0xc0]
    vmovdqu ymm0, [rax + 0xe0]
    
    ; Advance and mask the read offset
    add r9, 0x100
    and r9, rdx

    ; Update the read base pointer to point to the new offset
    mov rax, rsi
    add rax, r9
    
    ; Repeat
    sub rdi, 0x100
    jnz .loop
    
    ret


