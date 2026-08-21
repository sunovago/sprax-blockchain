# SPRX Protocol: Bug Bounty & Responsible Disclosure Program
**Document Version:** 1.0.0  
**Target:** Security Researchers, Whitehat Hackers

---

## 1. Scope & Guidelines

The SPRX Core Contributors value the contributions of security researchers in identifying vulnerabilities before mainnet launch.

### 1.1 In-Scope Targets
- **Consensus & State Machine** (`sprax-consensus`, `sprax-core`): Consensus halting, double-signing without slashing, state divergence.
- **Smart Contract VM** (`sprax-wasm`): Reentrancy bypass, sandbox escape, unauthorized state tampering, gas calculation flaws.
- **P2P Networking** (`sprax-network`): Eclipse attacks, malicious packet memory corruption, denial of service.
- **Cryptography** (`sprax-crypto`): Signature forgery, private key leakage in HD derivation.

### 1.2 Out-of-Scope
- Denial of Service attacks requiring $> 50\%$ network bandwidth exhaustion.
- Social engineering / phishing against node operators.
- Attacks on third-party testnet faucet interfaces.

---

## 2. Severity Classification & Rewards

| Severity Tier | Description | Testnet Recognition |
| :--- | :--- | :--- |
| **Critical** | Consensus halt, remote code execution (RCE), fund draining | Hall of Fame + Priority Mainnet Genesis Allocation |
| **High** | Validator jailing bypass, smart contract state corruption | Hall of Fame + Mainnet Allocation |
| **Medium** | RPC crashing, improper error handling causing node restart | Public Recognition |
| **Low** | Non-exploitable logic edge cases, documentation errors | Contributor Badge |

---

## 3. Submission & Responsible Disclosure

- **Email**: `security@sprax.network`
- **Encryption**: Researchers may encrypt submissions using the SPRX Security Team PGP Key.
- **Disclosure Policy**: Please allow 90 days for investigation and patch deployment before publishing any details publicly.
