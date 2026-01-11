#![allow(nonstandard_style)]
mod syntax;
use libc::free;
use syntax::F_lParseProgram;

use std::{collections::HashMap, fmt::Write};

#[derive(PartialEq, Clone, Debug)]
enum EType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    Ptr,
    Str,
    Bool,
    Void,
}

#[derive(Clone, Debug)]
enum EValue {
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    Ptr(u64),
    Str(&'static str),
    Bool(bool),
}

#[derive(Clone, Debug)]
enum EIrInstr {
    // Data manipulation
    PushI64(i64),               // number
    PushStr(&'static str),      // "..."

    // Arithmetic
    AddI64,                     // +
    SubI64,                     // not implemented
    MulI64,                     // not implemented
    DivI64,                     // not implemented
    ModI64,                     // not implemented   

    // Stack manipulation
    Dup,                        // dup
    Swap,                       // swap
    Puts,                       // puts
    Dump,                       // dump

    // Syscalls
    Syscall0,                   // syscall
    Syscall1,                   // syscall1
    Syscall2,                   // syscall2
    Syscall3,                   // syscall3
    Syscall4,                   // syscall4
    Syscall5,                   // syscall5
    Syscall6,                   // syscall6 

    // Control flow (not implemented in this example)
    Call(&'static str),         // [name_proc/const]
    Ret,                        // end (for proc)
    RetType,                    // -- [type] (for proc)


    Proc(&'static str, Vec<EIrInstr>, Vec<EType>, EType),  // proc [name] in ... end
    Const(&'static str, Vec<EIrInstr>), // const [name] in ... end
}

struct CAsmBuilder {
    l_sData: String,
    l_sCode: String,
    l_iFuncCode: String,
    l_iStrCount: usize,
}

impl CAsmBuilder {
    fn F_cNew() -> Self {
        CAsmBuilder { 
            l_sData: String::from("section .bss\n    dump_buf: resb 21\n    data_stack: resq 4096\n    data_stack_proc: resq 8192\nsection .data\n"),
            l_sCode: String::new(),
            l_iFuncCode: String::new(),
            l_iStrCount: 0,
        }
    }

    fn F_vEmitDataLine(&mut self, l_sLine: &str) {
        let _ = writeln!(self.l_sData, "{}", l_sLine);
    }

    fn F_sAddString(&mut self, l_sStr: &str) -> (String, usize) {
        let l_sLabel = format!("str_{}", self.l_iStrCount);
        self.l_iStrCount += 1;
        let l_iLen = l_sStr.len();
        
        let mut l_sEscaped = String::new();
        for l_cByte in l_sStr.bytes() {
            if l_cByte >= 32 && l_cByte <= 126 && l_cByte != b'"' && l_cByte != b'\\' {
                l_sEscaped.push(l_cByte as char);
            } else {
                l_sEscaped.push_str(&format!("\", {}, \"", l_cByte));
            }
        }
        
        self.F_vEmitDataLine(&format!("    {}: db \"{}\", 0", l_sLabel, l_sEscaped));
        (l_sLabel, l_iLen)
    }

    fn F_vEmitLine(&mut self, l_sLine: &str) {
        let _ = writeln!(self.l_sCode, "{}", l_sLine);
    }

    fn F_vEmitInstr(&mut self, l_sInstr: &str, l_sArgs: &str) {
        let _ = writeln!(self.l_sCode, "    {:8} {}", l_sInstr, l_sArgs);
    }

    fn F_vEmitFuncLine(&mut self, l_sLine: &str) {
        let _ = writeln!(self.l_iFuncCode, "{}", l_sLine);
    }

    fn F_vEmitFuncInstr(&mut self, l_sInstr: &str, l_sArgs: &str) {
        let _ = writeln!(self.l_iFuncCode, "    {:8} {}", l_sInstr, l_sArgs);
    }

    fn vEmitFuncLines(&mut self, l_sLines: &str) {
        let _ = writeln!(self.l_iFuncCode, "\n{}\n", l_sLines);
    }

    fn F_vEmitExit(&mut self) {
        self.F_vEmitInstr("mov", "rax, 60");
        self.F_vEmitInstr("xor", "rdi, rdi");
        self.F_vEmitInstr("syscall", "");
    }

    fn F_sBuild(self) -> String {
        format!("{}\nsection .text\n{}\nglobal _start\n_start:\n    lea      r15, [data_stack + 4096*8]\n    call     proc_main\n{}", self.l_sData, self.l_iFuncCode, self.l_sCode)
    }

    fn F_vEmitDumpFunction(&mut self) {
        self.vEmitFuncLines("dump_i:
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
");
    }

    fn F_vEmitSyscall3(&mut self, l_bInProc: bool, pile: &str) {
        if l_bInProc {
            self.F_vEmitFuncInstr("mov", &format!("rax, [{}]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rdi, [{}]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rsi, [{}]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rdx, [{}]", pile));
            self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 32"));
            self.F_vEmitFuncInstr("syscall", "");
            self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, ", 8"));
            self.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
            return;
        }
        self.F_vEmitInstr("mov", &format!("rax, [{} + 24]", pile));
        self.F_vEmitInstr("mov", &format!("rdi, [{} + 16]", pile));
        self.F_vEmitInstr("mov", &format!("rsi, [{} + 8]", pile));
        self.F_vEmitInstr("mov", &format!("rdx, [{}]", pile));
        self.F_vEmitInstr("add", &format!("{}{}", pile, ", 32"));
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("sub", &format!("{}{}", pile, ", 8"));
        self.F_vEmitInstr("mov", &format!("[{}], rax", pile));
    }

    fn F_vEmitSyscall0(&mut self, l_bInProc: bool, pile: &str) {
        if l_bInProc {
            self.F_vEmitFuncInstr("mov", &format!("rax, [{}]", pile));
            self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 8"));
            self.F_vEmitFuncInstr("syscall", "");
            self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, ", 8"));
            self.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
            return;
        }
        self.F_vEmitInstr("mov", &format!("rax, [{}]", pile));
        self.F_vEmitInstr("add", &format!("{}{}", pile, ", 8"));
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("sub", &format!("{}{}", pile, ", 8"));
        self.F_vEmitInstr("mov", &format!("[{}], rax", pile));
    }

    fn F_vEmitSyscall1(&mut self, l_bInProc: bool, pile: &str) {
        if l_bInProc {
            self.F_vEmitFuncInstr("mov", &format!("rax, [{} + 8]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rdi, [{}]", pile));
            self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 16"));
            self.F_vEmitFuncInstr("syscall", "");
            self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, ", 8"));
            self.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
            return;
        }
        self.F_vEmitInstr("mov", &format!("rax, [{} + 8]", pile));
        self.F_vEmitInstr("mov", &format!("rdi, [{}]", pile));
        self.F_vEmitInstr("add", &format!("{}{}", pile, ", 16"));
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("sub", &format!("{}{}", pile, ", 8"));
        self.F_vEmitInstr("mov", &format!("[{}], rax", pile));
    }

    fn F_vEmitSyscall2(&mut self, l_bInProc: bool, pile: &str) {
        if l_bInProc {
            self.F_vEmitFuncInstr("mov", &format!("rax, [{} + 16]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rdi, [{} + 8]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rsi, [{}]", pile));
            self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 24"));
            self.F_vEmitFuncInstr("syscall", "");
            self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, ", 8"));
            self.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
            return;
        }
        self.F_vEmitInstr("mov", &format!("rax, [{} + 16]", pile));
        self.F_vEmitInstr("mov", &format!("rdi, [{} + 8]", pile));
        self.F_vEmitInstr("mov", &format!("rsi, [{}]", pile));
        self.F_vEmitInstr("add", &format!("{}{}", pile, ", 24"));
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("sub", &format!("{}{}", pile, ", 8"));
        self.F_vEmitInstr("mov", &format!("[{}], rax", pile));
    }

    fn F_vEmitSyscall4(&mut self, l_bInProc: bool, pile: &str) {
        if l_bInProc {
            self.F_vEmitFuncInstr("mov", &format!("rax, [{} + 32]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rdi, [{} + 24]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rsi, [{} + 16]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rdx, [{} + 8]", pile));
            self.F_vEmitFuncInstr("mov", &format!("r10, [{}]", pile));
            self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 40"));
            self.F_vEmitFuncInstr("syscall", "");
            self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, ", 8"));
            self.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
            return;
        }
        self.F_vEmitInstr("mov", &format!("rax, [{} + 32]", pile));
        self.F_vEmitInstr("mov", &format!("rdi, [{} + 24]", pile));
        self.F_vEmitInstr("mov", &format!("rsi, [{} + 16]", pile));
        self.F_vEmitInstr("mov", &format!("rdx, [{} + 8]", pile));
        self.F_vEmitInstr("mov", &format!("r10, [{}]", pile));
        self.F_vEmitInstr("add", &format!("{}{}", pile, ", 40"));
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("sub", &format!("{}{}", pile, ", 8"));
        self.F_vEmitInstr("mov", &format!("[{}], rax", pile));
    }

    fn F_vEmitSyscall5(&mut self, l_bInProc: bool, pile: &str) {
        if l_bInProc {
            self.F_vEmitFuncInstr("mov", &format!("rax, [{} + 40]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rdi, [{} + 32]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rsi, [{} + 24]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rdx, [{} + 16]", pile));
            self.F_vEmitFuncInstr("mov", &format!("r10, [{} + 8]", pile));
            self.F_vEmitFuncInstr("mov", &format!("r8, [{}]", pile));
            self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 48"));
            self.F_vEmitFuncInstr("syscall", "");
            self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, ", 8"));
            self.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
            return;
        }
        self.F_vEmitInstr("mov", &format!("rax, [{} + 40]", pile));
        self.F_vEmitInstr("mov", &format!("rdi, [{} + 32]", pile));
        self.F_vEmitInstr("mov", &format!("rsi, [{} + 24]", pile));
        self.F_vEmitInstr("mov", &format!("rdx, [{} + 16]", pile));
        self.F_vEmitInstr("mov", &format!("r10, [{} + 8]", pile));
        self.F_vEmitInstr("mov", &format!("r8, [{}]", pile));
        self.F_vEmitInstr("add", &format!("{}{}", pile, ", 48"));
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("sub", &format!("{}{}", pile, ", 8"));
        self.F_vEmitInstr("mov", &format!("[{}], rax", pile));
    }

    fn F_vEmitSyscall6(&mut self, l_bInProc: bool, pile: &str) {
        if l_bInProc {
            self.F_vEmitFuncInstr("mov", &format!("rax, [{} + 48]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rdi, [{} + 40]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rsi, [{} + 32]", pile));
            self.F_vEmitFuncInstr("mov", &format!("rdx, [{} + 24]", pile));
            self.F_vEmitFuncInstr("mov", &format!("r10, [{} + 16]", pile));
            self.F_vEmitFuncInstr("mov", &format!("r8, [{} + 8]", pile));
            self.F_vEmitFuncInstr("mov", &format!("r9, [{}]", pile));
            self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 56"));
            self.F_vEmitFuncInstr("syscall", "");
            self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, ", 8"));
            self.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
            return;
        }
        self.F_vEmitInstr("mov", &format!("rax, [{} + 48]", pile));
        self.F_vEmitInstr("mov", &format!("rdi, [{} + 40]", pile));
        self.F_vEmitInstr("mov", &format!("rsi, [{} + 32]", pile));
        self.F_vEmitInstr("mov", &format!("rdx, [{} + 24]", pile));
        self.F_vEmitInstr("mov", &format!("r10, [{} + 16]", pile));
        self.F_vEmitInstr("mov", &format!("r8, [{} + 8]", pile));
        self.F_vEmitInstr("mov", &format!("r9, [{}]", pile));
        self.F_vEmitInstr("add", &format!("{}{}", pile, ", 56"));
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("sub", &format!("{}{}", pile, ", 8"));
        self.F_vEmitInstr("mov", &format!("[{}], rax", pile));
    }

    // generate cast for EType
    pub fn F_vEmitCast(&mut self, l_eFrom: &EType, l_eTo: &EType, l_bInProc: bool, pile: &str) {
        /*
        Architecture flottante x86_64

Registres: xmm0 à xmm15 (128 bits chacun).

​

    F32 (simple précision) : 32 bits bas

    F64 (double précision) : 64 bits bas

    ​

Instructions arithmétiques SSE2
Double précision (f64)

text
addsd xmm0, xmm1    # xmm0 += xmm1
subsd xmm0, xmm1    # xmm0 -= xmm1
mulsd xmm0, xmm1    # xmm0 *= xmm1
divsd xmm0, xmm1    # xmm0 /= xmm1
sqrtsd xmm0, xmm1   # xmm0 = sqrt(xmm1)

Simple précision (f32)

text
addss xmm0, xmm1    # xmm0 += xmm1
subss xmm0, xmm1    # xmm0 -= xmm1
mulss xmm0, xmm1    # xmm0 *= xmm1
divss xmm0, xmm1    # xmm0 /= xmm1
sqrtss xmm0, xmm1   # xmm0 = sqrt(xmm1)

Transfert mémoire ↔ registres

text
# Charger depuis pile (mémoire)
movsd xmm0, [r15]        # charge f64
movss xmm0, [r15]        # charge f32

# Stocker vers pile
movsd [r15], xmm0        # stocke f64
movss [r15], xmm0        # stocke f32

# Copie entre registres XMM
movsd xmm0, xmm1
movaps xmm0, xmm1        # alternative alignée
        */
        if !l_bInProc { return; }
    
        match (l_eFrom, l_eTo) {
            (EType::I64, EType::F64) => {
                self.F_vEmitFuncInstr("cvtsi2sd", &format!("xmm0, qword [{}]", pile));
                self.F_vEmitFuncInstr("movsd", &format!("[{}], xmm0", pile));
            },
            (EType::I32, EType::F64) => {
                self.F_vEmitFuncInstr("cvtsi2sd", &format!("xmm0, dword [{}]", pile));
                self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 4"));
                self.F_vEmitFuncInstr("movsd", &format!("[{}], xmm0", pile));
            },
            (EType::I64, EType::F32) => {
                self.F_vEmitFuncInstr("cvtsi2ss", &format!("xmm0, qword [{}]", pile));
                self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 4"));
                self.F_vEmitFuncInstr("movss", &format!("[{}], xmm0", pile));
            },
            (EType::I32, EType::F32) => {
                self.F_vEmitFuncInstr("cvtsi2ss", &format!("xmm0, dword [{}]", pile));
                self.F_vEmitFuncInstr("movss", &format!("[{}], xmm0", pile));
            },
            (EType::F64, EType::I64) => {
                self.F_vEmitFuncInstr("movsd", &format!("xmm0, [{}]", pile));
                self.F_vEmitFuncInstr("cvttsd2si", "rax, xmm0");
                self.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
            },
            (EType::F64, EType::I32) => {
                self.F_vEmitFuncInstr("movsd", &format!("xmm0, [{}]", pile));
                self.F_vEmitFuncInstr("cvttsd2si", "eax, xmm0");
                self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, ", 4"));
                self.F_vEmitFuncInstr("mov", &format!("dword [{}], eax", pile));
            },
            (EType::F32, EType::I64) => {
                self.F_vEmitFuncInstr("movss", &format!("xmm0, [{}]", pile));
                self.F_vEmitFuncInstr("cvttss2si", "rax, xmm0");
                self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, ", 4"));
                self.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
            },
            (EType::F32, EType::I32) => {
                self.F_vEmitFuncInstr("movss", &format!("xmm0, [{}]", pile));
                self.F_vEmitFuncInstr("cvttss2si", "eax, xmm0");
                self.F_vEmitFuncInstr("mov", &format!("dword [{}], eax", pile));
            },
            (EType::F32, EType::F64) => {
                self.F_vEmitFuncInstr("movss", &format!("xmm0, [{}]", pile));
                self.F_vEmitFuncInstr("cvtss2sd", "xmm0, xmm0");
                self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 4"));
                self.F_vEmitFuncInstr("movsd", &format!("[{}], xmm0", pile));
            },
            (EType::F64, EType::F32) => {
                self.F_vEmitFuncInstr("movsd", &format!("xmm0, [{}]", pile));
                self.F_vEmitFuncInstr("cvtsd2ss", "xmm0, xmm0");
                self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, ", 4"));
                self.F_vEmitFuncInstr("movss", &format!("[{}], xmm0", pile));
            },
            (EType::I32, EType::I64) | (EType::I16, EType::I64) | (EType::I8, EType::I64) => {
                let l_sSize = match l_eFrom {
                    EType::I32 => "dword",
                    EType::I16 => "word",
                    EType::I8 => "byte",
                    _ => unreachable!(),
                };
                self.F_vEmitFuncInstr("movsx", &format!("rax, {} [{}]", l_sSize, pile));
                self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 4"));
                self.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
            },
            (EType::U32, EType::I64) | (EType::U32, EType::U64) | 
            (EType::U16, EType::I64) | (EType::U16, EType::U64) |
            (EType::U8, EType::I64) | (EType::U8, EType::U64) => {
                let l_sSize = match l_eFrom {
                    EType::U32 => "dword",
                    EType::U16 => "word",
                    EType::U8 => "byte",
                    _ => unreachable!(),
                };
                self.F_vEmitFuncInstr("movzx", &format!("rax, {} [{}]", l_sSize, pile));
                self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 4"));
                self.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
            },
            (EType::I64, EType::I32) | (EType::U64, EType::I32) => {
                self.F_vEmitFuncInstr("mov", &format!("eax, dword [{}]", pile));
                self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, ", 4"));
                self.F_vEmitFuncInstr("mov", &format!("dword [{}], eax", pile));
            },
            (EType::I32, EType::I16) | (EType::I32, EType::U16) => {
                self.F_vEmitFuncInstr("mov", &format!("ax, word [{}]", pile));
                self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, ", 6"));
                self.F_vEmitFuncInstr("mov", &format!("word [{}], ax", pile));
            },
            (EType::I32, EType::I8) | (EType::I32, EType::U8) |
            (EType::I16, EType::I8) | (EType::I16, EType::U8) => {
                self.F_vEmitFuncInstr("mov", &format!("al, byte [{}]", pile));
                let l_iOffset = match l_eFrom {
                    EType::I32 => 3,
                    EType::I16 => 1,
                    _ => unreachable!(),
                };
                self.F_vEmitFuncInstr("sub", &format!("{}{}", pile, l_iOffset));
                self.F_vEmitFuncInstr("mov", &format!("byte [{}], al", pile));
            },
            (EType::I16, EType::I32) | (EType::I8, EType::I32) => {
                let l_sSize = if *l_eFrom == EType::I16 { "word" } else { "byte" };
                self.F_vEmitFuncInstr("movsx", &format!("eax, {} [{}]", l_sSize, pile));
                let l_iOffset = if *l_eFrom == EType::I16 { 2 } else { 3 };
                self.F_vEmitFuncInstr("add", &format!("{}{}", pile, l_iOffset));
                self.F_vEmitFuncInstr("mov", &format!("dword [{}], eax", pile));
            },
            (EType::U16, EType::I32) | (EType::U16, EType::U32) |
            (EType::U8, EType::I32) | (EType::U8, EType::U32) => {
                let l_sSize = if matches!(l_eFrom, EType::U16) { "word" } else { "byte" };
                self.F_vEmitFuncInstr("movzx", &format!("eax, {} [{}]", l_sSize, pile));
                let l_iOffset = if matches!(l_eFrom, EType::U16) { 2 } else { 3 };
                self.F_vEmitFuncInstr("add", &format!("{}{}", pile, l_iOffset));
                self.F_vEmitFuncInstr("mov", &format!("dword [{}], eax", pile));
            },
            (EType::I8, EType::I16) | (EType::U8, EType::I16) | (EType::U8, EType::U16) => {
                let l_sInstr = if *l_eFrom == EType::I8 { "movsx" } else { "movzx" };
                self.F_vEmitFuncInstr(l_sInstr, &format!("ax, byte [{}]", pile));
                self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 1"));
                self.F_vEmitFuncInstr("mov", &format!("word [{}], ax", pile));
            },
            (EType::Bool, EType::I32) | (EType::Bool, EType::I64) => {
                self.F_vEmitFuncInstr("movzx", &format!("rax, byte [{}]", pile));
                let l_iSize = if *l_eTo == EType::I64 { 7 } else { 3 };
                self.F_vEmitFuncInstr("add", &format!("{}{}", pile, l_iSize));
                let l_sInstr = if *l_eTo == EType::I64 { "mov" } else { "mov dword" };
                self.F_vEmitFuncInstr(l_sInstr, &format!("[{}], rax", pile));
            },
            // i64 to i8
            (EType::I64, EType::I8) => {
                let l_sSize = if *l_eFrom == EType::I64 { "qword" } else { "dword" };
                self.F_vEmitFuncInstr("mov", "al, 0");
                self.F_vEmitFuncInstr("cmp", &format!("{} [{}], 0", l_sSize, pile));
                self.F_vEmitFuncInstr("setne", "al");
                self.F_vEmitFuncInstr("add", &format!("{}{}", pile, ", 1"));
                self.F_vEmitFuncInstr("mov", &format!("byte [{}], al", pile));
            },
            (l_eA, l_eB) if l_eA == l_eB => {},
            _ => {
            },
        }
    }
}

struct CStackToX86_64;

impl CStackToX86_64 {
    fn F_sCompile(l_lIr: &[EIrInstr]) -> Result<String, String> {
        let mut l_cAsm = CAsmBuilder::F_cNew();
        let mut l_hmProcs: HashMap<&'static str, &Vec<EIrInstr>> = HashMap::new();
        let mut l_hmInfoProc: HashMap<&'static str, EIrInstr> = HashMap::new();
        let mut l_bHasMain = false;

        l_cAsm.F_vEmitDumpFunction();

        for l_cInstr in l_lIr {
            if let EIrInstr::Proc(l_sName, l_lBody, l_lTypes, l_eRetType) = l_cInstr {
                if *l_sName == "main" {
                    l_bHasMain = true;
                }
                l_hmProcs.insert(l_sName, l_lBody);
                l_hmInfoProc.insert(l_sName, l_cInstr.clone());
            } else if let EIrInstr::Const(l_sName, l_lBody) = l_cInstr {
                l_hmProcs.insert(l_sName, l_lBody);
                l_hmInfoProc.insert(l_sName, EIrInstr::Proc(l_sName, l_lBody.clone(), vec![EType::Void], EType::Void));
            }
        }

        if !l_bHasMain {
            return Err("Erreur: proc main non declaree".to_string());
        }

        for (l_sName, l_lBody) in &l_hmProcs {
            l_cAsm.F_vEmitFuncLine(&format!("proc_{}:", l_sName));

            let l_mhInfo = l_hmInfoProc.get(l_sName).unwrap();
            let l_bIsMain = *l_sName == "main";

            if !l_bIsMain {
                l_cAsm.F_vEmitFuncInstr("lea", "r14, [data_stack_proc + 8192*8]");
            }

            Self::F_vCompileInstrs(&mut l_cAsm, l_lBody, true, l_mhInfo.clone(), l_bIsMain);
        }

        l_cAsm.F_vEmitExit();
        Ok(l_cAsm.F_sBuild())
    }

    fn F_vCompileInstrs(l_cAsm: &mut CAsmBuilder, l_lInstrs: &[EIrInstr], l_bInProc: bool, l_mhInfo: EIrInstr, l_bIsMain: bool) {
        let pile = if l_bIsMain { "r15" } else { "r15" };

        if l_bInProc && !l_bIsMain {
            l_cAsm.F_vEmitFuncInstr("push", "r15");  // sauvegarde pointeur pile
        }
        
        for l_cInstr in l_lInstrs {
            match *l_cInstr {
                EIrInstr::PushI64(l_iVal) => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("sub", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("qword [{}], {}", pile, l_iVal));
                    } else {
                        l_cAsm.F_vEmitInstr("sub", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitInstr("mov", &format!("qword [{}], {}", pile, l_iVal));
                    }
                }
                EIrInstr::PushStr(l_sStr) => {
                    let (l_sLabel, _) = l_cAsm.F_sAddString(l_sStr);
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("sub", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("qword [{}], {}", pile, l_sLabel));
                    } else {
                        l_cAsm.F_vEmitInstr("sub", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitInstr("mov", &format!("qword [{}], {}", pile, l_sLabel));
                    }
                }
                EIrInstr::AddI64 => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("rax, [{}]", pile));
                        l_cAsm.F_vEmitFuncInstr("add", &format!("rax, [{} + 8]", pile));
                        l_cAsm.F_vEmitFuncInstr("add", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
                    } else {
                        l_cAsm.F_vEmitInstr("mov", &format!("rax, [{}]", pile));
                        l_cAsm.F_vEmitInstr("add", &format!("rax, [{} + 8]", pile));
                        l_cAsm.F_vEmitInstr("add", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitInstr("mov", &format!("[{}], rax", pile));
                    }
                }
                EIrInstr::SubI64 => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("rax, [{} + 8]", pile));
                        l_cAsm.F_vEmitFuncInstr("sub", &format!("rax, [{}]", pile));
                        l_cAsm.F_vEmitFuncInstr("add", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
                    } else {
                        l_cAsm.F_vEmitInstr("mov", &format!("rax, [{} + 8]", pile));
                        l_cAsm.F_vEmitInstr("sub", &format!("rax, [{}]", pile));
                        l_cAsm.F_vEmitInstr("add", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitInstr("mov", &format!("[{}], rax", pile));
                    }
                }
                EIrInstr::Swap => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("rax, [{}]", pile));
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("rbx, [{} + 8]", pile));
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("[{}], rbx", pile));
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("[{} + 8], rax", pile));
                    } else {
                        l_cAsm.F_vEmitInstr("mov", &format!("rax, [{}]", pile));
                        l_cAsm.F_vEmitInstr("mov", &format!("rbx, [{} + 8]", pile));
                        l_cAsm.F_vEmitInstr("mov", &format!("[{}], rbx", pile));
                        l_cAsm.F_vEmitInstr("mov", &format!("[{} + 8], rax", pile));
                    }
                }
                EIrInstr::Dup => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("rax, [{}]", pile));
                        l_cAsm.F_vEmitFuncInstr("sub", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("[{}], rax", pile));
                    } else {
                        l_cAsm.F_vEmitInstr("mov", &format!("rax, [{}]", pile));
                        l_cAsm.F_vEmitInstr("sub", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitInstr("mov", &format!("[{}], rax", pile));
                    }
                }
                EIrInstr::Dump => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("rdi, [{}]", pile));
                        l_cAsm.F_vEmitFuncInstr("add", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitFuncInstr("call", "dump_i");
                    } else {
                        l_cAsm.F_vEmitInstr("mov", &format!("rdi, [{}]", pile));
                        l_cAsm.F_vEmitInstr("add", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitInstr("call", "dump_i");
                    }
                }
                EIrInstr::Puts => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("rdi, [{}]", pile));
                        l_cAsm.F_vEmitFuncInstr("add", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitFuncInstr("call", "dump_str");
                    } else {
                        l_cAsm.F_vEmitInstr("mov", &format!("rdi, [{}]", pile));
                        l_cAsm.F_vEmitInstr("add", &format!("{}, 8", pile));
                        l_cAsm.F_vEmitInstr("call", "dump_str");
                    }
                }
                EIrInstr::Call(l_sName) => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("call", &format!("proc_{}", l_sName));
                    } else {
                        l_cAsm.F_vEmitInstr("call", &format!("proc_{}", l_sName));
                    }
                }
                /*EIrInstr::Ret => {
                    if l_bInProc {
                        /*if !l_bIsMain {
                            // Transfert pile locale (r14) → pile principale (r15)
                            l_cAsm.F_vEmitFuncInstr("mov", "rax, [r14]");
                            l_cAsm.F_vEmitFuncInstr("sub", "r15, 8");
                            l_cAsm.F_vEmitFuncInstr("mov", "[r15], rax");
                        }*/
                        if !l_bIsMain {
                            // Nettoyer stack frame et garder uniquement résultat
                            l_cAsm.F_vEmitFuncInstr("mov", "rax, [r15]");   // résultat
                            l_cAsm.F_vEmitFuncInstr("pop", "r15");          // restaure pile
                            l_cAsm.F_vEmitFuncInstr("sub", "r15, 8");       // alloue
                            l_cAsm.F_vEmitFuncInstr("mov", "[r15], rax");   // push résultat
                        }
                        l_cAsm.F_vEmitFuncInstr("ret", "");
                    } else {
                        /*if !l_bIsMain {
                            // Transfert pile locale (r14) → pile principale (r15)
                            l_cAsm.F_vEmitInstr("mov", "rax, [r14]");
                            l_cAsm.F_vEmitInstr("sub", "r15, 8");
                            l_cAsm.F_vEmitInstr("mov", "[r15], rax");
                        }*/
                        l_cAsm.F_vEmitInstr("ret", "");
                    }
                }*/
                EIrInstr::Ret => {
                    if l_bInProc {
                        if !l_bIsMain {
                            l_cAsm.F_vEmitFuncInstr("mov", "rax, [r15]");
                            l_cAsm.F_vEmitFuncInstr("pop", "r15");
                            l_cAsm.F_vEmitFuncInstr("mov", "[r15], rax");  // ✓ Remplace argument
                        }
                        l_cAsm.F_vEmitFuncInstr("ret", "");
                    } else {
                        l_cAsm.F_vEmitInstr("ret", "");
                    }
                }

                EIrInstr::Syscall0 => l_cAsm.F_vEmitSyscall0(l_bInProc, pile),
                EIrInstr::Syscall1 => l_cAsm.F_vEmitSyscall1(l_bInProc, pile),
                EIrInstr::Syscall2 => l_cAsm.F_vEmitSyscall2(l_bInProc, pile),
                EIrInstr::Syscall3 => l_cAsm.F_vEmitSyscall3(l_bInProc, pile),
                EIrInstr::Syscall4 => l_cAsm.F_vEmitSyscall4(l_bInProc, pile),
                EIrInstr::Syscall5 => l_cAsm.F_vEmitSyscall5(l_bInProc, pile),
                EIrInstr::Syscall6 => l_cAsm.F_vEmitSyscall6(l_bInProc, pile),
                EIrInstr::Proc(_, _, _, _) | EIrInstr::Const(_, _) => {
                    panic!("Instruction non supportee in statement");
                }
                _ => { panic!("Instruction non supportee"); }
            }
        }
    }
}

struct CStackToInterpreter;

impl CStackToInterpreter {
    fn F_vInterpret(l_lIr: &[EIrInstr]) -> Result<(), String> {
        let mut l_lDataStack: Vec<i64> = Vec::new();
        let mut l_hmProcs: HashMap<&'static str, &Vec<EIrInstr>> = HashMap::new();
        let mut l_hmInfoProc: HashMap<&'static str, EIrInstr> = HashMap::new();
        let mut l_bHasMain = false;

        for l_cInstr in l_lIr {
            if let EIrInstr::Proc(l_sName, l_lBody, l_lTypes, l_eRetType) = l_cInstr {
                if *l_sName == "main" { l_bHasMain = true; }
                l_hmProcs.insert(l_sName, l_lBody);
                l_hmInfoProc.insert(l_sName, l_cInstr.clone());
            } else if let EIrInstr::Const(l_sName, l_lBody) = l_cInstr {
                l_hmProcs.insert(l_sName, l_lBody);
                l_hmInfoProc.insert(l_sName, EIrInstr::Proc(l_sName, l_lBody.clone(), vec![EType::Void], EType::Void));
            }
        }

        if !l_bHasMain { return Err("Erreur: proc main non declaree".to_string()); }

        Self::F_vExecuteProc("main", &l_hmProcs, &l_hmInfoProc, &mut l_lDataStack, true)?;
        Ok(())
    }

    fn F_vExecuteProc(
        l_sName: &str,
        l_hmProcs: &HashMap<&'static str, &Vec<EIrInstr>>,
        l_hmInfo: &HashMap<&'static str, EIrInstr>,
        l_lDataStack: &mut Vec<i64>,
        l_bIsMain: bool
    ) -> Result<(), String> {
        let l_lBody = l_hmProcs.get(l_sName)
            .ok_or_else(|| format!("Proc {} non trouvee", l_sName))?;

        let l_iStackBase = l_lDataStack.len();

        for l_cInstr in l_lBody.iter() {
            match *l_cInstr {
                EIrInstr::PushI64(l_iVal) => l_lDataStack.push(l_iVal),
                EIrInstr::PushStr(l_sStr) => {
                    l_lDataStack.push(l_sStr.as_ptr() as i64);
                    l_lDataStack.push(l_sStr.len() as i64);
                }
                EIrInstr::AddI64 => {
                    let l_iB = l_lDataStack.pop().ok_or("Stack underflow add")?;
                    let l_iA = l_lDataStack.pop().ok_or("Stack underflow add")?;
                    l_lDataStack.push(l_iA + l_iB);
                }
                EIrInstr::SubI64 => {
                    let l_iB = l_lDataStack.pop().ok_or("Stack underflow sub")?;
                    let l_iA = l_lDataStack.pop().ok_or("Stack underflow sub")?;
                    l_lDataStack.push(l_iA - l_iB);
                }
                EIrInstr::Swap => {
                    let l_iB = l_lDataStack.pop().ok_or("Stack underflow swap")?;
                    let l_iA = l_lDataStack.pop().ok_or("Stack underflow swap")?;
                    l_lDataStack.push(l_iB);
                    l_lDataStack.push(l_iA);
                }
                EIrInstr::Dup => {
                    let l_iTop = *l_lDataStack.last().ok_or("Stack underflow dup")?;
                    l_lDataStack.push(l_iTop);
                }
                EIrInstr::Puts => {
                    let l_iLen = l_lDataStack.pop().ok_or("Stack underflow puts")? as usize;
                    let l_pBuf = l_lDataStack.pop().ok_or("Stack underflow puts")? as *const u8;
                    let l_sStr = unsafe {
                        std::str::from_utf8_unchecked(std::slice::from_raw_parts(l_pBuf, l_iLen))
                    };
                    print!("{}", l_sStr);
                }
                EIrInstr::Dump => {
                    let l_iVal = l_lDataStack.pop().ok_or("Stack underflow dump")?;
                    print!("{}", l_iVal);
                }
                EIrInstr::Call(l_sTarget) => {
                    Self::F_vExecuteProc(l_sTarget, l_hmProcs, l_hmInfo, l_lDataStack, false)?;
                }
                EIrInstr::Ret => {
                    if !l_bIsMain {
                        if l_lDataStack.len() < l_iStackBase + 1 {
                            println!("Warning: Stack underflow ret");
                            return Ok(());
                        }
                        let l_iResult = l_lDataStack.pop().ok_or("Stack underflow ret")?;
                        l_lDataStack.truncate(l_iStackBase);
                        l_lDataStack.push(l_iResult);
                    }
                    return Ok(());
                }
                EIrInstr::Syscall0 => {
                    let l_iSysno = l_lDataStack.pop().ok_or("Stack underflow syscall0")? as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno) };
                    l_lDataStack.push(l_iRet as i64);
                }
                EIrInstr::Syscall1 => {
                    let l_iArg1 = l_lDataStack.pop().ok_or("Stack underflow syscall1")? as libc::c_long;
                    let l_iSysno = l_lDataStack.pop().ok_or("Stack underflow syscall1")? as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno, l_iArg1) };
                    l_lDataStack.push(l_iRet as i64);
                }
                EIrInstr::Syscall2 => {
                    let l_iArg2 = l_lDataStack.pop().ok_or("Stack underflow syscall2")? as libc::c_long;
                    let l_iArg1 = l_lDataStack.pop().ok_or("Stack underflow syscall2")? as libc::c_long;
                    let l_iSysno = l_lDataStack.pop().ok_or("Stack underflow syscall2")? as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno, l_iArg1, l_iArg2) };
                    l_lDataStack.push(l_iRet as i64);
                }
                EIrInstr::Syscall3 => {
                    let l_iArg3 = l_lDataStack.pop().ok_or("Stack underflow syscall3")? as libc::c_long;
                    let l_iArg2 = l_lDataStack.pop().ok_or("Stack underflow syscall3")? as libc::c_long;
                    let l_iArg1 = l_lDataStack.pop().ok_or("Stack underflow syscall3")? as libc::c_long;
                    let l_iSysno = l_lDataStack.pop().ok_or("Stack underflow syscall3")? as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno, l_iArg1, l_iArg2, l_iArg3) };
                    l_lDataStack.push(l_iRet as i64);
                }
                EIrInstr::Syscall4 => {
                    let l_iArg4 = l_lDataStack.pop().ok_or("Stack underflow syscall4")? as libc::c_long;
                    let l_iArg3 = l_lDataStack.pop().ok_or("Stack underflow syscall4")? as libc::c_long;
                    let l_iArg2 = l_lDataStack.pop().ok_or("Stack underflow syscall4")? as libc::c_long;
                    let l_iArg1 = l_lDataStack.pop().ok_or("Stack underflow syscall4")? as libc::c_long;
                    let l_iSysno = l_lDataStack.pop().ok_or("Stack underflow syscall4")? as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno, l_iArg1, l_iArg2, l_iArg3, l_iArg4) };
                    l_lDataStack.push(l_iRet as i64);
                }
                EIrInstr::Syscall5 => {
                    let l_iArg5 = l_lDataStack.pop().ok_or("Stack underflow syscall5")? as libc::c_long;
                    let l_iArg4 = l_lDataStack.pop().ok_or("Stack underflow syscall5")? as libc::c_long;
                    let l_iArg3 = l_lDataStack.pop().ok_or("Stack underflow syscall5")? as libc::c_long;
                    let l_iArg2 = l_lDataStack.pop().ok_or("Stack underflow syscall5")? as libc::c_long;
                    let l_iArg1 = l_lDataStack.pop().ok_or("Stack underflow syscall5")? as libc::c_long;
                    let l_iSysno = l_lDataStack.pop().ok_or("Stack underflow syscall5")? as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno, l_iArg1, l_iArg2, l_iArg3, l_iArg4, l_iArg5) };
                    l_lDataStack.push(l_iRet as i64);
                }
                EIrInstr::Syscall6 => {
                    let l_iArg6 = l_lDataStack.pop().ok_or("Stack underflow syscall6")? as libc::c_long;
                    let l_iArg5 = l_lDataStack.pop().ok_or("Stack underflow syscall6")? as libc::c_long;
                    let l_iArg4 = l_lDataStack.pop().ok_or("Stack underflow syscall6")? as libc::c_long;
                    let l_iArg3 = l_lDataStack.pop().ok_or("Stack underflow syscall6")? as libc::c_long;
                    let l_iArg2 = l_lDataStack.pop().ok_or("Stack underflow syscall6")? as libc::c_long;
                    let l_iArg1 = l_lDataStack.pop().ok_or("Stack underflow syscall6")? as libc::c_long;
                    let l_iSysno = l_lDataStack.pop().ok_or("Stack underflow syscall6")? as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno, l_iArg1, l_iArg2, l_iArg3, l_iArg4, l_iArg5, l_iArg6) };
                    l_lDataStack.push(l_iRet as i64);
                }
                _ => {}
            }
        }
        Ok(())
    }

}


fn main() {
    // get argument filename
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <source_file>", args[0]);
        return;
    }

    let l_sFilename = &args[1];
    let l_sCode = std::fs::read_to_string(l_sFilename)
        .expect("Erreur lecture fichier");

    match F_lParseProgram(l_sCode.as_str()) {
        Ok(l_lProgram) => {
            println!("Programme parsé!");

            //=== IR ===
            println!("\n=== IR ===");
            for l_cInstr in &l_lProgram {
                // si l'instruction est une proc/const, afficher son nom et son corps
                match l_cInstr {
                    EIrInstr::Proc(l_sName, l_lBody, l_lTypes, l_eRetType) => {
                        //proc name (param)
                        print!("Proc {} (", l_sName);
                        for (i, l_cType) in l_lTypes.iter().enumerate() {
                            if i > 0 {
                                print!(", ");
                            }
                            print!("{:?}", l_cType);
                        }
                        println!(") -> {:?}", l_eRetType);
                        for l_cBodyInstr in l_lBody {
                            println!("    {:?}", l_cBodyInstr);
                        }
                    },
                    _ => {
                        println!("{:?}", l_cInstr);
                    }
                }
            }
            
            println!("=== INTERPRETATION ===");
            match CStackToInterpreter::F_vInterpret(&l_lProgram) {
                Ok(_) => {},
                Err(e) => eprintln!("Erreur: {}", e),
            }
            
            println!("\n=== COMPILATION X86_64 ===");
            match CStackToX86_64::F_sCompile(&l_lProgram) {
                Ok(l_sAsm) => {
                    std::fs::write("out.asm", l_sAsm).unwrap();
                },
                Err(e) => eprintln!("Erreur: {}", e),
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}