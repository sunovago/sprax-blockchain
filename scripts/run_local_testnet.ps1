# SPRX Local 3-Node Testnet Orchestration Script (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  SPRX PROTOCOL: LOCAL 3-NODE TESTNET INITIALIZATION" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

# Clean up previous testnet data
if (Test-Path ".testnet") {
    Remove-Item -Recurse -Force ".testnet"
}
New-Item -ItemType Directory -Path ".testnet/node1", ".testnet/node2", ".testnet/node3" -Force | Out-Null

Write-Host "[1/4] Compiling SPRX CLI Binary..." -ForegroundColor Yellow
cargo build --release -p sprax-cli
$SPRAX_BIN = ".\target\release\sprax.exe"

Write-Host "[2/4] Initializing Node 1, Node 2, Node 3 Home Directories..." -ForegroundColor Yellow
& $SPRAX_BIN init --chain-id sprax-devnet-1 --home .testnet/node1 | Out-Null
& $SPRAX_BIN init --chain-id sprax-devnet-1 --home .testnet/node2 | Out-Null
& $SPRAX_BIN init --chain-id sprax-devnet-1 --home .testnet/node3 | Out-Null

Copy-Item ".testnet/node1/genesis.json" ".testnet/node2/genesis.json" -Force
Copy-Item ".testnet/node1/genesis.json" ".testnet/node3/genesis.json" -Force

Write-Host "[3/4] Launching 3-Node Testnet Processes..." -ForegroundColor Yellow

$proc1 = Start-Process -FilePath $SPRAX_BIN -ArgumentList "start --home .testnet/node1 --p2p-port 26656 --rpc-port 26657" -PassThru -NoNewWindow
Start-Sleep -Seconds 2

$proc2 = Start-Process -FilePath $SPRAX_BIN -ArgumentList "start --home .testnet/node2 --p2p-port 26666 --rpc-port 26667 --peers 127.0.0.1:26656" -PassThru -NoNewWindow
Start-Sleep -Seconds 2

$proc3 = Start-Process -FilePath $SPRAX_BIN -ArgumentList "start --home .testnet/node3 --p2p-port 26676 --rpc-port 26677 --peers 127.0.0.1:26656,127.0.0.1:26666" -PassThru -NoNewWindow

Write-Host "------------------------------------------------------------" -ForegroundColor Green
Write-Host "  Node 1 PID: $($proc1.Id) (P2P: 26656, RPC: 26657)" -ForegroundColor Green
Write-Host "  Node 2 PID: $($proc2.Id) (P2P: 26666, RPC: 26667)" -ForegroundColor Green
Write-Host "  Node 3 PID: $($proc3.Id) (P2P: 26676, RPC: 26677)" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  To terminate, run: Stop-Process -Id $($proc1.Id), $($proc2.Id), $($proc3.Id)" -ForegroundColor Cyan
