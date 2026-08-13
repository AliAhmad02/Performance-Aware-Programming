global DoubleLoopRead_32x8

section .text

; rdi: outer count (no. of blocks to read full buffer equivalent)
; rsi: data pointer
; rdx: inner count (no. of 256 byte reads per block)

DoubleLoopRead_32x8:
	align 64

.outer:
    ; Store/reset inner count and data pointer
    mov r9, rdx
    mov rax, rsi

.inner:
    ; Read 256 bytes
    vmovdqu ymm0, [rax]
    vmovdqu ymm0, [rax + 0x20]
    vmovdqu ymm0, [rax + 0x40]
    vmovdqu ymm0, [rax + 0x60]
    vmovdqu ymm0, [rax + 0x80]
    vmovdqu ymm0, [rax + 0xa0]
    vmovdqu ymm0, [rax + 0xc0]
    vmovdqu ymm0, [rax + 0xe0]

    ; Increment data pointer
    add rax, 256
    
    ; Decrement inner count and loop
    dec r9
    jnz .inner

    ; Decrement outer count and loop
    dec rdi
    jnz .outer
    ret
