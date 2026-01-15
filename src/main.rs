#![allow(nonstandard_style)]
mod syntax;
pub mod engine;
use syntax::F_lParseProgram;
use engine::StackCompiler;

use std::collections::HashMap;

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
    SubI64,                     // -
    MulI64,                     // *
    DivI64,                     // /
    ModI64,                     // %

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
                EIrInstr::MulI64 => {
                    let l_iB = l_lDataStack.pop().ok_or("Stack underflow mul")?;
                    let l_iA = l_lDataStack.pop().ok_or("Stack underflow mul")?;
                    l_lDataStack.push(l_iA * l_iB);
                }
                EIrInstr::DivI64 => {
                    let l_iB = l_lDataStack.pop().ok_or("Stack underflow div")?;
                    let l_iA = l_lDataStack.pop().ok_or("Stack underflow div")?;
                    l_lDataStack.push(l_iA / l_iB);
                }
                EIrInstr::ModI64 => {
                    let l_iB = l_lDataStack.pop().ok_or("Stack underflow mod")?;
                    let l_iA = l_lDataStack.pop().ok_or("Stack underflow mod")?;
                    l_lDataStack.push(l_iA % l_iB);
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
                        if l_lDataStack.len() > l_iStackBase {
                            // Procedure returns a value
                            let l_iResult = l_lDataStack.pop().ok_or("Stack underflow ret")?;
                            l_lDataStack.truncate(l_iStackBase);
                            l_lDataStack.push(l_iResult);
                        } else {
                            // Procedure returns Void (stack is at base level)
                            l_lDataStack.truncate(l_iStackBase);
                        }
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
            match StackCompiler::compile(&l_lProgram) {
                Ok(l_sAsm) => {
                    std::fs::write("out.asm", l_sAsm).unwrap();
                },
                Err(e) => eprintln!("Erreur: {}", e),
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
