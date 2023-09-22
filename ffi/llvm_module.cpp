#include <llvm/IR/Module.h>

using namespace llvm;

extern "C" Module *llvm_module_new(LLVMContext *cx, const char *id)
{
    return new Module(id, *cx);
}

extern "C" void llvm_module_dispose(Module *md)
{
    delete md;
}

extern "C" void llvm_module_set_layout(Module *md, const DataLayout *dl)
{
    md->setDataLayout(*dl);
}

extern "C" Function *llvm_module_get_function(const Module *md, const char *name)
{
    return md->getFunction(name);
}
