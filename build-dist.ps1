$root = "$PSScriptRoot\dist\share\nitro"

# linux stubs
New-Item -ItemType Directory -Path "$root\stub\linux-gnu-x86_64" -Force

.\lib\llvm\bin\llvm-ifs.exe `
    --output-elf="$root\stub\linux-gnu-x86_64\libc.so" `
    --arch=x86_64 --bitwidth=64 --endianness=little `
    "$PSScriptRoot\stub\linux-gnu\libc.ifs"

if ($LASTEXITCODE -ne 0) {
    exit 1
}

# darwin stubs
New-Item -ItemType Directory -Path "$root\stub\darwin" -Force
Copy-Item "$PSScriptRoot\stub\darwin\libSystem.tbd" -Destination "$root\stub\darwin"

# win32 stubs
New-Item -ItemType Directory -Path "$root\stub\win32-x86_64" -Force
./lib/llvm/bin/llvm-lib.exe `
    /def:"$PSScriptRoot\stub\win32\msvcrt.def" `
    /machine:x64 `
    /out:"$root\stub\win32-x86_64\msvcrt.lib"

if ($LASTEXITCODE -ne 0) {
    exit 1
}

# std
.\dist\bin\nitro.exe build --pkg dist\share\nitro\nitro.npk

if ($LASTEXITCODE -ne 0) {
    exit 1
}
