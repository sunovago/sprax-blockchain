use sprax_consensus::{CommissionRates, StakingKeeper, StakingParams, ValidatorDescription};
use sprax_types::{Address, Amount};

fn register(keeper: &mut StakingKeeper, index: u8, stake: Amount) {
    keeper
        .register_validator(
            Address::new([index; 20]),
            vec![index; 32],
            ValidatorDescription {
                moniker: format!("validator-{index}"),
                identity: String::new(),
                website: String::new(),
                details: String::new(),
            },
            stake,
            CommissionRates::default(),
        )
        .unwrap();
}

#[test]
fn equal_stake_truncation_is_independent_of_hashmap_and_registration_order() {
    let params = StakingParams {
        max_validators: 2,
        ..StakingParams::default()
    };
    let mut first = StakingKeeper::new(params.clone());
    let mut second = StakingKeeper::new(params);
    for index in [1, 2, 3] {
        register(&mut first, index, Amount::from_sprx_whole(100_000).unwrap());
    }
    for index in [3, 2, 1] {
        register(
            &mut second,
            index,
            Amount::from_sprx_whole(100_000).unwrap(),
        );
    }
    let first = first.get_active_validator_set().unwrap();
    let second = second.get_active_validator_set().unwrap();
    assert_eq!(first, second);
    assert_eq!(
        first
            .validators()
            .iter()
            .map(|validator| validator.address)
            .collect::<Vec<_>>(),
        vec![Address::new([1; 20]), Address::new([2; 20])]
    );
}

#[test]
fn stake_power_conversion_rejects_wraparound() {
    let mut keeper = StakingKeeper::default();
    register(
        &mut keeper,
        1,
        Amount::from_atto((u128::from(u64::MAX) + 2) * 1_000_000_000_000_000_000),
    );
    assert!(keeper
        .get_active_validator_set()
        .unwrap_err()
        .to_string()
        .contains("overflow"));
}
