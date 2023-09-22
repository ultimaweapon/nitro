#include <llvm/IR/IRBuilder.h>

using namespace llvm;

extern "C" IRBuilder<> *llvm_builder_new(LLVMContext *cx)
{
    return new IRBuilder<>(*cx);
}

extern "C" void llvm_builder_dispose(IRBuilder<> *ib)
{
    delete ib;
}

extern "C" void llvm_builder_append_block(IRBuilder<> *ib, BasicBlock *bb)
{
    ib->SetInsertPoint(bb);
}

extern "C" ReturnInst *llvm_builder_ret_void(IRBuilder<> *ib)
{
    return ib->CreateRetVoid();
}
