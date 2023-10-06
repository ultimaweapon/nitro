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
mkdir -p "$root/dist/bin"

if test x"$type" = x"Release"; then
  arg="--release"
  dir="release"
else
  dir="debug"
fi

cargo build --manifest-path "$root/stage0/Cargo.toml" $arg

if test x"$CMAKE_OSX_ARCHITECTURES" = x""; then
  cp "$root/stage0/target/$dir/nitro" "$root/dist/bin"
elif test x"$CMAKE_OSX_ARCHITECTURES" != x"x86_64;arm64"; then
  echo "The value of CMAKE_OSX_ARCHITECTURES environment variable must be x86_64;arm64." 1>&2
  exit 1
elif test x"$(uname -m)" != x"x86_64"; then
  echo "CMAKE_OSX_ARCHITECTURES environment variable is not supported on Apple Silicon." 1>&2
  exit 1
else
  cargo build --manifest-path "$root/stage0/Cargo.toml" $arg --target aarch64-apple-darwin
  lipo \
    -create \
    -output "$root/dist/bin/nitro" \
    "$root/stage0/target/$dir/nitro" \
    "$root/stage0/target/aarch64-apple-darwin/$dir/nitro"
fi
