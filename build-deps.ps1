# LLVM
cmake `
    --install-prefix "$PSScriptRoot/lib/llvm" `
    -B "$PSScriptRoot/deps/llvm/build" `
    -Wno-dev `
    -DCMAKE_BUILD_TYPE:STRING=Release `
    -DLLVM_ENABLE_ZSTD:BOOL=OFF `
    -DLLVM_APPEND_VC_REV:BOOL=OFF `
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
