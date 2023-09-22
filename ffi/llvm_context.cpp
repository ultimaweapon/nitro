#include <llvm/IR/LLVMContext.h>

using namespace llvm;

extern "C" LLVMContext *llvm_context_new()
{
    return new LLVMContext();
}

extern "C" void llvm_context_dispose(LLVMContext *cx)
{
    delete cx;
}
