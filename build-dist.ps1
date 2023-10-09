.\dist\bin\nitro.exe build --pkg dist\share\nitro\nitro.npk std

if ($LASTEXITCODE -ne 0) {
    exit 1
}
