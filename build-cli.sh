#!/bin/sh -e
root=$(pwd)
type="$1"

if test x"$type" = x""; then
  type="Release"
fi

# ffi
cmake \
  --install-prefix "$root/lib/ffi" \
  -B "$root/ffi/build" \
  -DCMAKE_BUILD_TYPE:STRING="$type" \
  -DCMAKE_PREFIX_PATH:STRING="$root/lib/llvm" \
  "$root/ffi"

cmake --build "$root/ffi/build" --config "$type"
cmake --install "$root/ffi/build" --config "$type"

# cli
if test x"$type" = x"Release"; then
  cargo install --path stage0 --root dist
else
  cargo install --path stage0 --root dist --debug
fi
