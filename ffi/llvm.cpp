#include "nitro.hpp"

#include <llvm/Support/TargetSelect.h>
#include <llvm/TargetParser/Host.h>

extern "C" void llvm_init()
{
    llvm::InitializeAllTargetInfos();
    llvm::InitializeAllTargets();
    llvm::InitializeAllTargetMCs();
    llvm::InitializeAllAsmPrinters();
}

extern "C" void llvm_process_triple(nitro_string &t)
{
    nitro_string_set(t, llvm::sys::getProcessTriple().c_str());
}
