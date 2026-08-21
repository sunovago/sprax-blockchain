# SPRX Mainnet Genesis Ceremony Runbook

## Pre-Ceremony Requirements
### Coordinator Checklist
- [ ] Genesis configuration parameters finalized and audited.
- [ ] Multi-sig setup and keys distributed to all 5 signatories.
- [ ] Communication channels established for genesis validators.

### Validator Checklist
- [ ] Dedicated, air-gapped machine ready for key generation.
- [ ] HSM or secure storage ready for private key retention.
- [ ] Verify SPRX CLI toolchain version (v1.0.0).

## Phase 1: Validator Key Generation
- Conducted on air-gapped machine.
- Run `sprax-cli keys add <moniker> --generate-only`.
- Extract the `operatorAddress` and `consensusPubkey`.
- Securely store the generated mnemonic and private key.
- Send the `consensusPubkey` and `operatorAddress` to the Coordinator.

## Phase 2: Genesis Assembly
- Coordinator collects all pubkeys and operator addresses from genesis validators.
- Replaces placeholders in `genesis.json` with actual values.
- Runs `verify_genesis.py` to ensure all parameters and supply match exactly.
- Calculates the final SHA-256 checksum of `genesis.json`.

## Phase 3: Multi-Sig Sign-off
- Coordinator distributes the finalized `genesis.json` checksum.
- 3 out of 5 signatories independently verify the `genesis.json` against the checksum.
- Signatories sign off on the genesis using their respective multi-sig keys.
- Signatures are collected and added to `genesis.json`.

## Phase 4: Distribution & Launch
- Final `genesis.json` published via IPFS and direct download links.
- Genesis validators initialize their nodes: `sprax-cli init --genesis <path_to_genesis.json>`.
- Validators configure seed nodes and persistent peers.
- Synchronized start across all validators at exactly `2026-09-01T00:00:00Z`.

## Phase 5: Post-Launch Verification
- Verify Block #1 is produced successfully.
- Verify `chainId` matches `sprax-mainnet-1`.
- Verify block explorers are indexing the chain correctly.
- Verify multi-sig addresses and initial balances match allocations.

## Emergency Abort Procedure
- If consensus fails or a critical parameter mismatch is found:
  - Coordinator triggers an emergency abort signal via established channels.
  - All validators immediately halt their nodes.
  - Ceremony resets to Phase 2 after resolving the issue.

## Sign-off Log
| Role | Name | Signature / Hash | Date |
| --- | --- | --- | --- |
| Coordinator | | | |
| Signatory 1 | | | |
| Signatory 2 | | | |
| Signatory 3 | | | |
| Signatory 4 | | | |
| Signatory 5 | | | |
