param(
    [string]$Config = "config/suica-core.toml"
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true
$cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
if (-not (Test-Path -LiteralPath $cargo)) {
    throw "cargo.exe was not found. Install Rust stable with rustup first."
}

if (-not (Get-Command gcc -ErrorAction SilentlyContinue)) {
    $wingetRoot = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages"
    $gcc = Get-ChildItem -LiteralPath $wingetRoot -Filter gcc.exe -Recurse -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -like "*WinLibs*\mingw64\bin\gcc.exe" } |
        Select-Object -First 1
    if ($gcc) {
        $env:Path = "$($gcc.DirectoryName);$env:Path"
    }
}

& $cargo fmt --all --check
& $cargo clippy --workspace --all-targets -- -D warnings
& $cargo test --workspace
& $cargo build --release --workspace

Write-Host "Suica Core verification completed."
