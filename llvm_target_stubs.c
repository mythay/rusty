// FORK(no_std): see build.rs. No-op stand-ins for the LLVM target initializers that the
// fork does not link: inkwell compiles `Target::initialize_<target>` for every target
// unconditionally, so any codegen unit pulled into the link would otherwise reference
// backend libraries the fork dropped. The fork only initializes X86 and WebAssembly
// through inkwell; calling a stubbed target's init is a no-op and its triple is then
// rejected by `Target::from_triple`, which is the documented fork limitation.

void LLVM_InitializeAllTargetInfos(void) {}
void LLVM_InitializeAllTargets(void) {}
void LLVM_InitializeAllTargetMCs(void) {}
void LLVM_InitializeAllAsmPrinters(void) {}
void LLVM_InitializeAllAsmParsers(void) {}
void LLVM_InitializeAllDisassemblers(void) {}
void LLVM_InitializeNativeTarget(void) {}
void LLVM_InitializeNativeAsmParser(void) {}
void LLVM_InitializeNativeAsmPrinter(void) {}
void LLVM_InitializeNativeDisassembler(void) {}

#define STUB(target)                                                                          \
    void LLVMInitialize##target##Target(void) {}                                              \
    void LLVMInitialize##target##TargetInfo(void) {}                                          \
    void LLVMInitialize##target##TargetMC(void) {}                                            \
    void LLVMInitialize##target##AsmPrinter(void) {}                                          \
    void LLVMInitialize##target##AsmParser(void) {}                                           \
    void LLVMInitialize##target##Disassembler(void) {}

STUB(AArch64)
STUB(AMDGPU)
STUB(ARM)
STUB(AVR)
STUB(BPF)
STUB(Hexagon)
STUB(Lanai)
STUB(LoongArch)
STUB(Mips)
STUB(MSP430)
STUB(NVPTX)
STUB(PowerPC)
STUB(RISCV)
STUB(Sparc)
STUB(SPIRV)
STUB(SystemZ)
STUB(VE)
STUB(X86)
STUB(XCore)
