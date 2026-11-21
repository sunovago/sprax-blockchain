param(
    [string]$WasmOpt = "wasm-opt"
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot

if (-not (Get-Command $WasmOpt -ErrorAction SilentlyContinue)) {
    throw "Install Binaryen 123 and add its bin directory to PATH, or pass -WasmOpt C:\path\to\wasm-opt.exe. See docs/WASM_RUNTIME.md."
}

Push-Location $repoRoot
try {
    # Keep the output path stable even when the caller sets CARGO_TARGET_DIR.
    $targetDirectory = Join-Path $repoRoot "contracts/examples/counter/target"
    cargo build --manifest-path contracts/examples/counter/Cargo.toml `
        --target wasm32-unknown-unknown --target-dir $targetDirectory --release --locked
    if ($LASTEXITCODE -ne 0) {
        throw "Contract build failed. Ensure rustup target add wasm32-unknown-unknown has been run."
    }

    $fixture = Join-Path $targetDirectory "wasm32-unknown-unknown/release/sprax_counter.wasm"
    & $WasmOpt $fixture -Oz --llvm-memory-copy-fill-lowering `
        --strip-debug --strip-producers -o "${fixture}.optimized"
    if ($LASTEXITCODE -ne 0) {
        throw "WASM optimization failed."
    }
    Move-Item -LiteralPath "${fixture}.optimized" -Destination $fixture -Force
    Write-Output "Built WASM test fixture: $fixture"
} finally {
    Pop-Location
}
