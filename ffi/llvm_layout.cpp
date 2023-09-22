#include <llvm/IR/DataLayout.h>
#include <llvm/Target/TargetMachine.h>

using namespace llvm;

extern "C" DataLayout *llvm_layout_new(const TargetMachine *mc)
{
    return new DataLayout(mc->createDataLayout());
}

extern "C" void llvm_layout_dispose(DataLayout *dl)
{
    delete dl;
}

extern "C" unsigned llvm_layout_pointer_size(const DataLayout *dl)
{
    return dl->getPointerSize();
}
