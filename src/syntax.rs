#![allow(nonstandard_style)]
use crate::EIrInstr;

#[derive(Debug, Clone, PartialEq)]
enum EToken {
    Number(i64),
    String(String),
    Ident(String),
    Const,
    Proc,
    In,
    End,
    Plus,
    Dup,
    Swap,
    Puts,
    Dump,
    Syscall0,
    Syscall1,
    Syscall2,
    Syscall3,
    Syscall4,
    Syscall5,
    Syscall6,

    // Types
    I64,
    F64,
    Str,
    Bool,
}

struct CLexer {
    l_sInput: String,
    l_iPos: usize,
}

impl CLexer {
    fn F_cNew(l_sInput: String) -> Self {
        CLexer { l_sInput, l_iPos: 0 }
    }

    fn F_cPeek(&self) -> Option<char> {
        self.l_sInput.chars().nth(self.l_iPos)
    }

    fn F_cAdvance(&mut self) -> Option<char> {
        let l_cChar = self.F_cPeek();
        self.l_iPos += 1;
        l_cChar
    }

    fn F_vSkipWhitespace(&mut self) {
        while let Some(l_cChar) = self.F_cPeek() {
            if l_cChar.is_whitespace() {
                self.F_cAdvance();
            } else if l_cChar == '#' {
                while let Some(l_cChar) = self.F_cPeek() {
                    self.F_cAdvance();
                    if l_cChar == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn F_sReadString(&mut self) -> Result<String, String> {
        self.F_cAdvance();
        let mut l_sResult = String::new();
        
        while let Some(l_cChar) = self.F_cPeek() {
            if l_cChar == '"' {
                self.F_cAdvance();
                return Ok(l_sResult);
            } else if l_cChar == '\\' {
                self.F_cAdvance();
                match self.F_cAdvance() {
                    Some('n') => l_sResult.push('\n'),
                    Some('t') => l_sResult.push('\t'),
                    Some('r') => l_sResult.push('\r'),
                    Some('\\') => l_sResult.push('\\'),
                    Some('"') => l_sResult.push('"'),
                    Some(l_cC) => {
                        l_sResult.push('\\');
                        l_sResult.push(l_cC);
                    }
                    None => return Err("Fin inattendue dans escape".to_string()),
                }
            } else {
                l_sResult.push(l_cChar);
                self.F_cAdvance();
            }
        }
        
        Err("String non terminee".to_string())
    }

    fn F_sReadIdent(&mut self) -> String {
        let mut l_sResult = String::new();
        
        while let Some(l_cChar) = self.F_cPeek() {
            if l_cChar.is_alphanumeric() || l_cChar == '_' {
                l_sResult.push(l_cChar);
                self.F_cAdvance();
            } else {
                break;
            }
        }
        
        l_sResult
    }

    fn F_iReadNumber(&mut self) -> Result<i64, String> {
        let mut l_sNum = String::new();
        let l_bNeg = if self.F_cPeek() == Some('-') {
            self.F_cAdvance();
            true
        } else {
            false
        };

        while let Some(l_cChar) = self.F_cPeek() {
            if l_cChar.is_numeric() {
                l_sNum.push(l_cChar);
                self.F_cAdvance();
            } else {
                break;
            }
        }

        if l_sNum.is_empty() {
            return Err("Nombre invalide".to_string());
        }

        let l_iVal = l_sNum.parse::<i64>().map_err(|_| "Parse error".to_string())?;
        Ok(if l_bNeg { -l_iVal } else { l_iVal })
    }

    fn F_lTokenize(&mut self) -> Result<Vec<EToken>, String> {
        let mut l_lTokens = Vec::new();

        loop {
            self.F_vSkipWhitespace();

            match self.F_cPeek() {
                None => break,
                Some('"') => {
                    let l_sStr = self.F_sReadString()?;
                    l_lTokens.push(EToken::String(l_sStr));
                }
                Some('+') => {
                    self.F_cAdvance();
                    l_lTokens.push(EToken::Plus);
                }
                Some(l_cChar) if l_cChar.is_numeric() || (l_cChar == '-' && self.l_sInput.chars().nth(self.l_iPos + 1).map_or(false, |c| c.is_numeric())) => {
                    let l_iNum = self.F_iReadNumber()?;
                    l_lTokens.push(EToken::Number(l_iNum));
                }
                Some(l_cChar) if l_cChar.is_alphabetic() || l_cChar == '_' => {
                    let l_sIdent = self.F_sReadIdent();
                    let l_eToken = match l_sIdent.as_str() {
                        "proc" => EToken::Proc,
                        "const" => EToken::Const,
                        "in" => EToken::In,
                        "end" => EToken::End,
                        "dup" => EToken::Dup,
                        "swap" => EToken::Swap,
                        "puts" => EToken::Puts,
                        "dump" => EToken::Dump,
                        "syscall" => EToken::Syscall0,
                        "syscall1" => EToken::Syscall1,
                        "syscall2" => EToken::Syscall2,
                        "syscall3" => EToken::Syscall3,
                        "syscall4" => EToken::Syscall4,
                        "syscall5" => EToken::Syscall5,
                        "syscall6" => EToken::Syscall6,
                        "i64" => EToken::I64,
                        "f64" => EToken::F64,
                        "str" => EToken::Str,
                        "bool" => EToken::Bool,
                        _ => EToken::Ident(l_sIdent),
                    };
                    l_lTokens.push(l_eToken);
                }
                Some(l_cChar) => {
                    return Err(format!("Caractere inattendu: {}", l_cChar));
                }
            }
        }

        Ok(l_lTokens)
    }
}

struct CParser {
    l_lTokens: Vec<EToken>,
    l_iPos: usize,
}

impl CParser {
    fn F_cNew(l_lTokens: Vec<EToken>) -> Self {
        CParser { l_lTokens, l_iPos: 0 }
    }

    fn F_ePeek(&self) -> Option<&EToken> {
        self.l_lTokens.get(self.l_iPos)
    }

    fn F_eAdvance(&mut self) -> Option<&EToken> {
        let l_eToken = self.l_lTokens.get(self.l_iPos);
        self.l_iPos += 1;
        l_eToken
    }

    fn F_bExpect(&mut self, l_eExpected: EToken) -> Result<(), String> {
        match self.F_eAdvance() {
            Some(l_eToken) if l_eToken == &l_eExpected => Ok(()),
            Some(l_eToken) => Err(format!("Expected {:?}, got {:?}", l_eExpected, l_eToken)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn F_lParseProc(&mut self) -> Result<EIrInstr, String> {
        self.F_bExpect(EToken::Proc)?;

        let l_sName = match self.F_eAdvance() {
            Some(EToken::Ident(l_sName)) => l_sName.clone(),
            _ => return Err("Expected proc name".to_string()),
        };

        // parameters parsing can be added

        self.F_bExpect(EToken::In)?;

        let mut l_lBody = Vec::new();
        while let Some(l_eToken) = self.F_ePeek() {
            if l_eToken == &EToken::End {
                self.F_eAdvance();
                break;
            }
            l_lBody.push(self.F_eParseInstr()?);
        }
        l_lBody.push(EIrInstr::Ret);

        let l_sNameStatic = Box::leak(l_sName.into_boxed_str());
        Ok(EIrInstr::Proc(l_sNameStatic, l_lBody))
    }

    fn F_lParseConst(&mut self) -> Result<EIrInstr, String> {
        self.F_bExpect(EToken::Const)?;

        let l_sName = match self.F_eAdvance() {
            Some(EToken::Ident(l_sName)) => l_sName.clone(),
            _ => return Err("Expected const name".to_string()),
        };

        self.F_bExpect(EToken::In)?;

        let mut l_lBody = Vec::new();
        while let Some(l_eToken) = self.F_ePeek() {
            if l_eToken == &EToken::End {
                self.F_eAdvance();
                break;
            }
            l_lBody.push(self.F_eParseInstr()?);
        }
        l_lBody.push(EIrInstr::Ret);

        let l_sNameStatic = Box::leak(l_sName.into_boxed_str());
        Ok(EIrInstr::Proc(l_sNameStatic, l_lBody))
    }

    fn F_eParseInstr(&mut self) -> Result<EIrInstr, String> {
        match self.F_eAdvance() {
            Some(EToken::Number(l_iN)) => Ok(EIrInstr::PushI64(*l_iN)),
            Some(EToken::String(l_sStr)) => {
                let l_sStatic = Box::leak(l_sStr.clone().into_boxed_str());
                Ok(EIrInstr::PushStr(l_sStatic))
            }
            Some(EToken::Plus) => Ok(EIrInstr::AddI64),
            Some(EToken::Dup) => Ok(EIrInstr::Dup),
            Some(EToken::Swap) => Ok(EIrInstr::Swap),
            Some(EToken::Puts) => Ok(EIrInstr::Puts),
            Some(EToken::Dump) => Ok(EIrInstr::Dump),
            Some(EToken::Syscall0) => Ok(EIrInstr::Syscall0),
            Some(EToken::Syscall1) => Ok(EIrInstr::Syscall1),
            Some(EToken::Syscall2) => Ok(EIrInstr::Syscall2),
            Some(EToken::Syscall3) => Ok(EIrInstr::Syscall3),
            Some(EToken::Syscall4) => Ok(EIrInstr::Syscall4),
            Some(EToken::Syscall5) => Ok(EIrInstr::Syscall5),
            Some(EToken::Syscall6) => Ok(EIrInstr::Syscall6),
            Some(EToken::Ident(l_sName)) => {
                let l_sStatic = Box::leak(l_sName.clone().into_boxed_str());
                Ok(EIrInstr::Call(l_sStatic))
            }
            Some(EToken::End) => Ok(EIrInstr::Ret),
            Some(l_eToken) => Err(format!("Unexpected token: {:?}", l_eToken)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn F_lParse(&mut self) -> Result<Vec<EIrInstr>, String> {
        let mut l_lProgram = Vec::new();

        while self.l_iPos < self.l_lTokens.len() {
            if let Some(EToken::Proc) = self.F_ePeek() {
                l_lProgram.push(self.F_lParseProc()?);
            } else if let Some(EToken::Const) = self.F_ePeek() {
                l_lProgram.push(self.F_lParseConst()?);
            } else {
                return Err("Expected proc declaration".to_string());
            }
        }

        Ok(l_lProgram)
    }
}

pub fn F_lParseProgram(l_sInput: &str) -> Result<Vec<EIrInstr>, String> {
    let mut l_cLexer = CLexer::F_cNew(l_sInput.to_string());
    let l_lTokens = l_cLexer.F_lTokenize()?;
    
    let mut l_cParser = CParser::F_cNew(l_lTokens);
    l_cParser.F_lParse()
}
