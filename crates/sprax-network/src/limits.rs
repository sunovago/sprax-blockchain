use crate::{error::NetworkError, peer::PeerId};
use parking_lot::RwLock;
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Strict resource budgets and operational limits for P2P networking.
#[derive(Debug, Clone)]
pub struct NetworkLimits {
    pub max_inbound_peers: usize,
    pub max_outbound_peers: usize,
    pub max_pending_handshakes_global: usize,
    pub max_pending_handshakes_per_ip: usize,
    pub handshake_timeout: Duration,
    pub max_message_size_bytes: usize,
    pub max_peer_queue_bytes: usize,
    pub max_known_addresses: usize,
    pub max_proposal_reqs_per_sec: usize,
    pub ban_duration: Duration,
}

impl Default for NetworkLimits {
    fn default() -> Self {
        Self {
            max_inbound_peers: 40,
            max_outbound_peers: 20,
            max_pending_handshakes_global: 16,
            max_pending_handshakes_per_ip: 2,
            handshake_timeout: Duration::from_secs(5),
            max_message_size_bytes: 4 * 1024 * 1024, // 4 MiB
            max_peer_queue_bytes: 8 * 1024 * 1024,   // 8 MiB
            max_known_addresses: 1024,
            max_proposal_reqs_per_sec: 20,
            ban_duration: Duration::from_secs(600), // 10 minutes
        }
    }
}

/// Resource manager enforcing global and per-peer budgets.
#[derive(Debug, Clone)]
pub struct ResourceManager {
    limits: NetworkLimits,
    global_pending_handshakes: Arc<Semaphore>,
    ip_pending_handshakes: Arc<RwLock<HashMap<IpAddr, usize>>>,
    banned_peers: Arc<RwLock<HashMap<PeerId, Instant>>>,
    banned_ips: Arc<RwLock<HashMap<IpAddr, Instant>>>,
}

impl ResourceManager {
    pub fn new(limits: NetworkLimits) -> Self {
        Self {
            global_pending_handshakes: Arc::new(Semaphore::new(
                limits.max_pending_handshakes_global,
            )),
            ip_pending_handshakes: Arc::new(RwLock::new(HashMap::new())),
            banned_peers: Arc::new(RwLock::new(HashMap::new())),
            banned_ips: Arc::new(RwLock::new(HashMap::new())),
            limits,
        }
    }

    pub fn limits(&self) -> &NetworkLimits {
        &self.limits
    }

    /// Attempts to acquire a pending handshake permit for the given remote IP.
    pub fn acquire_handshake_permit(&self, ip: IpAddr) -> Result<HandshakePermit, NetworkError> {
        // Check IP ban
        self.cleanup_expired_bans();
        if self.is_ip_banned(ip) {
            return Err(NetworkError::PeerBanned(format!(
                "IP {ip} is temporarily banned"
            )));
        }

        // Check per-IP pending handshake limit
        {
            let mut ip_counts = self.ip_pending_handshakes.write();
            let count = ip_counts.entry(ip).or_insert(0);
            if *count >= self.limits.max_pending_handshakes_per_ip {
                return Err(NetworkError::BudgetExceeded(format!(
                    "exceeded pending handshake limit for IP {ip}"
                )));
            }
            *count += 1;
        }

        // Check global pending handshake limit
        let permit = match self.global_pending_handshakes.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                // Decrement IP count on failure
                let mut ip_counts = self.ip_pending_handshakes.write();
                if let Some(c) = ip_counts.get_mut(&ip) {
                    *c = c.saturating_sub(1);
                    if *c == 0 {
                        ip_counts.remove(&ip);
                    }
                }
                return Err(NetworkError::BudgetExceeded(
                    "global pending handshake capacity exhausted".into(),
                ));
            }
        };

        Ok(HandshakePermit {
            _permit: permit,
            ip,
            ip_pending_handshakes: self.ip_pending_handshakes.clone(),
        })
    }

    pub fn is_peer_banned(&self, peer_id: &PeerId) -> bool {
        let bans = self.banned_peers.read();
        if let Some(expiry) = bans.get(peer_id) {
            Instant::now() < *expiry
        } else {
            false
        }
    }

    pub fn is_ip_banned(&self, ip: IpAddr) -> bool {
        let bans = self.banned_ips.read();
        if let Some(expiry) = bans.get(&ip) {
            Instant::now() < *expiry
        } else {
            false
        }
    }

    pub fn ban_peer(&self, peer_id: PeerId) {
        let expiry = Instant::now() + self.limits.ban_duration;
        let mut bans = self.banned_peers.write();
        if bans.len() < 1024 {
            bans.insert(peer_id, expiry);
        }
    }

    pub fn ban_ip(&self, ip: IpAddr) {
        let expiry = Instant::now() + self.limits.ban_duration;
        let mut bans = self.banned_ips.write();
        if bans.len() < 1024 {
            bans.insert(ip, expiry);
        }
    }

    fn cleanup_expired_bans(&self) {
        let now = Instant::now();
        {
            let mut bans = self.banned_peers.write();
            bans.retain(|_, expiry| *expiry > now);
        }
        {
            let mut bans = self.banned_ips.write();
            bans.retain(|_, expiry| *expiry > now);
        }
    }

    /// Validates a peer address according to network policy.
    pub fn validate_socket_addr(&self, addr_str: &str) -> Result<SocketAddr, NetworkError> {
        if addr_str.is_empty() || addr_str.len() > 128 {
            return Err(NetworkError::InvalidAddress(
                "address length must be between 1 and 128 characters".into(),
            ));
        }

        let addr: SocketAddr = addr_str
            .parse()
            .map_err(|e| NetworkError::InvalidAddress(format!("malformed socket address: {e}")))?;

        let ip = addr.ip();
        if ip.is_unspecified() {
            return Err(NetworkError::InvalidAddress(
                "unspecified address (0.0.0.0 / ::) is not dialable".into(),
            ));
        }
        if ip.is_multicast() {
            return Err(NetworkError::InvalidAddress(
                "multicast address is not permitted".into(),
            ));
        }
        if addr.port() == 0 {
            return Err(NetworkError::InvalidAddress(
                "port 0 is not permitted".into(),
            ));
        }

        Ok(addr)
    }
}

/// Scoped RAII permit for an in-progress handshake.
#[derive(Debug)]
pub struct HandshakePermit {
    _permit: OwnedSemaphorePermit,
    ip: IpAddr,
    ip_pending_handshakes: Arc<RwLock<HashMap<IpAddr, usize>>>,
}

impl Drop for HandshakePermit {
    fn drop(&mut self) {
        let mut ip_counts = self.ip_pending_handshakes.write();
        if let Some(c) = ip_counts.get_mut(&self.ip) {
            *c = c.saturating_sub(1);
            if *c == 0 {
                ip_counts.remove(&self.ip);
            }
        }
    }
}

/// Tracks per-peer outbound queue byte utilization.
#[derive(Debug, Default)]
pub struct OutboundByteBudget {
    queued_bytes: Arc<AtomicUsize>,
    max_bytes: usize,
}

impl OutboundByteBudget {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            queued_bytes: Arc::new(AtomicUsize::new(0)),
            max_bytes,
        }
    }

    pub fn try_reserve(&self, bytes: usize) -> Result<ByteReservation, NetworkError> {
        let current = self.queued_bytes.load(Ordering::Relaxed);
        if current.saturating_add(bytes) > self.max_bytes {
            return Err(NetworkError::BudgetExceeded(format!(
                "peer outbound queue byte limit exceeded ({} > {})",
                current + bytes,
                self.max_bytes
            )));
        }

        self.queued_bytes.fetch_add(bytes, Ordering::SeqCst);
        Ok(ByteReservation {
            bytes,
            counter: Arc::clone(&self.queued_bytes),
        })
    }

    pub fn current_queued_bytes(&self) -> usize {
        self.queued_bytes.load(Ordering::Relaxed)
    }
}

/// Scoped RAII reservation for queued outbound bytes.
#[derive(Debug)]
pub struct ByteReservation {
    bytes: usize,
    counter: Arc<AtomicUsize>,
}

impl Drop for ByteReservation {
    fn drop(&mut self) {
        self.counter.fetch_sub(self.bytes, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_permits_enforce_ip_and_global_limits() {
        let limits = NetworkLimits {
            max_pending_handshakes_global: 2,
            max_pending_handshakes_per_ip: 1,
            ..Default::default()
        };
        let manager = ResourceManager::new(limits);

        let ip1: IpAddr = "192.168.1.1".parse().unwrap();
        let ip2: IpAddr = "192.168.1.2".parse().unwrap();

        let permit1 = manager.acquire_handshake_permit(ip1).unwrap();
        // Second from same IP should fail
        assert!(manager.acquire_handshake_permit(ip1).is_err());

        // From different IP should succeed
        let permit2 = manager.acquire_handshake_permit(ip2).unwrap();

        // Third should fail globally (global capacity 2)
        let ip3: IpAddr = "192.168.1.3".parse().unwrap();
        assert!(manager.acquire_handshake_permit(ip3).is_err());

        // Drop permit1 -> ip1 and global slot freed
        drop(permit1);
        let permit3 = manager.acquire_handshake_permit(ip3).unwrap();
        drop(permit2);
        drop(permit3);
    }

    #[test]
    fn test_outbound_byte_budget_enforcement() {
        let budget = OutboundByteBudget::new(100);

        let res1 = budget.try_reserve(60).unwrap();
        assert_eq!(budget.current_queued_bytes(), 60);

        // Exceeding 100 fails
        assert!(budget.try_reserve(50).is_err());

        // Under 100 succeeds
        let res2 = budget.try_reserve(30).unwrap();
        assert_eq!(budget.current_queued_bytes(), 90);

        drop(res1);
        assert_eq!(budget.current_queued_bytes(), 30);
        drop(res2);
        assert_eq!(budget.current_queued_bytes(), 0);
    }

    #[test]
    fn test_address_validation_rules() {
        let manager = ResourceManager::new(NetworkLimits::default());

        assert!(manager.validate_socket_addr("127.0.0.1:26656").is_ok());
        assert!(manager.validate_socket_addr("0.0.0.0:26656").is_err());
        assert!(manager.validate_socket_addr("224.0.0.1:26656").is_err());
        assert!(manager.validate_socket_addr("invalid").is_err());
        assert!(manager.validate_socket_addr("127.0.0.1:0").is_err());
    }
}
