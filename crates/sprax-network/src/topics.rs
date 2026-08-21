/// Canonical GossipSub Topic strings for the SPRX Protocol.
#[derive(Debug)]
pub struct GossipTopics;

impl GossipTopics {
    pub const TX_GOSSIP_V1: &'static str = "/sprax/mempool/tx/v1";
    pub const BLOCK_PROPOSAL_V1: &'static str = "/sprax/consensus/proposal/v1";
    pub const CONSENSUS_VOTE_V1: &'static str = "/sprax/consensus/vote/v1";
    pub const EVIDENCE_V1: &'static str = "/sprax/consensus/evidence/v1";
    pub const STATE_SYNC_V1: &'static str = "/sprax/sync/snapshot/v1";

    #[must_use]
    pub fn all_core_topics() -> Vec<&'static str> {
        vec![
            Self::TX_GOSSIP_V1,
            Self::BLOCK_PROPOSAL_V1,
            Self::CONSENSUS_VOTE_V1,
            Self::EVIDENCE_V1,
            Self::STATE_SYNC_V1,
        ]
    }
}
