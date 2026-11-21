pub use sprax_types::EquivocationEvidence;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vote::VoteType;
    use sprax_types::{Address, Hash32, Vote};

    #[test]
    fn test_equivocation_validation() {
        let addr = Address::new([5u8; 20]);
        let vote1 = Vote::new(
            Hash32::ZERO,
            VoteType::Precommit,
            100,
            0,
            Some(Hash32::new([1u8; 32])),
            addr,
            vec![1; 64],
        );
        let vote2 = Vote::new(
            Hash32::ZERO,
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
