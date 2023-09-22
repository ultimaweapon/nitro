#include <llvm/IR/Constants.h>

using namespace llvm;

extern "C" ConstantInt *llvm_integer_const(IntegerType *ty, uint64_t val, bool sign)
{
    return ConstantInt::get(ty, val, sign);
}
