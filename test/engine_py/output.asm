bits 64

section .bss
    print_buffer: resb 32
    float_buf: resb 64
    float_test: resb 4

section .data
    pi: dd 3.14159
    number: dd 42
    str_print_0: db "=== Test Allocation Intelligente ===", 10
    newline: db 10
    str_print_1: db "=== Test Float Automatique ===", 10
    f2: dd 2.718

section .text
    global _start

test_locals:
    push rbp
    mov rbp, rsp
    sub rsp, 8
    push r12
    push rbx
    mov ebx, [rbp+16]
    mov r12, rbx
    add r12, 10
    mov [rbp-4], r12
    movss xmm0, [pi]
    movss [rbp-8], xmm0
    mov r12d, [rbp-4]
    mov rax, r12
    pop r12
    pop rbx
    mov rsp, rbp
    pop rbp
    ret
print_int:
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    mov rax, [rbp+16]
    lea rsi, [print_buffer+31]
    mov byte [rsi], 0
    dec rsi
    mov rbx, 10
    test rax, rax
    jns .positive
    neg rax
    mov r13, 1
    jmp .convert
.positive:
    xor r13, r13
.convert:
    xor rdx, rdx
    div rbx
    add dl, '0'
    mov [rsi], dl
    dec rsi
    test rax, rax
    jnz .convert
    test r13, r13
    jz .print
    mov byte [rsi], '-'
    dec rsi
.print:
    inc rsi
    mov rax, 1
    mov rdi, 1
    lea rdx, [print_buffer+31]
    sub rdx, rsi
    syscall
    mov rax, 1
    mov rdi, 1
    lea rsi, [newline]
    mov rdx, 1
    syscall
    pop r13
    pop r12
    pop rbx
    mov rsp, rbp
    pop rbp
    ret
print_float:
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    push r15
    movss xmm0, [rbp+16]
    cvtss2sd xmm0, xmm0
    pxor xmm1, xmm1
    ucomisd xmm0, xmm1
    jae .positive_float
    mov byte [float_buf], '-'
    mov r14, 1
    movsd xmm1, xmm0
    xorpd xmm0, xmm0
    subsd xmm0, xmm1
    jmp .extract_int
.positive_float:
    xor r14, r14
.extract_int:
    cvttsd2si rax, xmm0
    cvtsi2sd xmm1, rax
    subsd xmm0, xmm1
    lea rsi, [float_buf+31]
    mov byte [rsi], 0
    dec rsi
    mov rbx, 10
.int_loop:
    xor rdx, rdx
    div rbx
    add dl, '0'
    mov [rsi], dl
    dec rsi
    test rax, rax
    jnz .int_loop
    test r14, r14
    jz .write_point
    mov byte [rsi], '-'
    dec rsi
.write_point:
    inc rsi
    mov r14, rsi
    lea rsi, [float_buf+32]
    mov byte [rsi], '.'
    inc rsi
    mov rcx, 6
.frac_loop:
    mov r15, 10
    cvtsi2sd xmm2, r15
    mulsd xmm0, xmm2
    cvttsd2si r12, xmm0
    cvtsi2sd xmm1, r12
    subsd xmm0, xmm1
    add r12, '0'
    mov [rsi], r12b
    inc rsi
    loop .frac_loop
    mov byte [rsi], 10
    inc rsi
    mov rax, 1
    mov rdi, 1
    mov rdx, rsi
    sub rdx, r14
    mov rsi, r14
    syscall
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    mov rsp, rbp
    pop rbp
    ret

_start:
    mov rax, 1
    mov rdi, 1
    lea rsi, [str_print_0]
    mov rdx, 37
    syscall
    mov r13, 5
    push r13
    call test_locals
    add rsp, 8
    mov r14, rax
    push r14
    call print_int
    add rsp, 8
    mov rax, 1
    mov rdi, 1
    lea rsi, [str_print_1]
    mov rdx, 31
    syscall
    movss xmm1, [pi]
    sub rsp, 8
    movss [rsp], xmm1
    call print_float
    add rsp, 8
    movss xmm2, [f2]
    movss [float_test], xmm2
    movss xmm3, [float_test]
    sub rsp, 8
    movss [rsp], xmm3
    call print_float
    add rsp, 8
    mov rax, 60
    mov rdi, 0
    syscall