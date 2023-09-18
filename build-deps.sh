#!/bin/sh -e
dir=$(pwd)

# LLVM
cmake \
  --install-prefix "$dir/lib/llvm" \
  -B "$dir/deps/llvm/build" \
  -Wno-dev \
  -DCMAKE_BUILD_TYPE:STRING=Release \
  -DLLVM_ENABLE_ZSTD:BOOL=OFF \
  -DLLVM_APPEND_VC_REV:BOOL=OFF \
  -DLLVM_TARGETS_TO_BUILD:STRING="AArch64;X86" \
  -DLLVM_ENABLE_PROJECTS:STRING="lld" \
  "$dir/deps/llvm/llvm"

cmake --build "$dir/deps/llvm/build" --config Release
cmake --install "$dir/deps/llvm/build" --config Release
