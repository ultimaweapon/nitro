param([string]$type="Release")

# ffi
cmake `
    --install-prefix "$PSScriptRoot/lib/ffi" `
    -B "$PSScriptRoot/ffi/build" `
    -DCMAKE_BUILD_TYPE:STRING="$type" `
    -DCMAKE_PREFIX_PATH:STRING="$PSScriptRoot/lib/llvm" `
    "$PSScriptRoot/ffi"

if ($LASTEXITCODE -ne 0) {
    exit 1
}

cmake --build "$PSScriptRoot/ffi/build" --config "$type"

if ($LASTEXITCODE -ne 0) {
    exit 1
}

cmake --install "$PSScriptRoot/ffi/build" --config "$type"

if ($LASTEXITCODE -ne 0) {
    exit 1
}

# cli
if ($type -eq "Release") {
    cargo install --path stage0 --root dist
} else {
    cargo install --path stage0 --root dist --debug
}
