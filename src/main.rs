#![allow(nonstandard_style)]
mod syntax;
use syntax::F_lParseProgram;

use std::{collections::HashMap, fmt::Write};

#[derive(Clone, Debug)]
enum EIrInstr {
    // Data manipulation
    PushI64(i64),               // number
    PushStr(&'static str),      // "..."
    PushStrRef(&'static str),   // "..."

    // Arithmetic
    AddI64,                     // +
    SubI64,                     // not implemented
    MulI64,                     // not implemented
    DivI64,                     // not implemented
    ModI64,                     // not implemented   

    // Stack manipulation
    Dup,                        // dup
    Swap,                       // swap
    DumpStr,                    // dump_str
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
    Label(String),              // not implemented
    Jump(String),               // not implemented
    JumpIfZero(String),         // not implemented
    Call(&'static str),         // [name_proc/const]
    Ret,                        // end (for proc)


    Proc(&'static str, Vec<EIrInstr>),  // proc [name] in ... end
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
        let mut l_cBuilder = CAsmBuilder { 
            //l_sData: String::from("section .data\n    dump_buf: resb 21\n"),
            l_sData: String::from("section .data\n    dump_buf: resb 21\n    data_stack: resq 4096\n"),
            l_sCode: String::new(),
            l_iFuncCode: String::new(),
            l_iStrCount: 0,
        };
        l_cBuilder
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

    fn F_vEmitExit(&mut self) {
        self.F_vEmitInstr("mov", "rax, 60");
        self.F_vEmitInstr("xor", "rdi, rdi");
        self.F_vEmitInstr("syscall", "");
    }

    fn F_sBuild(self) -> String {
        format!("{}\nsection .text\n{}\nglobal _start\n_start:\n    lea      r15, [data_stack + 4096*8]\n    call     proc_main\n{}", self.l_sData, self.l_iFuncCode, self.l_sCode)
    }

    fn F_vEmitDumpFunction(&mut self) {
        self.F_vEmitFuncLine("dump_value:");
        self.F_vEmitFuncInstr("cmp", "rax, 0x1000");
        self.F_vEmitFuncInstr("jb", ".print_int");
        
        self.F_vEmitFuncLine(".print_str:");
        self.F_vEmitFuncInstr("mov", "rsi, rax");
        self.F_vEmitFuncInstr("xor", "rdx, rdx");
        self.F_vEmitFuncLine(".strlen_loop:");
        self.F_vEmitFuncInstr("cmp", "byte [rsi + rdx], 0");
        self.F_vEmitFuncInstr("je", ".strlen_done");
        self.F_vEmitFuncInstr("inc", "rdx");
        self.F_vEmitFuncInstr("jmp", ".strlen_loop");
        self.F_vEmitFuncLine(".strlen_done:");
        self.F_vEmitFuncInstr("mov", "rax, 1");
        self.F_vEmitFuncInstr("mov", "rdi, 1");
        self.F_vEmitFuncInstr("syscall", "");
        self.F_vEmitFuncInstr("ret", "");
        
        self.F_vEmitFuncLine(".print_int:");
        self.F_vEmitFuncInstr("mov", "rcx, 10");
        self.F_vEmitFuncInstr("mov", "rdi, dump_buf");
        self.F_vEmitFuncInstr("add", "rdi, 20");
        self.F_vEmitFuncInstr("mov", "byte [rdi], 10");
        self.F_vEmitFuncInstr("dec", "rdi");
        self.F_vEmitFuncInstr("xor", "r8, r8");
        self.F_vEmitFuncInstr("test", "rax, rax");
        self.F_vEmitFuncInstr("jns", ".positive");
        self.F_vEmitFuncInstr("neg", "rax");
        self.F_vEmitFuncInstr("mov", "r8, 1");
        self.F_vEmitFuncLine(".positive:");
        self.F_vEmitFuncLine(".convert_loop:");
        self.F_vEmitFuncInstr("xor", "rdx, rdx");
        self.F_vEmitFuncInstr("div", "rcx");
        self.F_vEmitFuncInstr("add", "dl, '0'");
        self.F_vEmitFuncInstr("mov", "[rdi], dl");
        self.F_vEmitFuncInstr("dec", "rdi");
        self.F_vEmitFuncInstr("test", "rax, rax");
        self.F_vEmitFuncInstr("jnz", ".convert_loop");
        self.F_vEmitFuncInstr("test", "r8, r8");
        self.F_vEmitFuncInstr("jz", ".write");
        self.F_vEmitFuncInstr("mov", "byte [rdi], '-'");
        self.F_vEmitFuncInstr("dec", "rdi");
        self.F_vEmitFuncLine(".write:");
        self.F_vEmitFuncInstr("inc", "rdi");
        self.F_vEmitFuncInstr("mov", "rdx, 21");
        self.F_vEmitFuncInstr("mov", "rsi, dump_buf");
        self.F_vEmitFuncInstr("sub", "rdx, rdi");
        self.F_vEmitFuncInstr("add", "rdx, rsi");
        self.F_vEmitFuncInstr("mov", "rsi, rdi");
        self.F_vEmitFuncInstr("mov", "rax, 1");
        self.F_vEmitFuncInstr("mov", "rdi, 1");
        self.F_vEmitFuncInstr("syscall", "");
        self.F_vEmitFuncInstr("ret", "");
    }

    fn F_vEmitSyscall0(&mut self) {
        self.F_vEmitInstr("pop", "rax");
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("push", "rax");
    }

    fn F_vEmitSyscall1(&mut self) {
        self.F_vEmitInstr("pop", "rax");
        self.F_vEmitInstr("pop", "rdi");
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("push", "rax");
    }

    fn F_vEmitSyscall2(&mut self) {
        self.F_vEmitInstr("pop", "rax");
        self.F_vEmitInstr("pop", "rdi");
        self.F_vEmitInstr("pop", "rsi");
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("push", "rax");
    }

    fn F_vEmitSyscall3(&mut self) {
        self.F_vEmitInstr("pop", "rdx");
        self.F_vEmitInstr("pop", "rsi");
        self.F_vEmitInstr("pop", "rdi");
        self.F_vEmitInstr("pop", "rax");
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("push", "rax");
    }

    fn F_vEmitSyscall4(&mut self) {
        self.F_vEmitInstr("pop", "rax");
        self.F_vEmitInstr("pop", "rdi");
        self.F_vEmitInstr("pop", "rsi");
        self.F_vEmitInstr("pop", "rdx");
        self.F_vEmitInstr("pop", "r10");
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("push", "rax");
    }

    fn F_vEmitSyscall5(&mut self) {
        self.F_vEmitInstr("pop", "rax");
        self.F_vEmitInstr("pop", "rdi");
        self.F_vEmitInstr("pop", "rsi");
        self.F_vEmitInstr("pop", "rdx");
        self.F_vEmitInstr("pop", "r10");
        self.F_vEmitInstr("pop", "r8");
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("push", "rax");
    }

    fn F_vEmitSyscall6(&mut self) {
        self.F_vEmitInstr("pop", "r9");
        self.F_vEmitInstr("pop", "r8");
        self.F_vEmitInstr("pop", "r10");
        self.F_vEmitInstr("pop", "rdx");
        self.F_vEmitInstr("pop", "rsi");
        self.F_vEmitInstr("pop", "rdi");
        self.F_vEmitInstr("pop", "rax");
        self.F_vEmitInstr("syscall", "");
        self.F_vEmitInstr("push", "rax");
    }

}

struct CStackToX86_64;

impl CStackToX86_64 {
    fn F_sCompile(l_lIr: &[EIrInstr]) -> Result<String, String> {
        let mut l_cAsm = CAsmBuilder::F_cNew();
        let mut l_hmProcs: HashMap<&'static str, &Vec<EIrInstr>> = HashMap::new();
        let mut l_bHasMain = false;

        l_cAsm.F_vEmitDumpFunction();

        for l_cInstr in l_lIr {
            if let EIrInstr::Proc(l_sName, l_lBody) = l_cInstr {
                if *l_sName == "main" {
                    l_bHasMain = true;
                }
                l_hmProcs.insert(l_sName, l_lBody);
            } else if let EIrInstr::Const(l_sName, l_lBody) = l_cInstr {
                l_hmProcs.insert(l_sName, l_lBody);
            }
        }

        if !l_bHasMain {
            return Err("Erreur: proc main non declaree".to_string());
        }

        for (l_sName, l_lBody) in &l_hmProcs {
            l_cAsm.F_vEmitFuncLine(&format!("proc_{}:", l_sName));
            Self::F_vCompileInstrs(&mut l_cAsm, l_lBody, true);
        }

        l_cAsm.F_vEmitExit();
        Ok(l_cAsm.F_sBuild())
    }

    fn F_vCompileInstrs(l_cAsm: &mut CAsmBuilder, l_lInstrs: &[EIrInstr], l_bInProc: bool) {
        for l_cInstr in l_lInstrs {
            match *l_cInstr {
                EIrInstr::PushI64(l_iVal) => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("sub", "r15, 8");
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("qword [r15], {}", l_iVal));
                    } else {
                        l_cAsm.F_vEmitInstr("sub", "r15, 8");
                        l_cAsm.F_vEmitInstr("mov", &format!("qword [r15], {}", l_iVal));
                    }
                }
                EIrInstr::PushStr(l_sStr) => {
                    let (l_sLabel, _l_iLen) = l_cAsm.F_sAddString(l_sStr);
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("sub", "r15, 8");
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("qword [r15], {}", l_sLabel));
                    } else {
                        l_cAsm.F_vEmitInstr("sub", "r15, 8");
                        l_cAsm.F_vEmitInstr("mov", &format!("qword [r15], {}", l_sLabel));
                    }
                }
                EIrInstr::PushStrRef(l_sStr) => {
                    let (l_sLabel, l_iLen) = l_cAsm.F_sAddString(l_sStr);
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("sub", "r15, 16");
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("qword [r15 + 8], {}", l_sLabel));
                        l_cAsm.F_vEmitFuncInstr("mov", &format!("qword [r15], {}", l_iLen));
                    } else {
                        l_cAsm.F_vEmitInstr("sub", "r15, 16");
                        l_cAsm.F_vEmitInstr("mov", &format!("qword [r15 + 8], {}", l_sLabel));
                        l_cAsm.F_vEmitInstr("mov", &format!("qword [r15], {}", l_iLen));
                    }
                }
                EIrInstr::AddI64 => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("mov", "rax, [r15]");
                        l_cAsm.F_vEmitFuncInstr("add", "rax, [r15 + 8]");
                        l_cAsm.F_vEmitFuncInstr("add", "r15, 8");
                        l_cAsm.F_vEmitFuncInstr("mov", "[r15], rax");
                    } else {
                        l_cAsm.F_vEmitInstr("mov", "rax, [r15]");
                        l_cAsm.F_vEmitInstr("add", "rax, [r15 + 8]");
                        l_cAsm.F_vEmitInstr("add", "r15, 8");
                        l_cAsm.F_vEmitInstr("mov", "[r15], rax");
                    }
                }
                EIrInstr::SubI64 => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("mov", "rax, [r15 + 8]");
                        l_cAsm.F_vEmitFuncInstr("sub", "rax, [r15]");
                        l_cAsm.F_vEmitFuncInstr("add", "r15, 8");
                        l_cAsm.F_vEmitFuncInstr("mov", "[r15], rax");
                    } else {
                        l_cAsm.F_vEmitInstr("mov", "rax, [r15 + 8]");
                        l_cAsm.F_vEmitInstr("sub", "rax, [r15]");
                        l_cAsm.F_vEmitInstr("add", "r15, 8");
                        l_cAsm.F_vEmitInstr("mov", "[r15], rax");
                    }
                }
                EIrInstr::Swap => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("mov", "rax, [r15]");
                        l_cAsm.F_vEmitFuncInstr("mov", "rbx, [r15 + 8]");
                        l_cAsm.F_vEmitFuncInstr("mov", "[r15], rbx");
                        l_cAsm.F_vEmitFuncInstr("mov", "[r15 + 8], rax");
                    } else {
                        l_cAsm.F_vEmitInstr("mov", "rax, [r15]");
                        l_cAsm.F_vEmitInstr("mov", "rbx, [r15 + 8]");
                        l_cAsm.F_vEmitInstr("mov", "[r15], rbx");
                        l_cAsm.F_vEmitInstr("mov", "[r15 + 8], rax");
                    }
                }
                EIrInstr::Dup => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("mov", "rax, [r15]");
                        l_cAsm.F_vEmitFuncInstr("sub", "r15, 8");
                        l_cAsm.F_vEmitFuncInstr("mov", "[r15], rax");
                    } else {
                        l_cAsm.F_vEmitInstr("mov", "rax, [r15]");
                        l_cAsm.F_vEmitInstr("sub", "r15, 8");
                        l_cAsm.F_vEmitInstr("mov", "[r15], rax");
                    }
                }
                EIrInstr::DumpStr | EIrInstr::Dump => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("mov", "rax, [r15]");
                        l_cAsm.F_vEmitFuncInstr("add", "r15, 8");
                        l_cAsm.F_vEmitFuncInstr("call", "dump_value");
                    } else {
                        l_cAsm.F_vEmitInstr("mov", "rax, [r15]");
                        l_cAsm.F_vEmitInstr("add", "r15, 8");
                        l_cAsm.F_vEmitInstr("call", "dump_value");
                    }
                }
                EIrInstr::Call(l_sName) => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("call", &format!("proc_{}", l_sName));
                    } else {
                        l_cAsm.F_vEmitInstr("call", &format!("proc_{}", l_sName));
                    }
                }
                EIrInstr::Ret => {
                    if l_bInProc {
                        l_cAsm.F_vEmitFuncInstr("ret", "");
                    } else {
                        l_cAsm.F_vEmitInstr("ret", "");
                    }
                }
                EIrInstr::Syscall0 => l_cAsm.F_vEmitSyscall0(),
                EIrInstr::Syscall1 => l_cAsm.F_vEmitSyscall1(),
                EIrInstr::Syscall2 => l_cAsm.F_vEmitSyscall2(),
                EIrInstr::Syscall3 => l_cAsm.F_vEmitSyscall3(),
                EIrInstr::Syscall4 => l_cAsm.F_vEmitSyscall4(),
                EIrInstr::Syscall5 => l_cAsm.F_vEmitSyscall5(),
                EIrInstr::Syscall6 => l_cAsm.F_vEmitSyscall6(),
                EIrInstr::Proc(_, _) => {}
                _ => { panic!("Instruction non supportee"); }
            }
        }
    }
}

struct CStackToInterpreter;

impl CStackToInterpreter {
    fn F_vInterpret(l_lIr: &[EIrInstr]) -> Result<(), String> {
        let mut l_lDataStack: Vec<i64> = Vec::new();
        let mut l_lCallStack: Vec<usize> = Vec::new();
        let mut l_hmProcs: HashMap<&'static str, &Vec<EIrInstr>> = HashMap::new();
        let mut l_bHasMain = false;

        for l_cInstr in l_lIr {
            if let EIrInstr::Proc(l_sName, l_lBody) = l_cInstr {
                if *l_sName == "main" {
                    l_bHasMain = true;
                }
                l_hmProcs.insert(l_sName, l_lBody);
            } else if let EIrInstr::Const(l_sName, l_lBody) = l_cInstr {
                l_hmProcs.insert(l_sName, l_lBody);
            }
        }

        if !l_bHasMain {
            return Err("Erreur: proc main non declaree".to_string());
        }

        Self::F_vExecuteProc("main", &l_hmProcs, &mut l_lDataStack, &mut l_lCallStack)?;
        //println!("Stack finale: {:?}", l_lDataStack);
        Ok(())
    }

    fn F_vExecuteProc(
        l_sName: &str, 
        l_hmProcs: &HashMap<&'static str, &Vec<EIrInstr>>,
        l_lDataStack: &mut Vec<i64>,
        l_lCallStack: &mut Vec<usize>
    ) -> Result<(), String> {
        let l_lBody = l_hmProcs.get(l_sName)
            .ok_or_else(|| format!("Proc {} non trouvee", l_sName))?;

        for l_cInstr in l_lBody.iter() {
            match *l_cInstr {
                EIrInstr::PushI64(l_iVal) => {
                    l_lDataStack.push(l_iVal);
                }
                EIrInstr::PushStr(l_sStr) => {
                    let l_pBuf = l_sStr.as_ptr() as i64;
                    let l_iLen = l_sStr.len() as i64;
                    l_lDataStack.push(l_pBuf);
                    l_lDataStack.push(l_iLen);
                }
                EIrInstr::PushStrRef(l_sStr) => {
                    let l_pBuf = l_sStr.as_ptr() as i64;
                    let l_iLen = l_sStr.len() as i64;
                    l_lDataStack.push(l_pBuf);
                    l_lDataStack.push(l_iLen);
                }
                EIrInstr::AddI64 => {
                    let l_iB = l_lDataStack.pop().ok_or("Stack underflow")?;
                    let l_iA = l_lDataStack.pop().ok_or("Stack underflow")?;
                    l_lDataStack.push(l_iA + l_iB);
                }
                EIrInstr::SubI64 => {
                    let l_iB = l_lDataStack.pop().ok_or("Stack underflow")?;
                    let l_iA = l_lDataStack.pop().ok_or("Stack underflow")?;
                    l_lDataStack.push(l_iA - l_iB);
                }
                EIrInstr::Swap => {
                    let l_iB = l_lDataStack.pop().ok_or("Stack underflow")?;
                    let l_iA = l_lDataStack.pop().ok_or("Stack underflow")?;
                    l_lDataStack.push(l_iB);
                    l_lDataStack.push(l_iA);
                }
                EIrInstr::Dup => {
                    let l_iTop = *l_lDataStack.last().ok_or("Stack underflow")?;
                    l_lDataStack.push(l_iTop);
                }
                EIrInstr::DumpStr => {
                    let l_iLen = l_lDataStack.pop().ok_or("Stack underflow")? as usize;
                    let l_pBuf = l_lDataStack.pop().ok_or("Stack underflow")? as *const u8;
                    let l_sReconstructed = unsafe {
                        std::str::from_utf8_unchecked(std::slice::from_raw_parts(l_pBuf, l_iLen))
                    };
                    print!("{}", l_sReconstructed);
                }
                EIrInstr::Dump => {
                    let l_iVal = l_lDataStack.pop().ok_or("Stack underflow")?;
                    print!("{}\n", l_iVal);
                }
                EIrInstr::Call(l_sTarget) => {
                    Self::F_vExecuteProc(l_sTarget, l_hmProcs, l_lDataStack, l_lCallStack)?;
                }
                EIrInstr::Ret => {
                    return Ok(());
                }

                EIrInstr::Syscall0 => {
                    let l_iSysno = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno) };
                    l_lDataStack.push(l_iRet as i64);
                }
                EIrInstr::Syscall1 => {
                    let l_iArg1 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iSysno = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno, l_iArg1) };
                    l_lDataStack.push(l_iRet as i64);
                }
                EIrInstr::Syscall2 => {
                    let l_iArg2 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg1 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iSysno = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno, l_iArg1, l_iArg2) };
                    l_lDataStack.push(l_iRet as i64);
                }
                EIrInstr::Syscall3 => {
                    let l_iArg3 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg2 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg1 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iSysno = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno, l_iArg1, l_iArg2, l_iArg3) };
                    l_lDataStack.push(l_iRet as i64);
                }
                EIrInstr::Syscall4 => {
                    let l_iArg4 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg3 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg2 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg1 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iSysno = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno, l_iArg1, l_iArg2, l_iArg3, l_iArg4) };
                    l_lDataStack.push(l_iRet as i64);
                }
                EIrInstr::Syscall5 => {
                    let l_iArg5 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg4 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg3 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg2 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg1 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iSysno = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iRet = unsafe { libc::syscall(l_iSysno, l_iArg1, l_iArg2, l_iArg3, l_iArg4, l_iArg5) };
                    l_lDataStack.push(l_iRet as i64);
                }
                EIrInstr::Syscall6 => {
                    let l_iArg6 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg5 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg4 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg3 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg2 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iArg1 = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
                    let l_iSysno = l_lDataStack.pop().expect("Stack underflow") as libc::c_long;
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

    /*let l_lProgram = vec![
        EIrInstr::Proc("N", vec![
            EIrInstr::PushI64(69),
            EIrInstr::Ret,
        ]),
        EIrInstr::Proc("M", vec![
            EIrInstr::PushI64(420),
            EIrInstr::Ret,
        ]),
        EIrInstr::Proc("K", vec![
            EIrInstr::Call("N"),
            EIrInstr::Call("M"),
            EIrInstr::AddI64,
            EIrInstr::Ret,
        ]),
        EIrInstr::Proc("main", vec![
            EIrInstr::Call("M"),
            EIrInstr::Dump,
            EIrInstr::PushStr("\nHello\nd"),
            EIrInstr::DumpStr,
            EIrInstr::Ret,
        ]),
    ];

    println!("=== INTERPRETATION ===");
    match CStackToInterpreter::F_vInterpret(&l_lProgram) {
        Ok(_) => {},
        Err(l_sErr) => eprintln!("{}", l_sErr),
    }

    println!("\n=== COMPILATION X86_64 ===");
    match CStackToX86_64::F_sCompile(&l_lProgram) {
        Ok(l_sAsm) => {
            std::fs::write("out.asm", l_sAsm).expect("Erreur ecriture fichier");
            //println!("Code genere dans out.asm");
        },
        Err(l_sErr) => eprintln!("{}", l_sErr),
    }*/

    match F_lParseProgram(l_sCode.as_str()) {
        Ok(l_lProgram) => {
            println!("Programme parsé!");

            // print toutes les instruction
            for l_cInstr in &l_lProgram {
                // si l'instruction est une proc/const, afficher son nom et son corps
                match l_cInstr {
                    EIrInstr::Proc(l_sName, l_lBody) => {
                        println!("Proc {}:", l_sName);
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