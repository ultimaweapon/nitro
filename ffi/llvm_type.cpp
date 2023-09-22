#include <llvm/IR/DerivedTypes.h>
#include <llvm/IR/Type.h>

using namespace llvm;

extern "C" Type *llvm_type_void(LLVMContext *cx)
{
    return Type::getVoidTy(*cx);
}

extern "C" IntegerType *llvm_type_int8(LLVMContext *cx)
{
    return Type::getInt8Ty(*cx);
}

extern "C" IntegerType *llvm_type_int64(LLVMContext *cx)
{
    return Type::getInt64Ty(*cx);
}

extern "C" PointerType *llvm_type_ptr(LLVMContext *cx)
{
    return PointerType::get(*cx, 0);
}

extern "C" FunctionType *llvm_type_func(Type *ret, Type *params[], size_t count, bool va)
{
    ArrayRef<Type *> arr(params, count);
    return FunctionType::get(ret, arr, va);
}
