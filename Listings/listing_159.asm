global TemporalWrite
global NonTemporalWrite

section .text

; rdi: read pointer
; rsi: write pointer
; rdx: inner count
; rcx: outer count

TemporalWrite:
  align 64

.outer_t:
    ; Store/reset inner count and read pointer
    mov r9, rdx
    mov rax, rdi

.inner_t:
    ; Read and write 256 bytes
    vmovdqu ymm0, [rax]
    vmovdqu [rsi], ymm0 
    vmovdqu ymm0, [rax + 0x20]
    vmovdqu [rsi + 0x20], ymm0 
    vmovdqu ymm0, [rax + 0x40]
    vmovdqu [rsi + 0x40], ymm0 
    vmovdqu ymm0, [rax + 0x60]
    vmovdqu [rsi + 0x60], ymm0 
    vmovdqu ymm0, [rax + 0x80]
    vmovdqu [rsi + 0x80], ymm0 
    vmovdqu ymm0, [rax + 0xa0]
    vmovdqu [rsi + 0xa0], ymm0 
    vmovdqu ymm0, [rax + 0xc0]
    vmovdqu [rsi + 0xc0], ymm0 
    vmovdqu ymm0, [rax + 0xe0]
    vmovdqu [rsi + 0xe0], ymm0 

    ; Increment read and write pointers
    add rax, 256
    add rsi, 256
    
    ; Decrement inner count and loop
    dec r9
    jnz .inner_t

    ; Decrement outer count and loop
    dec rcx
    jnz .outer_t
    ret


NonTemporalWrite:
  align 64

.outer_nt:
    ; Store/reset inner count and read pointer
    mov r9, rdx
    mov rax, rdi

.inner_nt:
    ; Read and write 256 bytes
    vmovdqu ymm0, [rax]
    vmovntdq [rsi], ymm0 
    vmovdqu ymm0, [rax + 0x20]
    vmovntdq [rsi + 0x20], ymm0 
    vmovdqu ymm0, [rax + 0x40]
    vmovntdq [rsi + 0x40], ymm0 
    vmovdqu ymm0, [rax + 0x60]
    vmovntdq [rsi + 0x60], ymm0 
    vmovdqu ymm0, [rax + 0x80]
    vmovntdq [rsi + 0x80], ymm0 
    vmovdqu ymm0, [rax + 0xa0]
    vmovntdq [rsi + 0xa0], ymm0 
    vmovdqu ymm0, [rax + 0xc0]
    vmovntdq [rsi + 0xc0], ymm0 
    vmovdqu ymm0, [rax + 0xe0]
    vmovntdq [rsi + 0xe0], ymm0 

    ; Increment read and write pointers
    add rax, 256
    add rsi, 256
    
    ; Decrement inner count and loop
    dec r9
    jnz .inner_nt

    ; Decrement outer count and loop
    dec rcx
    jnz .outer_nt
    ret
