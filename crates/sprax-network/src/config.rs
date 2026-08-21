use crate::peer::NodeRole;
use serde::{Deserialize, Serialize};

/// P2P Network transport and connection configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub role: NodeRole,
    pub listen_addr: String,
    pub p2p_port: u16,
    pub max_inbound_peers: usize,
    pub max_outbound_peers: usize,
    pub bootstrap_peers: Vec<String>,
    pub enable_nat_upnp: bool,
    pub max_message_size_bytes: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            role: NodeRole::FullNode,
            listen_addr: "0.0.0.0".to_string(),
            p2p_port: 26656,
            max_inbound_peers: 40,
            max_outbound_peers: 20,
            bootstrap_peers: vec![],
            enable_nat_upnp: true,
            max_message_size_bytes: 4 * 1024 * 1024, // 4 MB
        }
    }
}
