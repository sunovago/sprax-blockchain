use crate::{error::CoreError, gas::GasConfig, state::StateAccessor};
use sprax_crypto::{Ed25519Keypair, Hasher, Secp256k1Keypair};
use sprax_storage::KVStore;
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

        // 3. Verify Cryptographic Signature
        let sign_bytes = tx
            .sign_bytes()
            .map_err(|e| CoreError::StateError(e.to_string()))?;

        match tx.key_type {
            KeyType::Ed25519 => {
                Ed25519Keypair::verify(&tx.public_key, &sign_bytes, &tx.signature).map_err(
                    |e| CoreError::ModuleError {
                        module: "crypto".into(),
                        reason: format!("ed25519 signature verification failed: {e}"),
                    },
                )?;
            }
            KeyType::Secp256k1 => {
                Secp256k1Keypair::verify(&tx.public_key, &sign_bytes, &tx.signature).map_err(
                    |e| CoreError::ModuleError {
                        module: "crypto".into(),
                        reason: format!("secp256k1 signature verification failed: {e}"),
                    },
                )?;
            }
        }

        // 4. Verify message invariants
        for msg in &tx.body.messages {
            if let TxMessage::Transfer { amount, .. } = msg {
                if amount.is_zero() {
                    return Err(CoreError::ModuleError {
                        module: "bank".into(),
                        reason: "transfer amount must be greater than zero".into(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Executes a transaction against state store, mutating balances and nonces atomically.
    pub fn execute_transaction<S: KVStore>(
        &self,
        store: &S,
        tx: &Transaction,
        current_height: u64,
        expected_chain_id: &str,
    ) -> Result<TxReceipt, CoreError> {
        // 1. Static Validation
        self.validate_transaction_static(tx, expected_chain_id)?;

        // 2. Fetch Sender Account State
        let mut sender_state = StateAccessor::get_account(store, &tx.body.sender)?;

        // 3. Nonce Verification (Replay Protection)
        if tx.body.nonce != sender_state.nonce {
            return Err(CoreError::InvalidNonce {
                account_nonce: sender_state.nonce,
                tx_nonce: tx.body.nonce,
            });
        }

        // 4. Calculate total cost for all messages + fee
        let mut total_transfer_cost = sprax_types::Amount::ZERO;
        for msg in &tx.body.messages {
            if let TxMessage::Transfer { amount, .. } = msg {
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

        // 5. Apply state mutations
        // Deduct transfer amount and fee from sender
        sender_state.balance = sender_state
            .balance
            .checked_sub(total_required)
            .map_err(|e| CoreError::StateError(e.to_string()))?;
        sender_state.nonce = sender_state.nonce.saturating_add(1);

        StateAccessor::set_account(store, &tx.body.sender, &sender_state)?;

        // Apply message credits
        for msg in &tx.body.messages {
            match msg {
                TxMessage::Transfer { to, amount } => {
                    let mut recipient_state = StateAccessor::get_account(store, to)?;
                    recipient_state.balance = recipient_state
                        .balance
                        .checked_add(*amount)
                        .map_err(|e| CoreError::StateError(e.to_string()))?;
                    StateAccessor::set_account(store, to, &recipient_state)?;
                }
                _ => {
                    // Other message types (Delegate, ContractCall) will be extended in subsequent phases
                }
            }
        }

        let tx_hash = Hasher::tx_hash(tx).map_err(|e| CoreError::StateError(e.to_string()))?;

        Ok(TxReceipt {
            tx_hash,
            height: current_height,
            success: true,
            gas_used: self.gas_config.base_tx_cost,
            error_message: None,
            return_data: vec![],
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

        let sign_bytes = serde_json::to_vec(&tx_body).unwrap();
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
            .execute_transaction(&store, &tx, 1, "sprax-devnet-1")
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
    }
}
