use crate::vote::Vote;
use serde::{Deserialize, Serialize};
use sprax_types::Address;

/// Slashing evidence for validator equivocation (double-signing).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquivocationEvidence {
    pub validator_address: Address,
    pub height: u64,
    pub round: u32,
    pub vote_a: Vote,
    pub vote_b: Vote,
}

impl EquivocationEvidence {
    pub fn is_valid_equivocation(&self) -> bool {
        self.vote_a.validator_address == self.vote_b.validator_address
            && self.vote_a.height == self.vote_b.height
            && self.vote_a.round == self.vote_b.round
            && self.vote_a.vote_type == self.vote_b.vote_type
            && self.vote_a.block_hash != self.vote_b.block_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vote::VoteType;
    use sprax_types::Hash32;

    #[test]
    fn test_equivocation_validation() {
        let addr = Address::new([5u8; 20]);
        let vote1 = Vote::new(
            VoteType::Precommit,
            100,
            0,
            Some(Hash32::new([1u8; 32])),
            addr,
            vec![1; 64],
        );
        let vote2 = Vote::new(
            VoteType::Precommit,
            100,
            0,
            Some(Hash32::new([2u8; 32])),
            addr,
            vec![2; 64],
        );

        let evidence = EquivocationEvidence {
            validator_address: addr,
            height: 100,
            round: 0,
            vote_a: vote1,
            vote_b: vote2,
        };

        assert!(evidence.is_valid_equivocation());
    }
}
