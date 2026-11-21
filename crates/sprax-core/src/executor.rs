use crate::{error::CoreError, gas::GasConfig, state::StateAccessor};
use sprax_crypto::{Ed25519Keypair, Hasher, Secp256k1Keypair};
use sprax_storage::{KVStore, OverlayStore};
use sprax_types::{Address, KeyType, Transaction, TxMessage, TxReceipt};

/// Transaction Validation & Execution Engine.
#[derive(Debug, Clone, Default)]
pub struct TxExecutor {
    gas_config: GasConfig,
}

impl TxExecutor {
    #[must_use]
    pub fn new(gas_config: GasConfig) -> Self {
        Self { gas_config }
    }

    /// Pre-validates transaction statically without mutating state.
    pub fn validate_transaction_static(
        &self,
        tx: &Transaction,
        expected_chain_id: &str,
    ) -> Result<(), CoreError> {
        // 1. Verify Chain ID
        if tx.body.chain_id.as_str() != expected_chain_id {
            return Err(CoreError::ModuleError {
                module: "auth".into(),
                reason: format!(
                    "chain ID mismatch: expected '{expected_chain_id}', found '{}'",
                    tx.body.chain_id.as_str()
                ),
            });
        }

        // 2. Verify Signer Public Key maps to Sender Address
        let derived_addr_bytes = Hasher::blake3(&tx.public_key);
        let mut expected_addr_bytes = [0u8; 20];
        expected_addr_bytes.copy_from_slice(&derived_addr_bytes.as_bytes()[0..20]);
        let expected_sender = Address::new(expected_addr_bytes);

        if tx.body.sender != expected_sender {
            return Err(CoreError::ModuleError {
                module: "auth".into(),
                reason: format!(
                    "sender address '{}' does not match public key derived address '{expected_sender}'",
                    tx.body.sender
                ),
            });
        }

        // 3. Verify Cryptographic Signature (RFC 8785 Canonical with legacy serde fallback)
        let sign_bytes = tx
            .sign_bytes()
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        let legacy_bytes = serde_json::to_vec(&tx.body).ok();

        match tx.key_type {
            KeyType::Ed25519 => {
                if Ed25519Keypair::verify(&tx.public_key, &sign_bytes, &tx.signature).is_err() {
                    let mut verified = false;
                    if let Some(ref leg) = legacy_bytes {
                        if Ed25519Keypair::verify(&tx.public_key, leg, &tx.signature).is_ok() {
                            verified = true;
                        }
                    }
                    if !verified {
                        return Err(CoreError::ModuleError {
                            module: "crypto".into(),
                            reason: "ed25519 signature verification failed".into(),
                        });
                    }
                }
            }
            KeyType::Secp256k1 => {
                if Secp256k1Keypair::verify(&tx.public_key, &sign_bytes, &tx.signature).is_err() {
                    let mut verified = false;
                    if let Some(ref leg) = legacy_bytes {
                        if Secp256k1Keypair::verify(&tx.public_key, leg, &tx.signature).is_ok() {
                            verified = true;
                        }
                    }
                    if !verified {
                        return Err(CoreError::ModuleError {
                            module: "crypto".into(),
                            reason: "secp256k1 signature verification failed".into(),
                        });
                    }
                }
            }
        }

        // 4. Verify message invariants
        if tx.body.messages.is_empty() {
            return Err(CoreError::ModuleError {
                module: "auth".into(),
                reason: "transaction must contain messages".into(),
            });
        }
        for msg in &tx.body.messages {
            if let TxMessage::Transfer { amount, .. }
            | TxMessage::Delegate { amount, .. }
            | TxMessage::Unbond { amount, .. } = msg
            {
                if amount.is_zero() {
                    return Err(CoreError::ModuleError {
                        module: "bank".into(),
                        reason: "message amount must be greater than zero".into(),
                    });
                }
            }
            if let TxMessage::RegisterValidator {
                consensus_pubkey,
                moniker,
                self_stake,
                proof_of_possession,
            } = msg
            {
                if consensus_pubkey.len() != 32 {
                    return Err(CoreError::ModuleError {
                        module: "staking".into(),
                        reason: "consensus public key must be exactly 32 bytes".into(),
                    });
                }
                if moniker.is_empty() || moniker.len() > 64 {
                    return Err(CoreError::ModuleError {
                        module: "staking".into(),
                        reason: "validator moniker must be between 1 and 64 characters".into(),
                    });
                }
                if *self_stake < crate::validator_set::MIN_SELF_STAKE {
                    return Err(CoreError::ModuleError {
                        module: "staking".into(),
                        reason: format!(
                            "self stake {} is less than minimum required {}",
                            self_stake,
                            crate::validator_set::MIN_SELF_STAKE
                        ),
                    });
                }
                if proof_of_possession.len() != 64 {
                    return Err(CoreError::ModuleError {
                        module: "staking".into(),
                        reason: "proof of possession signature must be exactly 64 bytes".into(),
                    });
                }
            }
            if let TxMessage::SubmitEquivocationEvidence { evidence } = msg {
                if !evidence.is_valid_equivocation() {
                    return Err(CoreError::ModuleError {
                        module: "slashing".into(),
                        reason: "invalid equivocation metadata".into(),
                    });
                }
            }
            if matches!(msg, TxMessage::Generic { .. }) {
                return Err(CoreError::ModuleError {
                    module: "execution".into(),
                    reason: "unsupported message type".into(),
                });
            }
        }

        Ok(())
    }

    /// Executes a transaction against state store, mutating balances and nonces atomically.
    pub fn execute_transaction<S: KVStore + Clone + 'static>(
        &self,
        store: &S,
        tx: &Transaction,
        current_height: u64,
        expected_chain_id: &str,
        unbonding_period_blocks: u64,
    ) -> Result<TxReceipt, CoreError> {
        self.execute_transaction_at_time(
            store,
            tx,
            current_height,
            1_700_000_000 + current_height * 2,
            expected_chain_id,
            unbonding_period_blocks,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn execute_transaction_at_time<S: KVStore + Clone + 'static>(
        &self,
        store: &S,
        tx: &Transaction,
        current_height: u64,
        timestamp: u64,
        expected_chain_id: &str,
        unbonding_period_blocks: u64,
    ) -> Result<TxReceipt, CoreError> {
        let overlay = OverlayStore::new(store.clone());
        let receipt = self.execute_transaction_inner(
            &overlay,
            tx,
            current_height,
            timestamp,
            expected_chain_id,
            unbonding_period_blocks,
        )?;
        overlay
            .commit_state()
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        Ok(receipt)
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_transaction_inner<S: KVStore + Clone + 'static>(
        &self,
        store: &S,
        tx: &Transaction,
        current_height: u64,
        timestamp: u64,
        expected_chain_id: &str,
        unbonding_period_blocks: u64,
    ) -> Result<TxReceipt, CoreError> {
        // 1. Static Validation
        self.validate_transaction_static(tx, expected_chain_id)?;
        if tx.body.timeout_height != 0 && current_height > tx.body.timeout_height {
            return Err(CoreError::ModuleError {
                module: "auth".into(),
                reason: "transaction expired".into(),
            });
        }
        let mut gas = crate::gas::GasMeter::new(tx.body.fee.gas_limit);
        gas.consume_gas(self.gas_config.base_tx_cost)?;
        gas.consume_gas(self.gas_config.signature_verify_cost)?;
        for msg in &tx.body.messages {
            match msg {
                TxMessage::RegisterValidator { .. } => {
                    gas.consume_gas(50_000)?;
                }
                TxMessage::Unjail => {
                    gas.consume_gas(20_000)?;
                }
                TxMessage::SubmitEquivocationEvidence { .. } => {
                    gas.consume_gas(150_000)?;
                }
                _ => {
                    gas.consume_gas(self.gas_config.transfer_cost)?;
                }
            }
        }

        // 2. Fetch Sender Account State
        let mut sender_state = StateAccessor::get_account(store, &tx.body.sender)?;

        // 3. Nonce Verification (Replay Protection)
        if tx.body.nonce != sender_state.nonce {
            return Err(CoreError::InvalidNonce {
                account_nonce: sender_state.nonce,
                tx_nonce: tx.body.nonce,
            });
        }

        // 4. Calculate total cost for all messages + fee. Delegate & RegisterValidator reserve funds
        // out of the sender's spendable balance; Unbond does not (it returns already-delegated funds).
        let mut total_transfer_cost = sprax_types::Amount::ZERO;
        for msg in &tx.body.messages {
            if let TxMessage::Transfer { amount, .. }
            | TxMessage::Delegate { amount, .. }
            | TxMessage::InstantiateContract { funds: amount, .. }
            | TxMessage::ContractCall { funds: amount, .. }
            | TxMessage::RegisterValidator {
                self_stake: amount, ..
            } = msg
            {
                total_transfer_cost = total_transfer_cost
                    .checked_add(*amount)
                    .map_err(|e| CoreError::StateError(e.to_string()))?;
            }
        }

        let total_required = total_transfer_cost
            .checked_add(tx.body.fee.amount)
            .map_err(|e| CoreError::StateError(e.to_string()))?;

        if sender_state.balance < total_required {
            return Err(CoreError::InsufficientFunds {
                balance: sender_state.balance.to_string(),
                required: total_required.to_string(),
            });
        }

        // 4b. Pre-validate state-dependent constraints before any mutation begins, so a
        // rejected message never leaves a partially-applied transaction behind.
        for msg in &tx.body.messages {
            match msg {
                TxMessage::Unbond { validator, amount } => {
                    let delegation =
                        StateAccessor::get_delegation(store, &tx.body.sender, validator)?;
                    if delegation.balance < *amount {
                        return Err(CoreError::InsufficientFunds {
                            balance: delegation.balance.to_string(),
                            required: amount.to_string(),
                        });
                    }
                }
                TxMessage::RegisterValidator {
                    consensus_pubkey,
                    proof_of_possession,
                    ..
                } => {
                    if StateAccessor::get_validator_record(store, &tx.body.sender)?.is_some() {
                        return Err(CoreError::ModuleError {
                            module: "staking".into(),
                            reason: "validator already registered".into(),
                        });
                    }
                    if StateAccessor::get_operator_by_consensus_key(store, consensus_pubkey)?
                        .is_some()
                    {
                        return Err(CoreError::ModuleError {
                            module: "staking".into(),
                            reason: "consensus public key is already registered".into(),
                        });
                    }
                    let pop_msg = [
                        b"sprax/pop/v1".as_slice(),
                        expected_chain_id.as_bytes(),
                        tx.body.sender.as_bytes(),
                        consensus_pubkey.as_slice(),
                    ]
                    .concat();
                    Ed25519Keypair::verify(consensus_pubkey, &pop_msg, proof_of_possession)
                        .map_err(|e| CoreError::ModuleError {
                            module: "staking".into(),
                            reason: format!("invalid proof of possession: {e}"),
                        })?;
                }
                TxMessage::Unjail => {
                    let record = StateAccessor::get_validator_record(store, &tx.body.sender)?
                        .ok_or_else(|| CoreError::ModuleError {
                            module: "staking".into(),
                            reason: "validator not found".into(),
                        })?;
                    if record.is_tombstoned {
                        return Err(CoreError::ModuleError {
                            module: "staking".into(),
                            reason: "tombstoned validator cannot be unjailed".into(),
                        });
                    }
                    if record.status != crate::state::CanonicalValidatorStatus::Jailed {
                        return Err(CoreError::ModuleError {
                            module: "staking".into(),
                            reason: "validator is not jailed".into(),
                        });
                    }
                    if let Some(until) = record.jailed_until {
                        if current_height < until {
                            return Err(CoreError::ModuleError {
                                module: "staking".into(),
                                reason: format!(
                                    "jail cooldown active until height {until} (current {current_height})"
                                ),
                            });
                        }
                    }
                }
                TxMessage::SubmitEquivocationEvidence { evidence } => {
                    if !evidence.is_valid_equivocation() {
                        return Err(CoreError::ModuleError {
                            module: "slashing".into(),
                            reason: "invalid equivocation evidence".into(),
                        });
                    }
                    if evidence.height > current_height {
                        return Err(CoreError::ModuleError {
                            module: "slashing".into(),
                            reason: "evidence height cannot be in the future".into(),
                        });
                    }
                    if current_height.saturating_sub(evidence.height) > unbonding_period_blocks {
                        return Err(CoreError::ModuleError {
                            module: "slashing".into(),
                            reason: "evidence is older than unbonding period".into(),
                        });
                    }
                    let evidence_hash = evidence.canonical_hash();
                    if StateAccessor::has_evidence(store, &evidence_hash)? {
                        return Err(CoreError::ModuleError {
                            module: "slashing".into(),
                            reason: "equivocation evidence has already been applied".into(),
                        });
                    }
                    let val_record =
                        StateAccessor::get_validator_record(store, &evidence.validator_address)?
                            .ok_or_else(|| CoreError::ModuleError {
                                module: "slashing".into(),
                                reason: "evidence validator not found in canonical registry".into(),
                            })?;
                    for vote in [&evidence.vote_a, &evidence.vote_b] {
                        let sign_bytes = vote
                            .sign_bytes()
                            .map_err(|e| CoreError::StateError(e.to_string()))?;
                        Ed25519Keypair::verify(
                            &val_record.consensus_pubkey,
                            &sign_bytes,
                            &vote.signature,
                        )
                        .map_err(|e| CoreError::ModuleError {
                            module: "slashing".into(),
                            reason: format!("forged evidence signature: {e}"),
                        })?;
                    }
                }
                _ => {}
            }
        }

        // 5. Apply state mutations
        // Deduct transfer amount and fee from sender
        sender_state.balance = sender_state
            .balance
            .checked_sub(total_required)
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        sender_state.nonce = sender_state
            .nonce
            .checked_add(1)
            .ok_or_else(|| CoreError::StateError("account nonce exhausted".into()))?;

        StateAccessor::set_account(store, &tx.body.sender, &sender_state)?;

        // Account for the burned fee: it left the sender's balance above and is never
        // credited anywhere, so circulating supply shrinks by the same amount it tracks as burned.
        if !tx.body.fee.amount.is_zero() {
            let mut supply = StateAccessor::get_supply_state(store)?;
            supply.circulating_supply = supply
                .circulating_supply
                .checked_sub(tx.body.fee.amount)
                .map_err(|e| CoreError::StateError(e.to_string()))?;
            supply.total_burned = supply
                .total_burned
                .checked_add(tx.body.fee.amount)
                .map_err(|e| CoreError::StateError(e.to_string()))?;
            StateAccessor::set_supply_state(store, &supply)?;
        }

        // Apply message credits
        let mut return_data = Vec::new();
        let mut collected_events = Vec::new();
        for (message_index, msg) in tx.body.messages.iter().enumerate() {
            let context = sprax_wasm::ContractContext {
                chain_id: expected_chain_id.into(),
                height: current_height,
                timestamp,
                sender: tx.body.sender,
                nonce: tx.body.nonce,
                message_index: u32::try_from(message_index)
                    .map_err(|_| CoreError::StateError("too many transaction messages".into()))?,
            };
            match msg {
                TxMessage::Transfer { to, amount } => {
                    let mut recipient_state = StateAccessor::get_account(store, to)?;
                    recipient_state.balance = recipient_state
                        .balance
                        .checked_add(*amount)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    StateAccessor::set_account(store, to, &recipient_state)?;
                }
                TxMessage::Delegate { validator, amount } => {
                    // Funds already reserved out of the sender's balance above (step 4); credit
                    // them into the canonical validator/delegation ledger.
                    let mut validator_stake = StateAccessor::get_validator_stake(store, validator)?;
                    validator_stake.tokens = validator_stake
                        .tokens
                        .checked_add(*amount)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    StateAccessor::set_validator_stake(store, validator, &validator_stake)?;

                    let mut delegation =
                        StateAccessor::get_delegation(store, &tx.body.sender, validator)?;
                    delegation.shares = delegation
                        .shares
                        .checked_add(*amount)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    delegation.balance = delegation
                        .balance
                        .checked_add(*amount)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    StateAccessor::set_delegation(store, &tx.body.sender, validator, &delegation)?;
                }
                TxMessage::Unbond { validator, amount } => {
                    // Balance sufficiency already verified in step 4b.
                    let mut delegation =
                        StateAccessor::get_delegation(store, &tx.body.sender, validator)?;
                    delegation.balance = delegation
                        .balance
                        .checked_sub(*amount)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    delegation.shares = delegation
                        .shares
                        .checked_sub(*amount)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    StateAccessor::set_delegation(store, &tx.body.sender, validator, &delegation)?;

                    let mut validator_stake = StateAccessor::get_validator_stake(store, validator)?;
                    validator_stake.tokens = validator_stake
                        .tokens
                        .checked_sub(*amount)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    StateAccessor::set_validator_stake(store, validator, &validator_stake)?;

                    let completion_height = current_height.saturating_add(unbonding_period_blocks);
                    let entry = crate::state::UnbondingRecord {
                        delegator: tx.body.sender,
                        validator: *validator,
                        completion_height,
                        amount: *amount,
                    };
                    StateAccessor::set_unbonding(store, &entry, tx.body.nonce)?;
                }
                TxMessage::StoreCode { wasm_bytecode } => {
                    let (id, consumed) = sprax_wasm::CosmWasmRuntime
                        .store_code(store, wasm_bytecode, gas.remaining())
                        .map_err(CoreError::ExecutionReverted)?;
                    gas.consume_gas(consumed)?;
                    return_data = serde_json::to_vec(&id)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                }
                TxMessage::InstantiateContract {
                    code_id,
                    msg,
                    funds,
                    label,
                    admin,
                } => {
                    let address = sprax_wasm::CosmWasmRuntime::contract_address(&context, *code_id);
                    let mut account = StateAccessor::get_account(store, &address)?;
                    account.balance = account
                        .balance
                        .checked_add(*funds)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    account.code_hash = *code_id;
                    StateAccessor::set_account(store, &address, &account)?;
                    let (address, result) = sprax_wasm::CosmWasmRuntime
                        .instantiate(
                            store,
                            *code_id,
                            &context,
                            msg,
                            label,
                            *admin,
                            *funds,
                            gas.remaining(),
                        )
                        .map_err(CoreError::ExecutionReverted)?;
                    gas.consume_gas(result.gas_used)?;
                    collected_events.extend(result.events);
                    return_data = serde_json::to_vec(&address)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                }
                TxMessage::ContractCall {
                    contract,
                    data,
                    funds,
                } => {
                    let mut account = StateAccessor::get_account(store, contract)?;
                    account.balance = account
                        .balance
                        .checked_add(*funds)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    StateAccessor::set_account(store, contract, &account)?;
                    let result = sprax_wasm::CosmWasmRuntime
                        .execute(store, *contract, &context, data, *funds, gas.remaining())
                        .map_err(CoreError::ExecutionReverted)?;
                    gas.consume_gas(result.gas_used)?;
                    collected_events.extend(result.events);
                    return_data = result.data;
                }
                TxMessage::MigrateContract {
                    contract,
                    new_code_id,
                    msg,
                } => {
                    let result = sprax_wasm::CosmWasmRuntime
                        .migrate(
                            store,
                            *contract,
                            *new_code_id,
                            &context,
                            msg,
                            gas.remaining(),
                        )
                        .map_err(CoreError::ExecutionReverted)?;
                    gas.consume_gas(result.gas_used)?;
                    collected_events.extend(result.events);
                    return_data = result.data;
                }
                TxMessage::UpdateAdmin {
                    contract,
                    new_admin,
                } => {
                    sprax_wasm::CosmWasmRuntime
                        .update_admin(store, *contract, &context, *new_admin)
                        .map_err(CoreError::ExecutionReverted)?;
                    gas.consume_gas(10_000)?;
                }

                TxMessage::RegisterValidator {
                    consensus_pubkey,
                    moniker,
                    self_stake,
                    proof_of_possession: _,
                } => {
                    let mut validator_stake =
                        StateAccessor::get_validator_stake(store, &tx.body.sender)?;
                    validator_stake.tokens = validator_stake
                        .tokens
                        .checked_add(*self_stake)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    StateAccessor::set_validator_stake(store, &tx.body.sender, &validator_stake)?;

                    let mut delegation =
                        StateAccessor::get_delegation(store, &tx.body.sender, &tx.body.sender)?;
                    delegation.shares = delegation
                        .shares
                        .checked_add(*self_stake)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    delegation.balance = delegation
                        .balance
                        .checked_add(*self_stake)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    StateAccessor::set_delegation(
                        store,
                        &tx.body.sender,
                        &tx.body.sender,
                        &delegation,
                    )?;

                    let record = crate::state::ValidatorRecordState {
                        operator_address: tx.body.sender,
                        consensus_pubkey: consensus_pubkey.clone(),
                        moniker: moniker.clone(),
                        status: crate::state::CanonicalValidatorStatus::Active,
                        jailed_until: None,
                        is_tombstoned: false,
                        registered_height: current_height,
                    };
                    StateAccessor::set_validator_record(store, &record)?;
                    StateAccessor::set_consensus_key_index(
                        store,
                        consensus_pubkey,
                        &tx.body.sender,
                    )?;
                }
                TxMessage::Unjail => {
                    let mut record = StateAccessor::get_validator_record(store, &tx.body.sender)?
                        .expect("validated in step 4b");
                    record.status = crate::state::CanonicalValidatorStatus::Active;
                    record.jailed_until = None;
                    StateAccessor::set_validator_record(store, &record)?;
                }
                TxMessage::SubmitEquivocationEvidence { evidence } => {
                    let evidence_hash = evidence.canonical_hash();
                    let mut record =
                        StateAccessor::get_validator_record(store, &evidence.validator_address)?
                            .expect("validated in step 4b");

                    if !record.is_tombstoned {
                        // 1. Slash bonded tokens (500 bps = 5.00%)
                        let mut val_stake =
                            StateAccessor::get_validator_stake(store, &evidence.validator_address)?;
                        let slashed_bonded_atto =
                            val_stake.tokens.as_atto().checked_mul(500).unwrap_or(0) / 10_000;
                        let slashed_bonded = sprax_types::Amount::from_atto(slashed_bonded_atto);
                        val_stake.tokens = val_stake
                            .tokens
                            .checked_sub(slashed_bonded)
                            .unwrap_or(sprax_types::Amount::ZERO);
                        StateAccessor::set_validator_stake(
                            store,
                            &evidence.validator_address,
                            &val_stake,
                        )?;

                        let mut total_slashed = slashed_bonded;

                        // 2. Slash pending unbonding entries for this validator created within slashable window
                        let unbondings = StateAccessor::scan_unbondings_for_validator(
                            store,
                            &evidence.validator_address,
                        )?;
                        for (key, mut unbond) in unbondings {
                            let slashed_unbond_atto =
                                unbond.amount.as_atto().checked_mul(500).unwrap_or(0) / 10_000;
                            let slashed_unbond =
                                sprax_types::Amount::from_atto(slashed_unbond_atto);
                            unbond.amount = unbond
                                .amount
                                .checked_sub(slashed_unbond)
                                .unwrap_or(sprax_types::Amount::ZERO);
                            let bytes = serde_json::to_vec(&unbond).map_err(|e| {
                                CoreError::StateError(format!("serialization error: {e}"))
                            })?;
                            store
                                .set(&key, &bytes)
                                .map_err(|e| CoreError::StateError(e.to_string()))?;
                            total_slashed = total_slashed
                                .checked_add(slashed_unbond)
                                .unwrap_or(total_slashed);
                        }

                        // 3. Burn total slashed tokens from circulating supply
                        if !total_slashed.is_zero() {
                            let mut supply = StateAccessor::get_supply_state(store)?;
                            supply.circulating_supply = supply
                                .circulating_supply
                                .checked_sub(total_slashed)
                                .unwrap_or(sprax_types::Amount::ZERO);
                            supply.total_burned = supply
                                .total_burned
                                .checked_add(total_slashed)
                                .map_err(|e| CoreError::StateError(e.to_string()))?;
                            StateAccessor::set_supply_state(store, &supply)?;
                        }

                        // 4. Mark validator as Jailed and permanently Tombstoned
                        record.status = crate::state::CanonicalValidatorStatus::Jailed;
                        record.is_tombstoned = true;
                        StateAccessor::set_validator_record(store, &record)?;
                    }

                    // 5. Record evidence processed marker
                    StateAccessor::record_evidence(store, &evidence_hash, current_height)?;
                }
                TxMessage::Generic { .. } => {
                    return Err(CoreError::ModuleError {
                        module: "execution".into(),
                        reason: "unsupported message type".into(),
                    })
                }
            }
        }

        let tx_hash = Hasher::tx_hash(tx).map_err(|e| CoreError::StateError(e.to_string()))?;

        Ok(TxReceipt {
            tx_hash,
            height: current_height,
            success: true,
            gas_used: gas.consumed(),
            error_message: None,
            return_data,
            events: collected_events,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AccountState;
    use sprax_crypto::Ed25519Keypair;
    use sprax_storage::MemKVStore;
    use sprax_types::{Amount, ChainId, TxBody, TxFee};

    #[test]
    fn test_valid_transaction_execution() {
        let store = MemKVStore::new();
        let alice_kp = Ed25519Keypair::generate();
        let bob_kp = Ed25519Keypair::generate();
        let alice_addr = alice_kp.address();
        let bob_addr = bob_kp.address();

        // Fund Alice with 1000 SPRX
        let alice_init = AccountState {
            nonce: 0,
            balance: Amount::from_sprx_whole(1000).unwrap(),
            code_hash: sprax_types::Hash32::ZERO,
            storage_root: sprax_types::Hash32::ZERO,
        };
        StateAccessor::set_account(&store, &alice_addr, &alice_init).unwrap();
        StateAccessor::set_supply_state(
            &store,
            &crate::state::SupplyState {
                circulating_supply: alice_init.balance,
                total_burned: Amount::ZERO,
            },
        )
        .unwrap();

        let transfer_amount = Amount::from_sprx_whole(150).unwrap();
        let fee = TxFee::default();

        let tx_body = TxBody {
            chain_id: ChainId::new("sprax-devnet-1").unwrap(),
            sender: alice_addr,
            nonce: 0,
            messages: vec![TxMessage::Transfer {
                to: bob_addr,
                amount: transfer_amount,
            }],
            fee: fee.clone(),
            memo: "test transfer".into(),
            timeout_height: 100,
        };

        let sign_bytes = tx_body.sign_bytes().unwrap();
        let sig = alice_kp.sign(&sign_bytes);

        let tx = Transaction::new(
            tx_body,
            KeyType::Ed25519,
            alice_kp.public_key_bytes().to_vec(),
            sig,
        )
        .unwrap();

        let executor = TxExecutor::default();
        let receipt = executor
            .execute_transaction(&store, &tx, 1, "sprax-devnet-1", 10)
            .unwrap();

        assert!(receipt.success);
        assert_eq!(receipt.height, 1);

        let alice_after = StateAccessor::get_account(&store, &alice_addr).unwrap();
        let bob_after = StateAccessor::get_account(&store, &bob_addr).unwrap();

        assert_eq!(alice_after.nonce, 1);
        assert_eq!(bob_after.balance, transfer_amount);
        let expected_alice_bal = Amount::from_sprx_whole(1000)
            .unwrap()
            .checked_sub(transfer_amount)
            .unwrap()
            .checked_sub(fee.amount)
            .unwrap();
        assert_eq!(alice_after.balance, expected_alice_bal);

        let supply = StateAccessor::get_supply_state(&store).unwrap();
        assert_eq!(supply.total_burned, fee.amount);
    }

    #[test]
    fn test_fee_burn_updates_supply_state() {
        let store = MemKVStore::new();
        let alice_kp = Ed25519Keypair::generate();
        let bob_kp = Ed25519Keypair::generate();
        let alice_addr = alice_kp.address();
        let bob_addr = bob_kp.address();

        let genesis_supply = Amount::from_sprx_whole(1000).unwrap();
        StateAccessor::set_account(
            &store,
            &alice_addr,
            &AccountState {
                nonce: 0,
                balance: genesis_supply,
                code_hash: sprax_types::Hash32::ZERO,
                storage_root: sprax_types::Hash32::ZERO,
            },
        )
        .unwrap();
        StateAccessor::set_supply_state(
            &store,
            &crate::state::SupplyState {
                circulating_supply: genesis_supply,
                total_burned: Amount::ZERO,
            },
        )
        .unwrap();

        let fee = TxFee::default();
        let tx_body = TxBody {
            chain_id: ChainId::new("sprax-devnet-1").unwrap(),
            sender: alice_addr,
            nonce: 0,
            messages: vec![TxMessage::Transfer {
                to: bob_addr,
                amount: Amount::from_sprx_whole(10).unwrap(),
            }],
            fee: fee.clone(),
            memo: "test burn".into(),
            timeout_height: 100,
        };
        let sign_bytes = tx_body.sign_bytes().unwrap();
        let sig = alice_kp.sign(&sign_bytes);
        let tx = Transaction::new(
            tx_body,
            KeyType::Ed25519,
            alice_kp.public_key_bytes().to_vec(),
            sig,
        )
        .unwrap();

        let executor = TxExecutor::default();
        executor
            .execute_transaction(&store, &tx, 1, "sprax-devnet-1", 10)
            .unwrap();

        let supply = StateAccessor::get_supply_state(&store).unwrap();
        assert_eq!(supply.total_burned, fee.amount);
        assert_eq!(
            supply.circulating_supply,
            genesis_supply.checked_sub(fee.amount).unwrap()
        );
    }
}
