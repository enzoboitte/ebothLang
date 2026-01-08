section .data
    dump_buf: resb 21
    data_stack: resq 4096
    str_0: db "Calcul: ", 0
    str_1: db "", 10, "Fin", 10, "", 0

section .text
dump_value:
    cmp      rax, 0x1000
    jb       .print_int
.print_str:
    mov      rsi, rax
    xor      rdx, rdx
.strlen_loop:
    cmp      byte [rsi + rdx], 0
    je       .strlen_done
    inc      rdx
    jmp      .strlen_loop
.strlen_done:
    mov      rax, 1
    mov      rdi, 1
    syscall  
    ret      
.print_int:
    mov      rcx, 10
    mov      rdi, dump_buf
    add      rdi, 20
    mov      byte [rdi], 10
    dec      rdi
    xor      r8, r8
    test     rax, rax
    jns      .positive
    neg      rax
    mov      r8, 1
.positive:
.convert_loop:
    xor      rdx, rdx
    div      rcx
    add      dl, '0'
    mov      [rdi], dl
    dec      rdi
    test     rax, rax
    jnz      .convert_loop
    test     r8, r8
    jz       .write
    mov      byte [rdi], '-'
    dec      rdi
.write:
    inc      rdi
    mov      rdx, 21
    mov      rsi, dump_buf
    sub      rdx, rdi
    add      rdx, rsi
    mov      rsi, rdi
    mov      rax, 1
    mov      rdi, 1
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
proc_main:
    sub      r15, 8
    mov      qword [r15], str_0
    mov      rax, [r15]
    add      r15, 8
    call     dump_value
    call     proc_calc
    mov      rax, [r15]
    add      r15, 8
    call     dump_value
    sub      r15, 8
    mov      qword [r15], str_1
    mov      rax, [r15]
    add      r15, 8
    call     dump_value
    ret      
proc_CONST_B:
    sub      r15, 8
    mov      qword [r15], 200
    ret      

global _start
_start:
    lea      r15, [data_stack + 4096*8]
    call     proc_main
    mov      rax, 60
    xor      rdi, rdi
    syscall  
