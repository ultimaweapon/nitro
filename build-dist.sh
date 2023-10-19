#!/bin/sh -e
dir="dist/share/nitro"

# linux stubs
mkdir -p "$dir/stub/linux-gnu-x86_64"

./lib/llvm/bin/llvm-ifs \
  --output-elf="$dir/stub/linux-gnu-x86_64/libc.so" \
  --arch=x86_64 --bitwidth=64 --endianness=little \
  stub/linux-gnu/libc.ifs

# darwin stubs
mkdir -p "$dir/stub/darwin"
cp stub/darwin/libSystem.tbd "$dir/stub/darwin"

# win32 stubs
mkdir -p "$dir/stub/win32-x86_64"
./lib/llvm/bin/llvm-lib \
  /def:stub/win32/msvcrt.def \
  /machine:x64 \
  /out:"$dir/stub/win32-x86_64/msvcrt.lib"

# std
./dist/bin/nitro pack --output "$dir/nitro.npk"
