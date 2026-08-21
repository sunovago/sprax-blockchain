use crate::{environment::Environment, error::NodeError};
use serde::{Deserialize, Serialize};
use sprax_consensus::ConsensusTimeoutConfig;
use sprax_network::NetworkConfig;
use sprax_storage::PruningStrategy;
use std::path::{Path, PathBuf};

/// Live multi-validator BFT consensus configuration for this node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConsensusNodeConfig {
    pub enabled: bool,
    pub local_validator_key_name: Option<String>,
    pub timeouts: ConsensusTimeoutConfig,
}

/// RPC server configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcConfig {
    pub enable_json_rpc: bool,
    pub enable_grpc: bool,
    pub json_rpc_port: u16,
    pub grpc_port: u16,
    pub max_request_body_bytes: usize,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            enable_json_rpc: true,
            enable_grpc: true,
            json_rpc_port: 26657,
            grpc_port: 9090,
            max_request_body_bytes: 5 * 1024 * 1024, // 5 MB
        }
    }
}

/// Telemetry and Prometheus metrics configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub enable_metrics: bool,
    pub prometheus_port: u16,
    pub log_level: String,
    pub log_format_json: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            prometheus_port: 26660,
            log_level: "info".to_string(),
            log_format_json: false,
        }
    }
}

/// Master Node Configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeConfig {
    pub environment: Environment,
    pub chain_id: String,
    pub home_dir: PathBuf,
    pub network: NetworkConfig,
    pub rpc: RpcConfig,
    pub pruning: PruningStrategy,
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub consensus: ConsensusNodeConfig,
}

impl Default for NodeConfig {
    fn default() -> Self {
        let env = Environment::Development;
        Self {
            environment: env,
            chain_id: env.default_chain_id().to_string(),
            home_dir: PathBuf::from(".sprx"),
            network: NetworkConfig::default(),
            rpc: RpcConfig::default(),
            pruning: PruningStrategy::default(),
            telemetry: TelemetryConfig::default(),
            consensus: ConsensusNodeConfig::default(),
        }
    }
}

impl NodeConfig {
    pub fn for_environment(env: Environment, home_dir: PathBuf) -> Self {
        Self {
            environment: env,
            chain_id: env.default_chain_id().to_string(),
            home_dir,
            network: NetworkConfig::default(),
            rpc: RpcConfig::default(),
            pruning: PruningStrategy::default(),
            telemetry: TelemetryConfig::default(),
            consensus: ConsensusNodeConfig::default(),
        }
    }

    pub fn load_from_file(path: &Path) -> Result<Self, NodeError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            NodeError::ConfigError(format!("failed to read config at {path:?}: {e}"))
        })?;
        toml::from_str(&content)
            .map_err(|e| NodeError::ConfigError(format!("failed to parse TOML config: {e}")))
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), NodeError> {
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| NodeError::ConfigError(format!("failed to serialize TOML config: {e}")))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, toml_str)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_config_toml_roundtrip() {
        let config = NodeConfig::for_environment(Environment::Testnet, PathBuf::from("/tmp/.sprx"));
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: NodeConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.environment, parsed.environment);
        assert_eq!(config.chain_id, parsed.chain_id);
    }

    #[test]
    fn test_node_config_consensus_section_toml_roundtrip() {
        let mut config =
            NodeConfig::for_environment(Environment::Testnet, PathBuf::from("/tmp/.sprx"));
        config.consensus.enabled = true;
        config.consensus.local_validator_key_name = Some("alice".to_string());

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: NodeConfig = toml::from_str(&toml_str).unwrap();
        assert!(parsed.consensus.enabled);
        assert_eq!(
            parsed.consensus.local_validator_key_name,
            Some("alice".to_string())
        );
        assert_eq!(
            parsed.consensus.timeouts.timeout_propose_ms,
            config.consensus.timeouts.timeout_propose_ms
        );
    }

    #[test]
    fn test_node_config_without_consensus_section_defaults_disabled() {
        let config = NodeConfig::for_environment(Environment::Testnet, PathBuf::from("/tmp/.sprx"));
        let full_toml = toml::to_string_pretty(&config).unwrap();
        let without_consensus: String = full_toml
            .lines()
            .scan(false, |in_consensus_section, line| {
                if line.trim_start().starts_with('[') {
                    *in_consensus_section =
                        line.trim() == "[consensus]" || line.trim() == "[consensus.timeouts]";
                }
                Some((*in_consensus_section, line))
            })
            .filter(|(skip, _)| !*skip)
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n");

        assert!(!without_consensus.contains("[consensus]"));

        let parsed: NodeConfig = toml::from_str(&without_consensus).unwrap();
        assert!(!parsed.consensus.enabled);
        assert_eq!(parsed.consensus.local_validator_key_name, None);
    }
}
