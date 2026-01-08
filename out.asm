section .bss
    dump_buf: resb 21
    data_stack: resq 4096
section .data
    str_0: db "Calcul: ", 0
    str_1: db "", 10, "Fin", 10, "", 0
    str_2: db "", 10, "", 10, "Done", 10, "", 0

section .text

dump_i:
        sub     rsp, 40
        xor     r9d, r9d
        test    rdi, rdi
        jns     .L2
        neg     rdi
        mov     r9d, 1
.L2:
        mov  rsi, 7378697629483820647
        mov     ecx, 32
.L3:
        mov     rax, rdi
        mov     r8, rcx
        sub     rcx, 1
        imul    rsi
        mov     rax, rdi
        sar     rax, 63
        sar     rdx, 2
        sub     rdx, rax
        lea     rax, [rdx+rdx*4]
        add     rax, rax
        sub     rdi, rax
        add     edi, 48
        mov     BYTE [rsp+rcx], dil
        mov     rdi, rdx
        test    rdx, rdx
        jne     .L3
        test    r9d, r9d
        je      .L4
        mov     BYTE [rsp-2+r8], 45
        lea     rcx, [r8-2]
.L4:
    mov     rdx, 32
    lea     rsi, [rsp+rcx]
    sub     rdx, rcx
    mov     rax, 1
    mov     rdi, 1
    syscall
    add     rsp, 40
    ret


dump_str:
    push    rbx
    mov     rbx, rdi
    xor     rax, rax
.loop:
    cmp     byte [rdi + rax], 0
    je      .done
    inc     rax
    jmp     .loop
.done:
    mov     rdx, rax
    mov     rsi, rbx
    mov     rax, 1
    mov     rdi, 1
    pop     rbx
    syscall
    ret


proc_calc:
    call     proc_CONST_A
    call     proc_CONST_B
    mov      rax, [r15]
    add      rax, [r15 + 8]
    add      r15, 8
    mov      [r15], rax
    ret      
proc_print:
    sub      r15, 8
    mov      rax, 10
    mov      qword [r15], rax
    sub      r15, 8
    mov      rax, 11
    mov      qword [r15], rax
    mov      rax, [r15 + 8]
    sub      rax, [r15]
    add      r15, 8
    mov      [r15], rax
    mov      rdi, [r15]
    add      r15, 8
    call     dump_i
    ret      
proc_CONST_A:
    sub      r15, 8
    mov      rax, 100
    mov      qword [r15], rax
    ret      
proc_main:
    sub      r15, 8
    mov      qword [r15], str_0
    mov      rdi, [r15]
    add      r15, 8
    call     dump_str
    call     proc_calc
    mov      rdi, [r15]
    add      r15, 8
    call     dump_i
    sub      r15, 8
    mov      qword [r15], str_1
    mov      rdi, [r15]
    add      r15, 8
    call     dump_str
    call     proc_print
    sub      r15, 8
    mov      qword [r15], str_2
    mov      rdi, [r15]
    add      r15, 8
    call     dump_str
    ret      
proc_CONST_B:
    sub      r15, 8
    mov      rax, 200
    mov      qword [r15], rax
    ret      

global _start
_start:
    lea      r15, [data_stack + 4096*8]
    call     proc_main
    mov      rax, 60
    xor      rdi, rdi
    syscall  
