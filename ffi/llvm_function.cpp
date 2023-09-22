#include <llvm/IR/Function.h>

using namespace llvm;

extern "C" Function *llvm_function_new(Module *md, FunctionType *type, const char *name)
{
    return Function::Create(type, GlobalValue::ExternalLinkage, name, md);
}

extern "C" void llvm_function_append(Function *fn, BasicBlock *bb)
{
    fn->insert(fn->end(), bb);
}
