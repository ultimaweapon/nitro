#include <llvm/IR/BasicBlock.h>

using namespace llvm;

extern "C" BasicBlock *llvm_block_new(LLVMContext *cx)
{
    return BasicBlock::Create(*cx);
}

extern "C" void llvm_block_dispose(BasicBlock *bb)
{
    bb->eraseFromParent();
}
