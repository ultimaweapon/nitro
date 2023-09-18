# LLVM
cmake `
    --install-prefix "$PSScriptRoot/lib/llvm" `
    -B "$PSScriptRoot/deps/llvm/build" `
    -Wno-dev `
    -DCMAKE_BUILD_TYPE:STRING=Release `
    -DLLVM_ENABLE_ZSTD:BOOL=OFF `
    -DLLVM_APPEND_VC_REV:BOOL=OFF `
    -DLLVM_TARGETS_TO_BUILD:STRING="AArch64;X86" `
    -DLLVM_ENABLE_PROJECTS:STRING="lld" `
    "$PSScriptRoot/deps/llvm/llvm"

if ($LASTEXITCODE -ne 0) {
    exit 1
}

cmake --build "$PSScriptRoot/deps/llvm/build" --config Release

if ($LASTEXITCODE -ne 0) {
    exit 1
}

cmake --install "$PSScriptRoot/deps/llvm/build" --config Release

if ($LASTEXITCODE -ne 0) {
    exit 1
}
