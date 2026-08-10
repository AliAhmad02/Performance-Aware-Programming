global Read_x1
global Read_x2
global Read_x3
global Read_x4
global Write_x1
global Write_x2
global Write_x3
global Write_x4

section .text

Read_x1:
	align 64
.loop:
    mov rax, [rsi]
    sub rdi, 1
    jnle .loop
    ret

Read_x2:
	align 64
.loop:
    mov rax, [rsi]
    mov rax, [rsi]
    sub rdi, 2
    jnle .loop
    ret

Read_x3:
    align 64
.loop:
    mov rax, [rsi]
    mov rax, [rsi]
    mov rax, [rsi]
    sub rdi, 3
    jnle .loop
    ret

Read_x4:
	align 64
.loop:
    mov rax, [rsi]
    mov rax, [rsi]
    mov rax, [rsi]
    mov rax, [rsi]
    sub rdi, 4
    jnle .loop
    ret

Write_x1:
	align 64
.loop:
    mov [rsi], 1
    sub rdi, 1
    jnle .loop
    ret

Write_x2:
	align 64
.loop:
    mov [rsi], 1
    mov [rsi], 2
    sub rdi, 2
    jnle .loop
    ret

Write_x3:
    align 64
.loop:
    mov [rsi], 1
    mov [rsi], 2
    mov [rsi], 3
    sub rdi, 3
    jnle .loop
    ret

Write_x4:
	align 64
.loop:
    mov [rsi], 1
    mov [rsi], 2
    mov [rsi], 3
    mov [rsi], 4
    sub rdi, 4
    jnle .loop
    ret
