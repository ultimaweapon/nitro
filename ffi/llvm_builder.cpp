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

extern "C" CallInst *llvm_builder_call(IRBuilder<> *ib, Function *fn, Value **args, size_t nargs)
{
    return ib->CreateCall(fn->getFunctionType(), fn, ArrayRef<Value *>(args, nargs));
}

extern "C" ReturnInst *llvm_builder_ret_void(IRBuilder<> *ib)
{
    return ib->CreateRetVoid();
}

extern "C" ReturnInst *llvm_builder_ret(IRBuilder<> *ib, Value *v)
{
    return ib->CreateRet(v);
}
