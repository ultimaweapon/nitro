# LLVM
cmake `
    --install-prefix "$PSScriptRoot/lib/llvm" `
    -DCMAKE_BUILD_TYPE:STRING=Release `
    -DLLVM_ENABLE_LIBEDIT:BOOL=OFF `
    -DLLVM_ENABLE_LIBPFM:BOOL=OFF `
    -DLLVM_ENABLE_LIBXML2:BOOL=OFF `
    -DLLVM_ENABLE_TERMINFO:BOOL=OFF `
    -DLLVM_ENABLE_ZLIB:BOOL=OFF `
    -DLLVM_ENABLE_ZSTD:BOOL=OFF `
    -DLLVM_APPEND_VC_REV:BOOL=OFF `
    -DLLVM_TARGETS_TO_BUILD:STRING="AArch64;X86" `
    -DLLVM_ENABLE_PROJECTS:STRING="lld" `
    -Wno-dev `
    -B "$PSScriptRoot/deps/llvm/build" `
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

# zstd
cmake `
    --install-prefix "$PSScriptRoot/lib/zstd" `
    -DCMAKE_BUILD_TYPE:STRING=Release `
    -DZSTD_LEGACY_SUPPORT:BOOL=OFF `
    -DZSTD_MULTITHREAD_SUPPORT:BOOL=OFF `
    -DZSTD_BUILD_PROGRAMS:BOOL=OFF `
    -DZSTD_BUILD_SHARED:BOOL=OFF `
    -Wno-dev `
    -B "$PSScriptRoot/deps/zstd/build/cmake/build" `
    "$PSScriptRoot/deps/zstd/build/cmake"

if ($LASTEXITCODE -ne 0) {
    exit 1
}

cmake --build "$PSScriptRoot/deps/zstd/build/cmake/build" --config Release

if ($LASTEXITCODE -ne 0) {
    exit 1
}

cmake --install "$PSScriptRoot/deps/zstd/build/cmake/build" --config Release

if ($LASTEXITCODE -ne 0) {
    exit 1
}
