# SPRX Mainnet Validator Onboarding Guide

Welcome to the SPRX Mainnet! This guide provides instructions for setting up and running a validator node on the SPRX blockchain.

## Hardware Requirements

| Component | Minimum | Recommended (Production) |
| :--- | :--- | :--- |
| **CPU** | 4 Cores | 8+ Cores (e.g., AMD EPYC or Intel Xeon) |
| **RAM** | 16 GB | 32+ GB ECC RAM |
| **Storage** | 500 GB NVMe SSD | 1 TB+ NVMe SSD (High IOPS) |
| **Network** | 500 Mbps | 1 Gbps+ Dedicated Port |
| **OS** | Ubuntu 22.04 LTS | Ubuntu 22.04 LTS or Debian 12 |

## Software Requirements

- Rust Toolchain (latest stable)
- Docker & Docker Compose (v2)
- `sprax-cli` version 1.0.0
- Git, curl, jq

## Step-by-Step Setup Guide

### 1. Install sprax-cli
Download and install the SPRX CLI binary.
```bash
wget https://github.com/spraxnetwork/sprax/releases/download/v1.0.0/sprax-cli-linux-amd64
chmod +x sprax-cli-linux-amd64
sudo mv sprax-cli-linux-amd64 /usr/local/bin/sprax-cli
```

### 2. Generate Validator Keys (Air-Gapped)
Generate your validator keys on a secure, air-gapped machine to prevent compromise.
```bash
sprax-cli keys add validator-key
```
*Note: Securely back up your mnemonic phrase!*

### 3. Submit gentx to Genesis Coordinator
Create your genesis transaction (gentx) to bond your initial stake.
```bash
sprax-cli genesis gentx validator-key 1000000000usprx \
  --chain-id=sprax-mainnet-1 \
  --moniker="YourValidatorName" \
  --commission-rate="0.05" \
  --commission-max-rate="0.20" \
  --commission-max-change-rate="0.01"
```
Submit the resulting `.json` file to the SPRX mainnet repository via PR.

### 4. Initialize Node with genesis.json
Once the final genesis file is published, initialize your node and download the `genesis.json`.
```bash
sprax-cli init "YourValidatorName" --chain-id sprax-mainnet-1
curl -s https://raw.githubusercontent.com/spraxnetwork/mainnet/main/genesis.json > ~/.sprax/config/genesis.json
```

### 5. Configure Sentry Architecture
Never expose your validator directly to the internet. Deploy a sentry node architecture:
- Set up 1-2 public sentry nodes.
- Configure your validator node to only connect to your sentry nodes via `persistent_peers`.
- See the provided `docker-compose.mainnet.yml` for an architectural reference.

### 6. Start Validator
Start your validator node (preferably via a systemd service or Docker Compose).
```bash
docker-compose -f deploy/docker-compose.mainnet.yml up -d validator
```

### 7. Verify Participation
Check the network block explorer or query your node to ensure it is syncing and signing blocks.
```bash
sprax-cli status
```

## Commission and Self-Delegation Requirements

- **Minimum Self-Stake**: 1,000 SPRX (`1,000,000,000usprx`)
- **Maximum Commission**: 20%
- **Max Commission Change Rate**: 1% per day

## Slashing Risks and How to Avoid Them

Validators face slashing penalties for misbehavior:
- **Downtime**: Missing >50% of blocks in a rolling window results in a small slash and temporary jailing. Ensure high availability through sentry nodes and proactive monitoring.
- **Double-Signing (Equivocation)**: Signing two different blocks at the same height results in a severe slash and permanent tombstoning. **Never** run the same validator key on multiple nodes simultaneously. Use a Hardware Security Module (HSM) if possible.

## Security Best Practices

- **HSM Usage**: Use an HSM (e.g., YubiHSM) or a secure KMS for signing blocks in production.
- **Firewall Rules**: Strictly firewall your validator node. Only allow P2P connections (e.g., port 26656) from your specific sentry node IPs. Deny all other incoming traffic.
- **Key Backup**: Store your mnemonic phrase and `priv_validator_key.json` in multiple secure, offline locations.
- **Monitoring**: Implement Prometheus and Grafana alerts for node health, peer count, missed blocks, and disk space.

## Contact/Support

For support, reach out to the SPRX validator community:
- Discord: [SPRX Network - #validators](https://discord.gg/spraxnetwork)
- Forum: [forum.sprax.network](https://forum.sprax.network)
- Email: validators@sprax.network
