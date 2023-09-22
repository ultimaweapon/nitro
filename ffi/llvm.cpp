#include <llvm/Support/TargetSelect.h>

extern "C" void llvm_init()
{
    llvm::InitializeAllTargetInfos();
    llvm::InitializeAllTargets();
    llvm::InitializeAllTargetMCs();
    llvm::InitializeAllAsmPrinters();
}
