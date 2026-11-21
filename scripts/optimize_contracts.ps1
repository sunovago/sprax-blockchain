param(
    [string]$WasmOpt = "D:\sprax-chain\.tools\binaryen-version_123\bin\wasm-opt.exe"
)

$ErrorActionPreference = "Stop"

$contracts = @(
    "contracts/examples/cw20-token/target/wasm32-unknown-unknown/release/sprax_cw20_token.wasm",
    "contracts/examples/escrow/target/wasm32-unknown-unknown/release/sprax_escrow.wasm"
)

foreach ($relPath in $contracts) {
    $fullPath = Join-Path "D:\sprax-chain" $relPath
    if (Test-Path $fullPath) {
        $optPath = "$fullPath.optimized"
        Write-Output "Optimizing $fullPath..."
        & $WasmOpt $fullPath -Oz --llvm-memory-copy-fill-lowering --strip-debug --strip-producers -o $optPath
        Move-Item -LiteralPath $optPath -Destination $fullPath -Force
        $size = (Get-Item $fullPath).Length
        Write-Output "Optimized $fullPath (size: $size bytes)"
    } else {
        Write-Warning "File not found: $fullPath"
    }
}
