use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

// ============================================================================
// Stack-Based Assembly Generator for x86_64
// ============================================================================

pub struct StackAsmBuilder {
    pub bss: Vec<String>,
    pub data: Vec<String>,
    pub funcs: Vec<String>,
    pub text: Vec<String>,
    pub str_count: usize,
    pub stack_reg: String,       // r15 pour pile principale
    pub proc_stack_reg: String,  // r14 pour pile proc
}

impl StackAsmBuilder {
    pub fn new() -> Self {
        Self {
            bss: vec![
                "    dump_buf: resb 21".to_string(),
                "    data_stack: resq 4096".to_string(),
                "    data_stack_proc: resq 8192".to_string(),
            ],
            data: Vec::new(),
            funcs: Vec::new(),
            text: Vec::new(),
            str_count: 0,
            stack_reg: "r15".to_string(),
            proc_stack_reg: "r14".to_string(),
        }
    }

    // ========================================================================
    // String Management
    // ========================================================================

    pub fn add_string(&mut self, s: &str) -> (String, usize) {
        let label = format!("str_{}", self.str_count);
        self.str_count += 1;
        let len = s.len();

        let mut escaped = String::new();
        for byte in s.bytes() {
            if byte >= 32 && byte <= 126 && byte != b'"' && byte != b'\\' {
                escaped.push(byte as char);
            } else {
                escaped.push_str(&format!("\", {}, \"", byte));
            }
        }

        self.data.push(format!("    {}: db \"{}\", 0", label, escaped));
        (label, len)
    }

    // ========================================================================
    // Emission helpers
    // ========================================================================

    pub fn emit_func_line(&mut self, line: &str) {
        self.funcs.push(line.to_string());
    }

    pub fn emit_func_instr(&mut self, instr: &str, args: &str) {
        self.funcs.push(format!("    {:8} {}", instr, args));
    }

    pub fn emit_text_line(&mut self, line: &str) {
        self.text.push(line.to_string());
    }

    pub fn emit_text_instr(&mut self, instr: &str, args: &str) {
        self.text.push(format!("    {:8} {}", instr, args));
    }

    fn emit_instr(&mut self, in_proc: bool, instr: &str, args: &str) {
        if in_proc {
            self.emit_func_instr(instr, args);
        } else {
            self.emit_text_instr(instr, args);
        }
    }

    // ========================================================================
    // Stack Operations (Stack-Based Language)
    // ========================================================================

    pub fn emit_push_i64(&mut self, value: i64, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "sub", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("qword [{}], {}", pile, value));
    }

    pub fn emit_push_str(&mut self, s: &str, in_proc: bool) {
        let (label, _) = self.add_string(s);
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "sub", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("qword [{}], {}", pile, label));
    }

    pub fn emit_add_i64(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "add", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_sub_i64(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "sub", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_mul_i64(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "imul", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_div_i64(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "cqo", "");
        self.emit_instr(in_proc, "idiv", &format!("qword [{}]", pile));
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_mod_i64(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "cqo", "");
        self.emit_instr(in_proc, "idiv", &format!("qword [{}]", pile));
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rdx", pile)); // rdx = remainder
    }

    pub fn emit_dup(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "sub", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_swap(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "mov", &format!("rbx, [{} + 8]", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rbx", pile));
        self.emit_instr(in_proc, "mov", &format!("[{} + 8], rax", pile));
    }

    pub fn emit_drop(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
    }

    pub fn emit_over(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "sub", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_rot(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 16]", pile)); // c
        self.emit_instr(in_proc, "mov", &format!("rbx, [{} + 8]", pile));  // b
        self.emit_instr(in_proc, "mov", &format!("rcx, [{}]", pile));      // a
        self.emit_instr(in_proc, "mov", &format!("[{} + 16], rbx", pile)); // b -> c pos
        self.emit_instr(in_proc, "mov", &format!("[{} + 8], rcx", pile));  // a -> b pos
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));      // c -> a pos
    }

    // ========================================================================
    // I/O Operations
    // ========================================================================

    pub fn emit_dump(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rdi, [{}]", pile));
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "call", "dump_i");
    }

    pub fn emit_puts(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rdi, [{}]", pile));
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "call", "dump_str");
    }

    // ========================================================================
    // Function / Procedure Calls
    // ========================================================================

    pub fn emit_call(&mut self, name: &str, in_proc: bool) {
        self.emit_instr(in_proc, "call", &format!("proc_{}", name));
    }

    pub fn emit_ret(&mut self, in_proc: bool, is_main: bool, _returns_value: bool) {
        let pile = &self.stack_reg.clone();
        if in_proc && !is_main {
            // Always try to return the top of stack value
            // TODO: When type system is complete, use returns_value to differentiate
            self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
            self.emit_instr(in_proc, "pop", &pile.clone());
            self.emit_instr(in_proc, "sub", &format!("{}, 8", pile));
            self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
        }
        self.emit_instr(in_proc, "ret", "");
    }

    pub fn emit_proc_start(&mut self, name: &str, is_main: bool) {
        self.emit_func_line(&format!("proc_{}:", name));
        if !is_main {
            self.emit_func_instr("push", &self.stack_reg.clone());
        }
    }

    pub fn emit_proc_end(&mut self) {
        // Nothing special needed, ret handles cleanup
    }

    // ========================================================================
    // Syscalls
    // ========================================================================

    pub fn emit_syscall(&mut self, arg_count: usize, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        
        match arg_count {
            0 => {
                self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
                self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
                self.emit_instr(in_proc, "syscall", "");
                self.emit_instr(in_proc, "sub", &format!("{}, 8", pile));
                self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
            }
            1 => {
                self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
                self.emit_instr(in_proc, "mov", &format!("rdi, [{}]", pile));
                self.emit_instr(in_proc, "add", &format!("{}, 16", pile));
                self.emit_instr(in_proc, "syscall", "");
                self.emit_instr(in_proc, "sub", &format!("{}, 8", pile));
                self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
            }
            2 => {
                self.emit_instr(in_proc, "mov", &format!("rax, [{} + 16]", pile));
                self.emit_instr(in_proc, "mov", &format!("rdi, [{} + 8]", pile));
                self.emit_instr(in_proc, "mov", &format!("rsi, [{}]", pile));
                self.emit_instr(in_proc, "add", &format!("{}, 24", pile));
                self.emit_instr(in_proc, "syscall", "");
                self.emit_instr(in_proc, "sub", &format!("{}, 8", pile));
                self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
            }
            3 => {
                self.emit_instr(in_proc, "mov", &format!("rax, [{} + 24]", pile));
                self.emit_instr(in_proc, "mov", &format!("rdi, [{} + 16]", pile));
                self.emit_instr(in_proc, "mov", &format!("rsi, [{} + 8]", pile));
                self.emit_instr(in_proc, "mov", &format!("rdx, [{}]", pile));
                self.emit_instr(in_proc, "add", &format!("{}, 32", pile));
                self.emit_instr(in_proc, "syscall", "");
                self.emit_instr(in_proc, "sub", &format!("{}, 8", pile));
                self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
            }
            4 => {
                self.emit_instr(in_proc, "mov", &format!("rax, [{} + 32]", pile));
                self.emit_instr(in_proc, "mov", &format!("rdi, [{} + 24]", pile));
                self.emit_instr(in_proc, "mov", &format!("rsi, [{} + 16]", pile));
                self.emit_instr(in_proc, "mov", &format!("rdx, [{} + 8]", pile));
                self.emit_instr(in_proc, "mov", &format!("r10, [{}]", pile));
                self.emit_instr(in_proc, "add", &format!("{}, 40", pile));
                self.emit_instr(in_proc, "syscall", "");
                self.emit_instr(in_proc, "sub", &format!("{}, 8", pile));
                self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
            }
            5 => {
                self.emit_instr(in_proc, "mov", &format!("rax, [{} + 40]", pile));
                self.emit_instr(in_proc, "mov", &format!("rdi, [{} + 32]", pile));
                self.emit_instr(in_proc, "mov", &format!("rsi, [{} + 24]", pile));
                self.emit_instr(in_proc, "mov", &format!("rdx, [{} + 16]", pile));
                self.emit_instr(in_proc, "mov", &format!("r10, [{} + 8]", pile));
                self.emit_instr(in_proc, "mov", &format!("r8, [{}]", pile));
                self.emit_instr(in_proc, "add", &format!("{}, 48", pile));
                self.emit_instr(in_proc, "syscall", "");
                self.emit_instr(in_proc, "sub", &format!("{}, 8", pile));
                self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
            }
            6 => {
                self.emit_instr(in_proc, "mov", &format!("rax, [{} + 48]", pile));
                self.emit_instr(in_proc, "mov", &format!("rdi, [{} + 40]", pile));
                self.emit_instr(in_proc, "mov", &format!("rsi, [{} + 32]", pile));
                self.emit_instr(in_proc, "mov", &format!("rdx, [{} + 24]", pile));
                self.emit_instr(in_proc, "mov", &format!("r10, [{} + 16]", pile));
                self.emit_instr(in_proc, "mov", &format!("r8, [{} + 8]", pile));
                self.emit_instr(in_proc, "mov", &format!("r9, [{}]", pile));
                self.emit_instr(in_proc, "add", &format!("{}, 56", pile));
                self.emit_instr(in_proc, "syscall", "");
                self.emit_instr(in_proc, "sub", &format!("{}, 8", pile));
                self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
            }
            _ => {}
        }
    }

    // ========================================================================
    // Comparison Operations
    // ========================================================================

    pub fn emit_eq(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "cmp", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "sete", "al");
        self.emit_instr(in_proc, "movzx", "rax, al");
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_neq(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "cmp", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "setne", "al");
        self.emit_instr(in_proc, "movzx", "rax, al");
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_lt(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "cmp", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "setl", "al");
        self.emit_instr(in_proc, "movzx", "rax, al");
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_gt(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "cmp", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "setg", "al");
        self.emit_instr(in_proc, "movzx", "rax, al");
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_le(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "cmp", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "setle", "al");
        self.emit_instr(in_proc, "movzx", "rax, al");
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_ge(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "cmp", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "setge", "al");
        self.emit_instr(in_proc, "movzx", "rax, al");
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    // ========================================================================
    // Bitwise Operations
    // ========================================================================

    pub fn emit_and(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "and", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_or(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "or", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_xor(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "xor", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_not(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "not", &format!("qword [{}]", pile));
    }

    pub fn emit_shl(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rcx, [{}]", pile));
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "shl", "rax, cl");
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_shr(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rcx, [{}]", pile));
        self.emit_instr(in_proc, "mov", &format!("rax, [{} + 8]", pile));
        self.emit_instr(in_proc, "shr", "rax, cl");
        self.emit_instr(in_proc, "add", &format!("{}, 8", pile));
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    // ========================================================================
    // Memory Operations
    // ========================================================================

    pub fn emit_load8(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "movzx", "rax, byte [rax]");
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_load16(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "movzx", "rax, word [rax]");
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_load32(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "mov", "eax, dword [rax]");
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_load64(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "mov", "rax, qword [rax]");
        self.emit_instr(in_proc, "mov", &format!("[{}], rax", pile));
    }

    pub fn emit_store8(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));      // value
        self.emit_instr(in_proc, "mov", &format!("rbx, [{} + 8]", pile));  // addr
        self.emit_instr(in_proc, "mov", "[rbx], al");
        self.emit_instr(in_proc, "add", &format!("{}, 16", pile));
    }

    pub fn emit_store16(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "mov", &format!("rbx, [{} + 8]", pile));
        self.emit_instr(in_proc, "mov", "[rbx], ax");
        self.emit_instr(in_proc, "add", &format!("{}, 16", pile));
    }

    pub fn emit_store32(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "mov", &format!("rbx, [{} + 8]", pile));
        self.emit_instr(in_proc, "mov", "dword [rbx], eax");
        self.emit_instr(in_proc, "add", &format!("{}, 16", pile));
    }

    pub fn emit_store64(&mut self, in_proc: bool) {
        let pile = &self.stack_reg.clone();
        self.emit_instr(in_proc, "mov", &format!("rax, [{}]", pile));
        self.emit_instr(in_proc, "mov", &format!("rbx, [{} + 8]", pile));
        self.emit_instr(in_proc, "mov", "qword [rbx], rax");
        self.emit_instr(in_proc, "add", &format!("{}, 16", pile));
    }

    // ========================================================================
    // Helper Functions (added to output)
    // ========================================================================

    pub fn add_dump_helper(&mut self) {
        self.funcs.push(r#"dump_i:
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
    ret"#.to_string());

        self.funcs.push(r#"dump_str:
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
    ret"#.to_string());
    }

    // ========================================================================
    // Exit
    // ========================================================================

    pub fn emit_exit(&mut self) {
        self.emit_text_instr("mov", "rax, 60");
        self.emit_text_instr("xor", "rdi, rdi");
        self.emit_text_instr("syscall", "");
    }

    // ========================================================================
    // Build Final Assembly
    // ========================================================================

    pub fn build(&self) -> String {
        let mut output = Vec::new();

        // BSS Section
        output.push("section .bss".to_string());
        output.extend(self.bss.clone());

        // Data Section
        output.push("\nsection .data".to_string());
        output.extend(self.data.clone());

        // Text Section
        output.push("\nsection .text".to_string());

        // Functions
        if !self.funcs.is_empty() {
            output.push(String::new());
            output.extend(self.funcs.clone());
        }

        // Entry point
        output.push("\nglobal _start".to_string());
        output.push("_start:".to_string());
        output.push(format!("    lea      r15, [data_stack + 4096*8]"));
        output.push("    call     proc_main".to_string());

        // Main code
        output.extend(self.text.clone());

        output.join("\n")
    }

    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(self.build().as_bytes())?;
        Ok(())
    }
}

// ============================================================================
// Compiler from IR to Assembly
// ============================================================================

use crate::{EIrInstr, EType};

pub struct StackCompiler;

impl StackCompiler {
    pub fn compile(ir: &[EIrInstr]) -> Result<String, String> {
        let mut asm = StackAsmBuilder::new();
        let mut procs: HashMap<&'static str, &Vec<EIrInstr>> = HashMap::new();
        let mut has_main = false;

        // Add helper functions
        asm.add_dump_helper();

        // Collect all procedures
        for instr in ir {
            match instr {
                EIrInstr::Proc(name, body, _, _) => {
                    if *name == "main" {
                        has_main = true;
                    }
                    procs.insert(name, body);
                }
                EIrInstr::Const(name, body) => {
                    procs.insert(name, body);
                }
                _ => {}
            }
        }

        if !has_main {
            return Err("Erreur: proc main non declaree".to_string());
        }

        // Collect return types
        let mut ret_types: HashMap<&'static str, EType> = HashMap::new();
        for instr in ir {
            if let EIrInstr::Proc(name, _, _, ret_type) = instr {
                ret_types.insert(name, ret_type.clone());
            }
        }

        // Compile all procedures
        for (name, body) in &procs {
            let is_main = *name == "main";
            let returns_value = ret_types.get(name).map_or(false, |t| *t != EType::Void);
            asm.emit_proc_start(name, is_main);
            Self::compile_instrs(&mut asm, body, true, is_main, returns_value);
        }

        // Emit exit
        asm.emit_exit();

        Ok(asm.build())
    }

    fn compile_instrs(asm: &mut StackAsmBuilder, instrs: &[EIrInstr], in_proc: bool, is_main: bool, returns_value: bool) {
        for instr in instrs {
            match instr {
                EIrInstr::PushI64(val) => asm.emit_push_i64(*val, in_proc),
                EIrInstr::PushStr(s) => asm.emit_push_str(s, in_proc),
                EIrInstr::AddI64 => asm.emit_add_i64(in_proc),
                EIrInstr::SubI64 => asm.emit_sub_i64(in_proc),
                EIrInstr::MulI64 => asm.emit_mul_i64(in_proc),
                EIrInstr::DivI64 => asm.emit_div_i64(in_proc),
                EIrInstr::ModI64 => asm.emit_mod_i64(in_proc),
                EIrInstr::Dup => asm.emit_dup(in_proc),
                EIrInstr::Swap => asm.emit_swap(in_proc),
                EIrInstr::Dump => asm.emit_dump(in_proc),
                EIrInstr::Puts => asm.emit_puts(in_proc),
                EIrInstr::Call(name) => asm.emit_call(name, in_proc),
                EIrInstr::Ret => asm.emit_ret(in_proc, is_main, returns_value),
                EIrInstr::Syscall0 => asm.emit_syscall(0, in_proc),
                EIrInstr::Syscall1 => asm.emit_syscall(1, in_proc),
                EIrInstr::Syscall2 => asm.emit_syscall(2, in_proc),
                EIrInstr::Syscall3 => asm.emit_syscall(3, in_proc),
                EIrInstr::Syscall4 => asm.emit_syscall(4, in_proc),
                EIrInstr::Syscall5 => asm.emit_syscall(5, in_proc),
                EIrInstr::Syscall6 => asm.emit_syscall(6, in_proc),
                EIrInstr::Proc(_, _, _, _) | EIrInstr::Const(_, _) => {
                    // Skip nested proc/const definitions
                }
                _ => {
                    // Handle other instructions as needed
                }
            }
        }
    }
}
