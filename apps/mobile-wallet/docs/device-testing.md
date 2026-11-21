# Android device test build

The device-test APK uses the local chain RPC at `http://10.111.250.173:26657`.
Both the phone and this PC must be on the same Wi-Fi network, and the PC must
remain awake with the node running. This is a release-mode test build signed
with the existing Android debug signing key.

## Install and check

1. Transfer `build/app/outputs/flutter-apk/app-release.apk` to the phone and install it.
2. Open `http://10.111.250.173:26657/status` in the phone browser. It should return
   JSON with `chain_id: sprax-devnet-1` and the current block height.
3. Open SPRX Wallet. A fresh wallet install shows the new onboarding. An existing
   wallet installation continues to its normal lock/home screen.
4. The default network is `Local-WiFi-Devnet`. For a previously selected public
   network, choose `Local-WiFi-Devnet` in Network Settings.

Wallet balances and chain requests connect directly to the local chain RPC;
this build does not start or configure the separate FastAPI service.

## Restart the node

From `D:\sprax-chain` (only if the node is not already running):

```powershell
.\target\debug\sprax.exe start --home .devnet/test_node
```

## Rebuild after the PC's Wi-Fi IP changes

From `D:\sprax-chain\apps\mobile-wallet`, replace the IP in these flags:

```powershell
flutter build apk --release --dart-define=SPRAX_DEVNET_NAME=Local-WiFi-Devnet --dart-define=SPRAX_DEVNET_RPC_URL=http://10.111.250.173:26657 --dart-define=SPRAX_DEVNET_EXPLORER_URL=http://10.111.250.173:3000
```

The explorer URL requires a separately running explorer on port 3000.
Without these flags, the original desktop localhost defaults remain intact.
