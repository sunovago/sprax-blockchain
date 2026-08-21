use crate::{error::CoreError, gas::GasMeter, state::StateTransitionContext};
use sprax_storage::KVStore;
use sprax_types::{Address, Amount, TxMessage, TxReceipt};

/// Modular application component interface.
pub trait AppModule<S: KVStore> {
    fn name(&self) -> &'static str;
    fn init_genesis(&self, ctx: &mut StateTransitionContext<'_, S>) -> Result<(), CoreError>;
    fn execute_message(
        &self,
        ctx: &mut StateTransitionContext<'_, S>,
        gas_meter: &mut GasMeter,
        sender: &Address,
        msg: &TxMessage,
    ) -> Result<TxReceipt, CoreError>;
}

/// Genesis initial state configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenesisAccount {
    pub address: Address,
    pub initial_balance: Amount,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenesisState {
    pub chain_id: String,
    pub genesis_time_unix_secs: u64,
    pub accounts: Vec<GenesisAccount>,
}

impl Default for GenesisState {
    fn default() -> Self {
        Self {
            chain_id: "sprax-devnet-1".to_string(),
            genesis_time_unix_secs: 1_700_000_000,
            accounts: vec![],
        }
    }
}
