.\dist\bin\nitro.exe build --export dist/share/nitro std

if ($LASTEXITCODE -ne 0) {
    exit 1
}
