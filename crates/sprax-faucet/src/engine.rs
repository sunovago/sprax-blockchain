use crate::{
    error::FaucetError,
    limiter::RateLimiter,
    models::{AuditEntry, ClaimResponse, FaucetStats},
};
use parking_lot::RwLock;
use sprax_core::ledger::ChainLedger;
use sprax_crypto::Ed25519Keypair;
use sprax_types::{Address, Amount, ChainId, KeyType, Transaction, TxBody, TxFee, TxMessage};
use tracing::info;

pub const DEFAULT_FAUCET_PAYOUT_SPRX: u64 = 100;
pub const MAX_FAUCET_PAYOUT_SPRX: u64 = 500;
pub const TESTNET_NOTICE: &str = "SPRX TESTNET ONLY — TOKENS HAVE NO REAL WORLD VALUE";

/// Core Faucet Disbursement & Audit Engine.
#[derive(Debug)]
pub struct FaucetService {
    keypair: Ed25519Keypair,
    chain_id: ChainId,
    limiter: RateLimiter,
    max_payout: Amount,
    total_disbursed: RwLock<Amount>,
    claims_count: RwLock<usize>,
    audit_log: RwLock<Vec<AuditEntry>>,
}

impl FaucetService {
    pub fn new(
        keypair: Ed25519Keypair,
        chain_id: ChainId,
        rate_limit_secs: u64,
        max_payout_sprx: u64,
    ) -> Self {
        let max_payout = Amount::from_sprx_whole(max_payout_sprx as u128).unwrap_or(Amount::ZERO);
        Self {
            keypair,
            chain_id,
            limiter: RateLimiter::new(rate_limit_secs),
            max_payout,
            total_disbursed: RwLock::new(Amount::ZERO),
            claims_count: RwLock::new(0),
            audit_log: RwLock::new(Vec::new()),
        }
    }

    pub fn faucet_address(&self) -> Address {
        self.keypair.address()
    }

    /// Processes a public faucet disbursement request.
    pub fn request_funds<
        S: sprax_storage::KVStore
            + sprax_storage::StateCommitment
            + sprax_storage::ChainMetaStore
            + Clone
            + 'static,
    >(
        &self,
        ledger: &mut ChainLedger<S>,
        recipient_str: &str,
        requested_sprx: Option<u64>,
        client_ip: &str,
        now_unix: u64,
    ) -> Result<ClaimResponse, FaucetError> {
        let recipient = Address::parse(recipient_str.trim())
            .map_err(|e| FaucetError::InvalidAddress(format!("invalid address: {e}")))?;

        let payout_sprx = requested_sprx.unwrap_or(DEFAULT_FAUCET_PAYOUT_SPRX);
        let payout_amount = Amount::from_sprx_whole(payout_sprx as u128)
            .map_err(|e| FaucetError::InvalidAddress(e.to_string()))?;

        if payout_amount > self.max_payout {
            return Err(FaucetError::AmountExceedsLimit {
                requested: format!("{payout_sprx} tSPRX"),
                max_allowed: self.max_payout.to_string(),
            });
        }

        // Check Rate Limiter
        self.limiter
            .check_and_record(&recipient, client_ip, now_unix)?;

        // Fetch faucet account balance & nonce from ledger
        let faucet_addr = self.faucet_address();
        let faucet_acc = ledger
            .get_account(&faucet_addr)
            .map_err(|_| FaucetError::InsufficientFunds)?;

        let total_needed = payout_amount
            .checked_add(TxFee::default().amount)
            .map_err(|_| FaucetError::InsufficientFunds)?;

        if faucet_acc.balance < total_needed {
            return Err(FaucetError::InsufficientFunds);
        }

        // Construct Transfer Transaction
        let tx_body = TxBody {
            chain_id: self.chain_id.clone(),
            sender: faucet_addr,
            nonce: faucet_acc.nonce,
            messages: vec![TxMessage::Transfer {
                to: recipient,
                amount: payout_amount,
            }],
            fee: TxFee::default(),
            memo: "SPRX Public Testnet Faucet Disbursement".into(),
            timeout_height: ledger.height() + 50,
        };

        let sign_bytes = tx_body
            .sign_bytes()
            .map_err(|e| FaucetError::SubmissionError(format!("sign bytes error: {e}")))?;

        let sig = self.keypair.sign(&sign_bytes);
        let tx = Transaction::new(
            tx_body,
            KeyType::Ed25519,
            self.keypair.public_key_bytes().to_vec(),
            sig,
        )
        .map_err(|e| FaucetError::SubmissionError(e.to_string()))?;

        let tx_hash = ledger
            .submit_transaction(tx)
            .map_err(|e| FaucetError::SubmissionError(e.to_string()))?;

        // Update statistics and audit log
        let mut disbursed = self.total_disbursed.write();
        *disbursed = (*disbursed)
            .checked_add(payout_amount)
            .unwrap_or(*disbursed);
        *self.claims_count.write() += 1;

        let audit = AuditEntry {
            timestamp_unix: now_unix,
            recipient,
            amount: payout_amount,
            tx_hash,
            client_ip: client_ip.to_string(),
        };
        self.audit_log.write().push(audit);

        info!(
            recipient = %recipient,
            amount = %payout_amount,
            tx_hash = %tx_hash,
            "Disbursed testnet tokens from faucet"
        );

        Ok(ClaimResponse {
            success: true,
            tx_hash,
            recipient,
            amount: payout_amount,
            amount_sprx: format!("{payout_sprx} tSPRX"),
            message: "Tokens successfully disbursed to testnet address".into(),
            network_notice: TESTNET_NOTICE.into(),
        })
    }

    /// Derives public operational status and pool health.
    pub fn get_stats<
        S: sprax_storage::KVStore
            + sprax_storage::StateCommitment
            + sprax_storage::ChainMetaStore
            + Clone
            + 'static,
    >(
        &self,
        ledger: &ChainLedger<S>,
    ) -> FaucetStats {
        let faucet_addr = self.faucet_address();
        let balance = ledger
            .get_account(&faucet_addr)
            .map(|a| a.balance)
            .unwrap_or(Amount::ZERO);

        FaucetStats {
            network: self.chain_id.to_string(),
            faucet_address: faucet_addr,
            available_balance: balance,
            total_disbursed: *self.total_disbursed.read(),
            total_claims_count: *self.claims_count.read(),
            max_payout_per_request: self.max_payout,
            rate_limit_window_secs: self.limiter.window_duration_secs(),
            disclaimer: TESTNET_NOTICE.into(),
        }
    }

    pub fn audit_log(&self) -> Vec<AuditEntry> {
        self.audit_log.read().clone()
    }
}
