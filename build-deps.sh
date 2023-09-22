#!/bin/sh -e
dir=$(pwd)

# LLVM
cmake \
  --install-prefix "$dir/lib/llvm" \
  -DCMAKE_BUILD_TYPE:STRING=Release \
  -DLLVM_ENABLE_LIBEDIT:BOOL=OFF \
  -DLLVM_ENABLE_LIBPFM:BOOL=OFF \
  -DLLVM_ENABLE_LIBXML2:BOOL=OFF \
  -DLLVM_ENABLE_TERMINFO:BOOL=OFF \
  -DLLVM_ENABLE_ZLIB:BOOL=OFF \
  -DLLVM_ENABLE_ZSTD:BOOL=OFF \
  -DLLVM_APPEND_VC_REV:BOOL=OFF \
  -DLLVM_TARGETS_TO_BUILD:STRING="AArch64;X86" \
  -DLLVM_ENABLE_PROJECTS:STRING="lld" \
  -Wno-dev \
  -B "$dir/deps/llvm/build" \
  "$dir/deps/llvm/llvm"

cmake --build "$dir/deps/llvm/build" --config Release
cmake --install "$dir/deps/llvm/build" --config Release
