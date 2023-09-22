#include "nitro.hpp"

#include <llvm/IR/LegacyPassManager.h>
#include <llvm/MC/TargetRegistry.h>
#include <llvm/Target/TargetMachine.h>
#include <llvm/Target/TargetOptions.h>

using namespace llvm;

extern "C" const Target *llvm_target_lookup(const char *triple, nitro_string &err)
{
    std::string buf;
    auto target = TargetRegistry::lookupTarget(triple, buf);

    if (target) {
        return target;
    }

    nitro_string_set(err, buf.c_str());
    return nullptr;
}

extern "C" TargetMachine *llvm_target_create_machine(
    const Target *target,
    const char *triple,
    const char *cpu,
    const char *features)
{
    TargetOptions opts;
    return target->createTargetMachine(triple, cpu, features, opts, std::nullopt);
}

extern "C" void llvm_target_dispose_machine(TargetMachine *mc)
{
    delete mc;
}

extern "C" bool llvm_target_emit_object(
    TargetMachine *mc,
    Module *md,
    const char *file,
    nitro_string &err)
{
    std::error_code code;
    raw_fd_ostream os(file, code);

    if (code) {
        nitro_string_set(err, code.message().c_str());
        return false;
    }

    llvm::legacy::PassManager pass;

    if (mc->addPassesToEmitFile(pass, os, nullptr, CodeGenFileType::CGFT_ObjectFile)) {
        nitro_string_set(err, "The machine can't emit an object file");
        return false;
    }

    pass.run(*md);
    os.flush();

    return true;
}
