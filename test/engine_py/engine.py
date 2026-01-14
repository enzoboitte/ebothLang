from typing import Any, Dict, Optional, Union, List, Set

class C_AsmVar:
    def __init__(self, p_sName, p_sType, p_cEmitter):
        self.l_sName = p_sName
        self.l_sType = p_sType
        self.l_cEmitter = p_cEmitter

    def get(self):
        return f"[{self.l_sName}]"

    def cast(self, p_sTargetType):
        return self.l_cEmitter.F_cCast(self, p_sTargetType)

class C_AsmLocalVar:
    def __init__(self, p_sName, p_sType, p_iOffset, p_cEmitter):
        self.l_sName = p_sName
        self.l_sType = p_sType
        self.l_iOffset = p_iOffset
        self.l_cEmitter = p_cEmitter

    def get(self):
        return f"[{self.l_cEmitter.l_dRegs['bp']}-{self.l_iOffset}]"

    def cast(self, p_sTargetType):
        return self.l_cEmitter.F_cCast(self, p_sTargetType)

class C_AsmConst:
    def __init__(self, p_sName, p_sType, p_cEmitter):
        self.l_sName = p_sName
        self.l_sType = p_sType
        self.l_cEmitter = p_cEmitter

    def get(self):
        return f"[{self.l_sName}]"

    def cast(self, p_sTargetType):
        return self.l_cEmitter.F_cCast(self, p_sTargetType)

class C_AsmReg:
    def __init__(self, p_sReg, p_sType, p_cEmitter, p_bTemp=False):
        self.l_sReg = p_sReg
        self.l_sType = p_sType
        self.l_cEmitter = p_cEmitter
        self.l_bTemp = p_bTemp

    def get(self):
        return self.l_sReg

    def cast(self, p_sTargetType):
        return self.l_cEmitter.F_cCast(self, p_sTargetType)

    def free(self):
        if self.l_bTemp:
            self.l_cEmitter.F_vFreeReg(self.l_sReg)

class C_AsmFunc:
    def __init__(self, p_sName, p_dParams, p_cEmitter):
        self.l_sName = p_sName
        self.l_dParams = p_dParams
        self.l_cEmitter = p_cEmitter
        self.l_lSavedRegs = []
        self.l_lFuncLines = []
        self.l_iLocalOffset = 0
        self.l_dLocalVars = {}
        self.params = C_ParamAccess(self, p_dParams)

    def emitLocal(self, p_sName, p_sType='int64'):
        l_iSize = self.l_cEmitter.F_iGetTypeSize(p_sType)
        self.l_iLocalOffset += l_iSize
        l_cVar = C_AsmLocalVar(p_sName, p_sType, self.l_iLocalOffset, self.l_cEmitter)
        self.l_dLocalVars[p_sName] = l_cVar
        return l_cVar

    def ret(self, p_xValue):
        l_sVal = self.l_cEmitter.F_sResolveValue(p_xValue)
        l_bIsFloat = isinstance(p_xValue, C_AsmReg) and 'float' in p_xValue.l_sType

        if l_bIsFloat:
            self.l_lFuncLines.append(f"    movss {self.l_cEmitter.l_dRegs['fret']}, {l_sVal}")
        else:
            self.l_lFuncLines.append(f"    mov {self.l_cEmitter.l_dRegs['ret']}, {l_sVal}")

    def endFunc(self):
        for l_sReg in reversed(self.l_lSavedRegs):
            self.l_lFuncLines.append(f"    pop {l_sReg}")
        self.l_lFuncLines.append(f"    mov {self.l_cEmitter.l_dRegs['sp']}, {self.l_cEmitter.l_dRegs['bp']}")
        self.l_lFuncLines.append(f"    pop {self.l_cEmitter.l_dRegs['bp']}")
        self.l_lFuncLines.append("    ret")

        if self.l_iLocalOffset > 0:
            self.l_lFuncLines.insert(0, f"    sub {self.l_cEmitter.l_dRegs['sp']}, {self.l_iLocalOffset}")

        self.l_cEmitter.l_lFuncs.extend(self.l_lFuncLines)
        self.l_cEmitter.l_cCurrentFunc = None

class C_ParamAccess:
    def __init__(self, p_cFunc, p_dParams):
        self.l_cFunc = p_cFunc
        self.l_dParams = p_dParams
        self.l_dParamRegs = {}

    def get(self, p_sName):
        if p_sName in self.l_dParamRegs:
            return self.l_dParamRegs[p_sName]

        l_sType = self.l_dParams[p_sName]
        l_iOffset = (list(self.l_dParams.keys()).index(p_sName) + 2) * self.l_cFunc.l_cEmitter.l_iWordSize

        if 'float' in l_sType or 'double' in l_sType:
            l_sReg = self.l_cFunc.l_cEmitter.F_sAllocFloatReg()
            self.l_cFunc.l_lFuncLines.append(f"    movss {l_sReg}, [{self.l_cFunc.l_cEmitter.l_dRegs['bp']}+{l_iOffset}]")
            l_cReg = C_AsmReg(l_sReg, l_sType, self.l_cFunc.l_cEmitter)
        else:
            l_sReg = self.l_cFunc.l_cEmitter.F_sAllocReg()
            l_iSize = self.l_cFunc.l_cEmitter.F_iGetTypeSize(l_sType)
            l_sSizedReg = self.l_cFunc.l_cEmitter.F_sResizeReg(l_sReg, l_iSize)
            self.l_cFunc.l_lFuncLines.append(f"    mov {l_sSizedReg}, [{self.l_cFunc.l_cEmitter.l_dRegs['bp']}+{l_iOffset}]")
            l_cReg = C_AsmReg(l_sReg, l_sType, self.l_cFunc.l_cEmitter)

        self.l_dParamRegs[p_sName] = l_cReg
        return l_cReg

class C_AsmLite:
    def __init__(self, p_sBits):
        self.l_sBits = p_sBits
        self.l_iBits = int(p_sBits.replace("bits_", ""))
        self.l_iWordSize = self.l_iBits // 8

        self.l_lBss = []
        self.l_lData = []
        self.l_lFuncs = []
        self.l_lText = []

        self.l_lRegPool = self.F_lInitRegPool()
        self.l_lFloatRegPool = ['xmm0', 'xmm1', 'xmm2', 'xmm3', 'xmm4', 'xmm5', 'xmm6', 'xmm7']
        self.l_lUsedRegs = []
        self.l_lUsedFloatRegs = []

        self.l_dRegs = self.F_dGetSysRegs()
        self.l_cCurrentFunc = None
        self.l_iPrintBufCounter = 0

        self.l_bPrintHelperAdded = False
        self.l_bPrintFloatHelperAdded = False

    def F_lInitRegPool(self):
        if self.l_iBits == 64:
            return ['rbx', 'r12', 'r13', 'r14', 'r15', 'r10', 'r11', 'r8', 'r9', 'rcx', 'rdx']
        elif self.l_iBits == 32:
            return ['ebx', 'ecx', 'edx', 'esi', 'edi']
        elif self.l_iBits == 16:
            return ['bx', 'cx', 'dx', 'si', 'di']
        return ['bl', 'cl', 'dl']

    def F_dGetSysRegs(self):
        if self.l_iBits == 64:
            return {'sp': 'rsp', 'bp': 'rbp', 'ret': 'rax', 'ax': 'rax', 'fret': 'xmm0'}
        elif self.l_iBits == 32:
            return {'sp': 'esp', 'bp': 'ebp', 'ret': 'eax', 'ax': 'eax', 'fret': 'xmm0'}
        elif self.l_iBits == 16:
            return {'sp': 'sp', 'bp': 'bp', 'ret': 'ax', 'ax': 'ax', 'fret': 'st0'}
        return {'sp': 'sp', 'bp': 'bp', 'ret': 'al', 'ax': 'al', 'fret': 'st0'}

    def F_sAllocReg(self, p_bTemp=False):
        for l_sReg in self.l_lRegPool:
            if l_sReg not in self.l_lUsedRegs:
                self.l_lUsedRegs.append(l_sReg)

                if self.l_cCurrentFunc and l_sReg not in self.l_cCurrentFunc.l_lSavedRegs:
                    self.l_cCurrentFunc.l_lSavedRegs.append(l_sReg)
                    self.l_cCurrentFunc.l_lFuncLines.insert(0, f"    push {l_sReg}")

                return l_sReg

        l_sReg = self.l_lRegPool[0]
        return l_sReg

    def F_sAllocFloatReg(self):
        for l_sReg in self.l_lFloatRegPool:
            if l_sReg not in self.l_lUsedFloatRegs:
                self.l_lUsedFloatRegs.append(l_sReg)
                return l_sReg

        return self.l_lFloatRegPool[0]

    def F_vFreeReg(self, p_sReg):
        if p_sReg in self.l_lUsedRegs:
            self.l_lUsedRegs.remove(p_sReg)

    def F_vFreeFloatReg(self, p_sReg):
        if p_sReg in self.l_lUsedFloatRegs:
            self.l_lUsedFloatRegs.remove(p_sReg)

    def F_cAllocTypedReg(self, p_sType='int64', p_bTemp=True):
        if 'float' in p_sType or 'double' in p_sType:
            l_sReg = self.F_sAllocFloatReg()
        else:
            l_sReg = self.F_sAllocReg(p_bTemp=p_bTemp)
        return C_AsmReg(l_sReg, p_sType, self, p_bTemp=p_bTemp)

    def F_cLoadValue(self, p_xValue, p_sTargetType=None):
        if isinstance(p_xValue, C_AsmReg):
            return p_xValue
        
        l_sType = p_sTargetType or getattr(p_xValue, 'l_sType', 'int64')
        l_cReg = self.F_cAllocTypedReg(l_sType, p_bTemp=True)
        l_sVal = self.F_sResolveValue(p_xValue)
        
        if 'float' in l_sType or 'double' in l_sType:
            l_sInstr = 'movss' if l_sType == 'float' else 'movsd'
            self.F_vAddText(f"    {l_sInstr} {l_cReg.get()}, {l_sVal}")
        else:
            l_iSize = self.F_iGetTypeSize(l_sType)
            l_sSizedReg = self.F_sResizeReg(l_cReg.get(), l_iSize)
            self.F_vAddText(f"    mov {l_sSizedReg}, {l_sVal}")
        
        return l_cReg

    def emitDecl(self, p_sName, p_sType='int64'):
        l_iSize = self.F_iGetTypeSize(p_sType)
        self.l_lBss.append(f"    {p_sName}: resb {l_iSize}")
        return C_AsmVar(p_sName, p_sType, self)

    def emitConst(self, p_sName, p_xValue, p_sType=None):
        if p_sType is None:
            p_sType = 'float' if isinstance(p_xValue, float) else f'int{self.l_iBits}'

        l_sDataType = self.F_sGetDataType(p_sType)
        l_sVal = self.F_sFormatValue(p_xValue, p_sType)
        self.l_lData.append(f"    {p_sName}: {l_sDataType} {l_sVal}")
        return C_AsmConst(p_sName, p_sType, self)

    def F_iGetTypeSize(self, p_sType):
        if 'float' in p_sType or 'double' in p_sType:
            return 4 if p_sType == 'float' else 8
        elif 'int' in p_sType:
            return int(p_sType.replace('int', '')) // 8
        return self.l_iWordSize

    def F_sGetDataType(self, p_sType):
        l_dTypes = {
            'float': 'dd', 'double': 'dq',
            'int8': 'db', 'int16': 'dw', 'int32': 'dd', 'int64': 'dq'
        }
        return l_dTypes.get(p_sType, 'dd' if self.l_iBits == 32 else 'dq')

    def F_sFormatValue(self, p_xValue, p_sType='int64'):
        if isinstance(p_xValue, str):
            return f'"{p_xValue}", 0'
        elif isinstance(p_xValue, float):
            return str(p_xValue)
        return str(p_xValue)

    def F_sResolveValue(self, p_xValue):
        if isinstance(p_xValue, (C_AsmVar, C_AsmConst, C_AsmReg, C_AsmLocalVar)):
            return p_xValue.get()
        return str(p_xValue)

    def F_vAddPrintHelper(self):
        if self.l_bPrintHelperAdded:
            return

        self.l_lBss.append("    print_buffer: resb 32")
        self.l_lData.append("    newline: db 10")

        if self.l_iBits == 32:
            self.l_lFuncs.extend([
                "print_int:",
                "    push ebp",
                "    mov ebp, esp",
                "    push ebx",
                "    push esi",
                "    push edi",
                "    mov eax, [ebp+8]",
                "    lea esi, [print_buffer+31]",
                "    mov byte [esi], 0",
                "    dec esi",
                "    mov ebx, 10",
                "    test eax, eax",
                "    jns .positive",
                "    neg eax",
                "    mov edi, 1",
                "    jmp .convert",
                ".positive:",
                "    xor edi, edi",
                ".convert:",
                "    xor edx, edx",
                "    div ebx",
                "    add dl, '0'",
                "    mov [esi], dl",
                "    dec esi",
                "    test eax, eax",
                "    jnz .convert",
                "    test edi, edi",
                "    jz .print",
                "    mov byte [esi], '-'",
                "    dec esi",
                ".print:",
                "    inc esi",
                "    mov eax, 4",
                "    mov ebx, 1",
                "    mov ecx, esi",
                "    lea edx, [print_buffer+31]",
                "    sub edx, esi",
                "    int 0x80",
                "    mov eax, 4",
                "    mov ebx, 1",
                "    mov ecx, newline",
                "    mov edx, 1",
                "    int 0x80",
                "    pop edi",
                "    pop esi",
                "    pop ebx",
                "    mov esp, ebp",
                "    pop ebp",
                "    ret"
            ])
        else:
            self.l_lFuncs.extend([
                "print_int:",
                "    push rbp",
                "    mov rbp, rsp",
                "    push rbx",
                "    push r12",
                "    push r13",
                "    mov rax, [rbp+16]",
                "    lea rsi, [print_buffer+31]",
                "    mov byte [rsi], 0",
                "    dec rsi",
                "    mov rbx, 10",
                "    test rax, rax",
                "    jns .positive",
                "    neg rax",
                "    mov r13, 1",
                "    jmp .convert",
                ".positive:",
                "    xor r13, r13",
                ".convert:",
                "    xor rdx, rdx",
                "    div rbx",
                "    add dl, '0'",
                "    mov [rsi], dl",
                "    dec rsi",
                "    test rax, rax",
                "    jnz .convert",
                "    test r13, r13",
                "    jz .print",
                "    mov byte [rsi], '-'",
                "    dec rsi",
                ".print:",
                "    inc rsi",
                "    mov rax, 1",
                "    mov rdi, 1",
                "    lea rdx, [print_buffer+31]",
                "    sub rdx, rsi",
                "    syscall",
                "    mov rax, 1",
                "    mov rdi, 1",
                "    lea rsi, [newline]",
                "    mov rdx, 1",
                "    syscall",
                "    pop r13",
                "    pop r12",
                "    pop rbx",
                "    mov rsp, rbp",
                "    pop rbp",
                "    ret"
            ])

        self.l_bPrintHelperAdded = True

    def F_vAddPrintFloatHelper(self):
        if self.l_bPrintFloatHelperAdded:
            return

        self.F_vAddPrintHelper()

        if not any("float_buf" in line for line in self.l_lBss):
            self.l_lBss.append("    float_buf: resb 64")

        if self.l_iBits == 64:
            self.l_lFuncs.extend([
                "print_float:",
                "    push rbp",
                "    mov rbp, rsp",
                "    push rbx",
                "    push r12",
                "    push r13",
                "    push r14",
                "    push r15",
                "    movss xmm0, [rbp+16]",
                "    cvtss2sd xmm0, xmm0",
                "    pxor xmm1, xmm1",
                "    ucomisd xmm0, xmm1",
                "    jae .positive_float",
                "    mov byte [float_buf], '-'",
                "    mov r14, 1",
                "    movsd xmm1, xmm0",
                "    xorpd xmm0, xmm0",
                "    subsd xmm0, xmm1",
                "    jmp .extract_int",
                ".positive_float:",
                "    xor r14, r14",
                ".extract_int:",
                "    cvttsd2si rax, xmm0",
                "    cvtsi2sd xmm1, rax",
                "    subsd xmm0, xmm1",
                "    lea rsi, [float_buf+31]",
                "    mov byte [rsi], 0",
                "    dec rsi",
                "    mov rbx, 10",
                ".int_loop:",
                "    xor rdx, rdx",
                "    div rbx",
                "    add dl, '0'",
                "    mov [rsi], dl",
                "    dec rsi",
                "    test rax, rax",
                "    jnz .int_loop",
                "    test r14, r14",
                "    jz .write_point",
                "    mov byte [rsi], '-'",
                "    dec rsi",
                ".write_point:",
                "    inc rsi",
                "    mov r14, rsi",
                "    lea rsi, [float_buf+32]",
                "    mov byte [rsi], '.'",
                "    inc rsi",
                "    mov rcx, 6",
                ".frac_loop:",
                "    mov r15, 10",
                "    cvtsi2sd xmm2, r15",
                "    mulsd xmm0, xmm2",
                "    cvttsd2si r12, xmm0",
                "    cvtsi2sd xmm1, r12",
                "    subsd xmm0, xmm1",
                "    add r12, '0'",
                "    mov [rsi], r12b",
                "    inc rsi",
                "    loop .frac_loop",
                "    mov byte [rsi], 10",
                "    inc rsi",
                "    mov rax, 1",
                "    mov rdi, 1",
                "    mov rdx, rsi",
                "    sub rdx, r14",
                "    mov rsi, r14",
                "    syscall",
                "    pop r15",
                "    pop r14",
                "    pop r13",
                "    pop r12",
                "    pop rbx",
                "    mov rsp, rbp",
                "    pop rbp",
                "    ret"
            ])
        else:
            self.l_lFuncs.extend([
                "print_float:",
                "    push ebp",
                "    mov ebp, esp",
                "    push ebx",
                "    push esi",
                "    push edi",
                "    movss xmm0, [ebp+8]",
                "    cvtss2sd xmm0, xmm0",
                "    pxor xmm1, xmm1",
                "    ucomisd xmm0, xmm1",
                "    jae .positive_float",
                "    mov byte [float_buf], '-'",
                "    mov edi, 1",
                "    movsd xmm1, xmm0",
                "    xorpd xmm0, xmm0",
                "    subsd xmm0, xmm1",
                "    jmp .extract_int",
                ".positive_float:",
                "    xor edi, edi",
                ".extract_int:",
                "    cvttsd2si eax, xmm0",
                "    cvtsi2sd xmm1, eax",
                "    subsd xmm0, xmm1",
                "    lea esi, [float_buf+31]",
                "    mov byte [esi], 0",
                "    dec esi",
                "    mov ebx, 10",
                ".int_loop:",
                "    xor edx, edx",
                "    div ebx",
                "    add dl, '0'",
                "    mov [esi], dl",
                "    dec esi",
                "    test eax, eax",
                "    jnz .int_loop",
                "    test edi, edi",
                "    jz .write_point",
                "    mov byte [esi], '-'",
                "    dec esi",
                ".write_point:",
                "    inc esi",
                "    push esi",
                "    lea esi, [float_buf+32]",
                "    mov byte [esi], '.'",
                "    inc esi",
                "    mov ecx, 6",
                ".frac_loop:",
                "    push ecx",
                "    mov ecx, 10",
                "    cvtsi2sd xmm2, ecx",
                "    mulsd xmm0, xmm2",
                "    cvttsd2si ecx, xmm0",
                "    push ecx",
                "    cvtsi2sd xmm1, ecx",
                "    subsd xmm0, xmm1",
                "    pop ecx",
                "    add cl, '0'",
                "    mov [esi], cl",
                "    inc esi",
                "    pop ecx",
                "    loop .frac_loop",
                "    mov byte [esi], 10",
                "    inc esi",
                "    mov eax, 4",
                "    mov ebx, 1",
                "    pop ecx",
                "    mov edx, esi",
                "    sub edx, ecx",
                "    int 0x80",
                "    pop edi",
                "    pop esi",
                "    pop ebx",
                "    mov esp, ebp",
                "    pop ebp",
                "    ret"
            ])

        self.l_bPrintFloatHelperAdded = True

    def emitPrint(self, p_xValue):
        if isinstance(p_xValue, str):
            l_sBufName = f"str_print_{self.l_iPrintBufCounter}"
            self.l_iPrintBufCounter += 1
            l_iLen = len(p_xValue)
            self.l_lData.append(f"    {l_sBufName}: db \"{p_xValue}\", 10")

            if self.l_iBits == 32:
                self.l_lText.append(f"    mov eax, 4")
                self.l_lText.append(f"    mov ebx, 1")
                self.l_lText.append(f"    mov ecx, {l_sBufName}")
                self.l_lText.append(f"    mov edx, {l_iLen + 1}")
                self.l_lText.append(f"    int 0x80")
            else:
                self.l_lText.append(f"    mov rax, 1")
                self.l_lText.append(f"    mov rdi, 1")
                self.l_lText.append(f"    lea rsi, [{l_sBufName}]")
                self.l_lText.append(f"    mov rdx, {l_iLen + 1}")
                self.l_lText.append(f"    syscall")
        else:
            l_sType = getattr(p_xValue, 'l_sType', 'int64')

            if 'float' in l_sType or 'double' in l_sType:
                self.emitPrintFloat(p_xValue)
            else:
                self.F_vAddPrintHelper()
                l_sVal = self.F_sResolveValue(p_xValue)

                if isinstance(p_xValue, C_AsmReg):
                    self.l_lText.append(f"    push {l_sVal}")
                else:
                    l_sTempReg = self.F_sAllocReg(p_bTemp=True)
                    self.l_lText.append(f"    mov {l_sTempReg}, {l_sVal}")
                    self.l_lText.append(f"    push {l_sTempReg}")

                self.l_lText.append(f"    call print_int")
                self.l_lText.append(f"    add {self.l_dRegs['sp']}, {self.l_iWordSize}")

    def emitPrintFloat(self, p_xValue):
        self.F_vAddPrintFloatHelper()
        l_sVal = self.F_sResolveValue(p_xValue)

        if isinstance(p_xValue, C_AsmReg):
            l_sTempReg = self.F_sAllocReg(p_bTemp=True)
            self.l_lText.append(f"    sub {self.l_dRegs['sp']}, 8")
            self.l_lText.append(f"    movss [{self.l_dRegs['sp']}], {l_sVal}")
        else:
            l_sTempReg = self.F_sAllocFloatReg()
            self.l_lText.append(f"    movss {l_sTempReg}, {l_sVal}")
            self.l_lText.append(f"    sub {self.l_dRegs['sp']}, 8")
            self.l_lText.append(f"    movss [{self.l_dRegs['sp']}], {l_sTempReg}")

        self.l_lText.append(f"    call print_float")
        self.l_lText.append(f"    add {self.l_dRegs['sp']}, 8")

    def F_cCast(self, p_xValue, p_sTargetType):
        l_sSourceType = getattr(p_xValue, 'l_sType', 'int64')

        l_bSourceFloat = 'float' in l_sSourceType or 'double' in l_sSourceType
        l_bTargetFloat = 'float' in p_sTargetType or 'double' in p_sTargetType

        if l_bSourceFloat and not l_bTargetFloat:
            return self.F_cFloatToInt(p_xValue, p_sTargetType)
        elif not l_bSourceFloat and l_bTargetFloat:
            return self.F_cIntToFloat(p_xValue, p_sTargetType)
        elif l_bSourceFloat and l_bTargetFloat:
            return self.F_cFloatToFloat(p_xValue, p_sTargetType)
        else:
            return self.F_cIntToInt(p_xValue, l_sSourceType, p_sTargetType)

    def F_cIntToInt(self, p_xVal, p_sSourceType, p_sTargetType):
        l_iSourceSize = self.F_iGetTypeSize(p_sSourceType)
        l_iTargetSize = self.F_iGetTypeSize(p_sTargetType)
        l_sDestReg = self.F_sAllocReg()
        l_sSrcVal = self.F_sResolveValue(p_xVal)

        if l_iSourceSize < l_iTargetSize:
            l_sSrcSize = self.F_sGetRegSize(l_iSourceSize)
            self.F_vAddText(f"    movsx {l_sDestReg}, {l_sSrcSize} {l_sSrcVal}")
        elif l_iSourceSize > l_iTargetSize:
            l_sDestSized = self.F_sResizeReg(l_sDestReg, l_iTargetSize)
            self.F_vAddText(f"    mov {l_sDestSized}, {l_sSrcVal}")
        else:
            self.F_vAddText(f"    mov {l_sDestReg}, {l_sSrcVal}")

        return C_AsmReg(l_sDestReg, p_sTargetType, self, p_bTemp=True)

    def F_cIntToFloat(self, p_xVal, p_sTargetType):
        l_sDestReg = self.F_sAllocFloatReg()
        l_sInstr = 'cvtsi2ss' if p_sTargetType == 'float' else 'cvtsi2sd'

        if isinstance(p_xVal, C_AsmReg):
            self.F_vAddText(f"    {l_sInstr} {l_sDestReg}, {p_xVal.get()}")
        else:
            l_sTempReg = self.F_sAllocReg(p_bTemp=True)
            self.F_vAddText(f"    mov {l_sTempReg}, {self.F_sResolveValue(p_xVal)}")
            self.F_vAddText(f"    {l_sInstr} {l_sDestReg}, {l_sTempReg}")

        return C_AsmReg(l_sDestReg, p_sTargetType, self, p_bTemp=True)

    def F_cFloatToInt(self, p_xVal, p_sTargetType):
        l_sDestReg = self.F_sAllocReg()
        l_sSourceType = getattr(p_xVal, 'l_sType', 'float')
        l_sInstr = 'cvtss2si' if l_sSourceType == 'float' else 'cvtsd2si'

        if isinstance(p_xVal, C_AsmReg):
            self.F_vAddText(f"    {l_sInstr} {l_sDestReg}, {p_xVal.get()}")
        else:
            l_sTempReg = self.F_sAllocFloatReg()
            l_sMovInstr = 'movss' if l_sSourceType == 'float' else 'movsd'
            self.F_vAddText(f"    {l_sMovInstr} {l_sTempReg}, {self.F_sResolveValue(p_xVal)}")
            self.F_vAddText(f"    {l_sInstr} {l_sDestReg}, {l_sTempReg}")

        return C_AsmReg(l_sDestReg, p_sTargetType, self, p_bTemp=True)

    def F_cFloatToFloat(self, p_xVal, p_sTargetType):
        l_sDestReg = self.F_sAllocFloatReg()
        l_sSourceType = getattr(p_xVal, 'l_sType', 'float')

        if l_sSourceType == 'float' and p_sTargetType == 'double':
            self.F_vAddText(f"    cvtss2sd {l_sDestReg}, {self.F_sResolveValue(p_xVal)}")
        elif l_sSourceType == 'double' and p_sTargetType == 'float':
            self.F_vAddText(f"    cvtsd2ss {l_sDestReg}, {self.F_sResolveValue(p_xVal)}")
        else:
            l_sMovInstr = 'movss' if p_sTargetType == 'float' else 'movsd'
            self.F_vAddText(f"    {l_sMovInstr} {l_sDestReg}, {self.F_sResolveValue(p_xVal)}")

        return C_AsmReg(l_sDestReg, p_sTargetType, self, p_bTemp=True)

    def F_sGetRegSize(self, p_iSize):
        return {1: 'byte', 2: 'word', 4: 'dword', 8: 'qword'}.get(p_iSize, 'dword' if self.l_iBits == 32 else 'qword')

    def F_sResizeReg(self, p_sReg, p_iSize):
        if self.l_iBits == 64:
            l_dMap = {
                'rbx': {1: 'bl', 2: 'bx', 4: 'ebx', 8: 'rbx'},
                'r12': {1: 'r12b', 2: 'r12w', 4: 'r12d', 8: 'r12'},
                'r13': {1: 'r13b', 2: 'r13w', 4: 'r13d', 8: 'r13'},
                'r14': {1: 'r14b', 2: 'r14w', 4: 'r14d', 8: 'r14'},
                'r15': {1: 'r15b', 2: 'r15w', 4: 'r15d', 8: 'r15'},
                'r10': {1: 'r10b', 2: 'r10w', 4: 'r10d', 8: 'r10'},
                'r11': {1: 'r11b', 2: 'r11w', 4: 'r11d', 8: 'r11'},
                'r8': {1: 'r8b', 2: 'r8w', 4: 'r8d', 8: 'r8'},
                'r9': {1: 'r9b', 2: 'r9w', 4: 'r9d', 8: 'r9'},
                'rcx': {1: 'cl', 2: 'cx', 4: 'ecx', 8: 'rcx'},
                'rdx': {1: 'dl', 2: 'dx', 4: 'edx', 8: 'rdx'},
            }
            return l_dMap.get(p_sReg, {}).get(p_iSize, p_sReg)
        elif self.l_iBits == 32:
            l_dMap = {
                'ebx': {1: 'bl', 2: 'bx', 4: 'ebx'},
                'ecx': {1: 'cl', 2: 'cx', 4: 'ecx'},
                'edx': {1: 'dl', 2: 'dx', 4: 'edx'},
                'esi': {1: 'sil', 2: 'si', 4: 'esi'},
                'edi': {1: 'dil', 2: 'di', 4: 'edi'},
            }
            return l_dMap.get(p_sReg, {}).get(p_iSize, p_sReg)
        return p_sReg

    def emitFunc(self, p_sName, p_dParams):
        l_lFuncHeader = [
            f"{p_sName}:",
            f"    push {self.l_dRegs['bp']}",
            f"    mov {self.l_dRegs['bp']}, {self.l_dRegs['sp']}"
        ]
        self.l_lFuncs.extend(l_lFuncHeader)

        l_cFunc = C_AsmFunc(p_sName, p_dParams, self)
        self.l_cCurrentFunc = l_cFunc
        self.l_lUsedRegs = []
        self.l_lUsedFloatRegs = []
        return l_cFunc

    def call(self, p_sName, p_dParams):
        for l_sKey, l_xVal in reversed(list(p_dParams.items())):
            l_sVal = self.F_sResolveValue(l_xVal)

            if isinstance(l_xVal, C_AsmReg):
                self.l_lText.append(f"    push {l_sVal}")
            else:
                l_sTempReg = self.F_sAllocReg(p_bTemp=True)
                l_iSize = self.F_iGetTypeSize(getattr(l_xVal, 'l_sType', 'int32' if self.l_iBits == 32 else 'int64'))
                l_sSizedReg = self.F_sResizeReg(l_sTempReg, l_iSize)
                self.l_lText.append(f"    mov {l_sSizedReg}, {l_sVal}")
                self.l_lText.append(f"    push {l_sTempReg}")

        self.l_lText.append(f"    call {p_sName}")

        if p_dParams:
            l_iCleanup = len(p_dParams) * self.l_iWordSize
            self.l_lText.append(f"    add {self.l_dRegs['sp']}, {l_iCleanup}")

        l_sReg = self.F_sAllocReg()
        self.l_lText.append(f"    mov {l_sReg}, {self.l_dRegs['ret']}")
        return C_AsmReg(l_sReg, 'int32' if self.l_iBits == 32 else 'int64', self)

    def addInstrLine(self, p_sInstr):
        self.F_vAddText(f"    {p_sInstr}")

    def F_vAddText(self, p_sLine):
        if self.l_cCurrentFunc:
            self.l_cCurrentFunc.l_lFuncLines.append(p_sLine)
        else:
            self.l_lText.append(p_sLine)

    def emitLines(self, p_sCode):
        for l_sLine in p_sCode.strip().split('\n'):
            l_sStripped = l_sLine.strip()
            if l_sStripped:
                self.F_vAddText(f"    {l_sStripped}")

    def end(self):
        l_lOutput = [f"bits {self.l_iBits}"]

        if self.l_lBss:
            l_lOutput.append("\nsection .bss")
            l_lOutput.extend(self.l_lBss)

        if self.l_lData:
            l_lOutput.append("\nsection .data")
            l_lOutput.extend(self.l_lData)

        l_lOutput.append("\nsection .text")
        l_lOutput.append("    global _start")

        if self.l_lFuncs:
            l_lOutput.append("")
            l_lOutput.extend(self.l_lFuncs)

        l_lOutput.append("\n_start:")
        l_lOutput.extend(self.l_lText)

        return '\n'.join(l_lOutput)
    
    def emitExit(self, p_iCode=0):
        if self.l_iBits == 64:
            self.F_vAddText(f"    mov {self.l_dRegs['ret']}, 60")
            self.F_vAddText(f"    mov rdi, {p_iCode}")
            self.F_vAddText("    syscall")
        elif self.l_iBits == 32:
            self.F_vAddText(f"    mov {self.l_dRegs['ret']}, 1")
            self.F_vAddText(f"    xor ebx, ebx")
            self.F_vAddText("    int 0x80")
        elif self.l_iBits == 16:
            self.F_vAddText(f"    mov ah, 0x4C")
            self.F_vAddText(f"    mov al, {p_iCode}")
            self.F_vAddText("    int 0x21")
        else:
            self.F_vAddText(f"    mov ah, 0x4C")
            self.F_vAddText(f"    mov al, {p_iCode}")
            self.F_vAddText("    int 0x21")



l_cAsm = C_AsmLite("bits_64")

l_cPi = l_cAsm.emitConst("pi", 3.14159, "float")
l_cNum = l_cAsm.emitConst("number", 42, "int32")

l_cAsm.emitPrint("=== Test Allocation Intelligente ===")

l_cFunc = l_cAsm.emitFunc("test_locals", {"x": "int32"})
l_cLocalA = l_cFunc.emitLocal("local_a", "int32")
l_cLocalB = l_cFunc.emitLocal("local_b", "float")

l_cParamX = l_cFunc.params.get("x")
l_cReg1 = l_cAsm.F_cLoadValue(l_cParamX, "int32")
l_cReg2 = l_cAsm.F_cAllocTypedReg("int32", p_bTemp=True)
l_cAsm.addInstrLine(f"mov {l_cReg2.get()}, {l_cReg1.get()}")
l_cAsm.addInstrLine(f"add {l_cReg2.get()}, 10")
l_cAsm.addInstrLine(f"mov {l_cLocalA.get()}, {l_cReg2.get()}")
l_cReg2.free()

l_cFloatReg = l_cAsm.F_cLoadValue(l_cPi, "float")
l_cAsm.addInstrLine(f"movss {l_cLocalB.get()}, {l_cFloatReg.get()}")
l_cFloatReg.free()

l_cRetReg = l_cAsm.F_cLoadValue(l_cLocalA, "int32")
l_cFunc.ret(l_cRetReg)
l_cFunc.endFunc()

l_cRetVal = l_cAsm.call("test_locals", {"x": 5})
l_cAsm.emitPrint(l_cRetVal)

l_cAsm.emitPrint("=== Test Float Automatique ===")
l_cAsm.emitPrintFloat(l_cPi)

l_cFloatVar = l_cAsm.emitDecl("float_test", "float")
l_cF2 = l_cAsm.emitConst("f2", 2.718, "float")
l_cLoadedF2 = l_cAsm.F_cLoadValue(l_cF2, "float")
l_cAsm.addInstrLine(f"movss {l_cFloatVar.get()}, {l_cLoadedF2.get()}")
l_cAsm.emitPrintFloat(l_cFloatVar)
l_cLoadedF2.free()

l_cAsm.emitExit(0)

with open("output.asm", "w") as f:
    f.write(l_cAsm.end())

print("Code amélioré: output.asm")
