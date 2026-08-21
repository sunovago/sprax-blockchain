use crate::error::FaucetError;
use parking_lot::RwLock;
use sprax_types::Address;
use std::collections::HashMap;

/// Anti-abuse sliding-window rate limiter for public testnet faucets.
#[derive(Debug)]
pub struct RateLimiter {
    window_duration_secs: u64,
    last_claim_by_address: RwLock<HashMap<Address, u64>>,
    last_claim_by_ip: RwLock<HashMap<String, u64>>,
}

impl RateLimiter {
    pub const DEFAULT_WINDOW_SECS: u64 = 86_400; // 24 hours

    pub fn new(window_duration_secs: u64) -> Self {
        Self {
            window_duration_secs,
            last_claim_by_address: RwLock::new(HashMap::new()),
            last_claim_by_ip: RwLock::new(HashMap::new()),
        }
    }

    /// Validates eligibility and updates timestamp upon successful check.
    pub fn check_and_record(
        &self,
        address: &Address,
        client_ip: &str,
        now_unix: u64,
    ) -> Result<(), FaucetError> {
        // 1. Check Address rate limit
        let mut addr_map = self.last_claim_by_address.write();
        if let Some(&last_time) = addr_map.get(address) {
            let elapsed = now_unix.saturating_sub(last_time);
            if elapsed < self.window_duration_secs {
                return Err(FaucetError::RateLimitExceeded {
                    retry_after_secs: self.window_duration_secs - elapsed,
                });
            }
        }

        // 2. Check IP rate limit (if provided)
        let clean_ip = client_ip.trim();
        let mut ip_map = self.last_claim_by_ip.write();
        if !clean_ip.is_empty() && clean_ip != "127.0.0.1" && clean_ip != "localhost" {
            if let Some(&last_time) = ip_map.get(clean_ip) {
                let elapsed = now_unix.saturating_sub(last_time);
                if elapsed < self.window_duration_secs {
                    return Err(FaucetError::RateLimitExceeded {
                        retry_after_secs: self.window_duration_secs - elapsed,
                    });
                }
            }
            ip_map.insert(clean_ip.to_string(), now_unix);
        }

        addr_map.insert(*address, now_unix);
        Ok(())
    }

    pub fn window_duration_secs(&self) -> u64 {
        self.window_duration_secs
    }
}
