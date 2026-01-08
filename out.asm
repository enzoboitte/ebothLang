section .data
    dump_buf: resb 21
    data_stack: resq 4096
    str_0: db "Calcul: ", 0
    str_1: db "", 10, "Fin", 10, "", 0

section .text

dump_i:
    mov     r8, -3689348814741910323
    sub     rsp, 40
    lea     rcx, [rsp+30]
.L2:
    mov     rax, rdi
    mul     r8
    mov     rax, rdi
    shr     rdx, 3
    lea     rsi, [rdx+rdx*4]
    add     rsi, rsi
    sub     rax, rsi
    mov     rsi, rcx
    sub     rcx, 1
    add     eax, 48
    mov     byte [rcx+1], al
    mov     rax, rdi
    mov     rdi, rdx
    cmp     rax, 9
    ja      .L2
    lea     rdx, [rsp+32]
    sub     rdx, rsi
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


proc_CONST_A:
    sub      r15, 8
    mov      qword [r15], 100
    ret      
proc_calc:
    call     proc_CONST_A
    call     proc_CONST_B
    mov      rax, [r15]
    add      rax, [r15 + 8]
    add      r15, 8
    mov      [r15], rax
    ret      
proc_CONST_B:
    sub      r15, 8
    mov      qword [r15], 200
    ret      
proc_print:
    sub      r15, 8
    mov      qword [r15], 1234567891
    mov      rdi, [r15]
    add      r15, 8
    call     dump_i
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
    ret      

global _start
_start:
    lea      r15, [data_stack + 4096*8]
    call     proc_main
    mov      rax, 60
    xor      rdi, rdi
    syscall  
