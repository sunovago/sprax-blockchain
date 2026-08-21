use sprax_types::{Address, Amount};
use sprax_wasm::{
    ContractError, Env, EscrowContract, EscrowExecuteMsg, EscrowQueryMsg, EscrowState,
    EscrowStatus, GasMeter, GovExecuteMsg, GovQueryMsg, GovernanceContract, InitialBalance,
    MessageInfo, Proposal, ProposalStatus, TokenContract, TokenExecuteMsg, TokenInfo,
    TokenQueryMsg, VoteOption, WasmContractEngine,
};

#[test]
fn test_cw20_token_full_lifecycle_and_access_control() {
    let vm = WasmContractEngine::new();
    let alice = Address::new([1u8; 20]);
    let bob = Address::new([2u8; 20]);
    let charlie = Address::new([3u8; 20]);

    let mut env = Env {
        block_height: 1,
        block_timestamp_unix: 1_700_000_000,
        chain_id: "sprax-devnet-1".into(),
        contract_address: Address::ZERO,
    };

    let mut gas = GasMeter::new(500_000);

    // 1. Store Code & Instantiate CW20 Token
    let code_id = vm.store_code(alice, vec![0xaa; 100], 1).unwrap();
    let _contract_addr = vm
        .instantiate(
            alice,
            code_id,
            "SPRX Stable USD".into(),
            Some(alice),
            Amount::ZERO,
            &mut env,
            &mut gas,
        )
        .unwrap();

    let info = MessageInfo {
        sender: alice,
        funds: Amount::ZERO,
    };

    let mut storage = vm.storage().write();

    let init_res = TokenContract::instantiate(
        &mut storage,
        &env,
        &info,
        &mut gas,
        "SPRX USD".into(),
        "USDS".into(),
        6,
        vec![InitialBalance {
            address: alice,
            amount: Amount::from_sprx_whole(1_000_000).unwrap(),
        }],
        Some(alice),
    )
    .unwrap();

    assert_eq!(init_res.events.len(), 1);
    assert_eq!(init_res.events[0].event_type, "cw20-instantiate");

    // 2. Query Initial Balance of Alice
    let bal_bytes =
        TokenContract::query(&storage, &env, TokenQueryMsg::Balance { address: alice }).unwrap();
    let alice_bal: Amount = serde_json::from_slice(&bal_bytes).unwrap();
    assert_eq!(alice_bal, Amount::from_sprx_whole(1_000_000).unwrap());

    // 3. Alice Transfers 200,000 to Bob
    let transfer_res = TokenContract::execute(
        &mut storage,
        &env,
        &info,
        &mut gas,
        TokenExecuteMsg::Transfer {
            recipient: bob,
            amount: Amount::from_sprx_whole(200_000).unwrap(),
        },
    )
    .unwrap();

    assert_eq!(transfer_res.events[0].event_type, "cw20-transfer");

    let bob_bal: Amount = serde_json::from_slice(
        &TokenContract::query(&storage, &env, TokenQueryMsg::Balance { address: bob }).unwrap(),
    )
    .unwrap();
    assert_eq!(bob_bal, Amount::from_sprx_whole(200_000).unwrap());

    // 4. Bob Approves Charlie for 50,000
    let bob_info = MessageInfo {
        sender: bob,
        funds: Amount::ZERO,
    };
    TokenContract::execute(
        &mut storage,
        &env,
        &bob_info,
        &mut gas,
        TokenExecuteMsg::Approve {
            spender: charlie,
            amount: Amount::from_sprx_whole(50_000).unwrap(),
        },
    )
    .unwrap();

    let allowance_bytes = TokenContract::query(
        &storage,
        &env,
        TokenQueryMsg::Allowance {
            owner: bob,
            spender: charlie,
        },
    )
    .unwrap();
    let allowance: Amount = serde_json::from_slice(&allowance_bytes).unwrap();
    assert_eq!(allowance, Amount::from_sprx_whole(50_000).unwrap());

    // 5. Charlie Executes TransferFrom (Bob -> Charlie 30,000)
    let charlie_info = MessageInfo {
        sender: charlie,
        funds: Amount::ZERO,
    };
    TokenContract::execute(
        &mut storage,
        &env,
        &charlie_info,
        &mut gas,
        TokenExecuteMsg::TransferFrom {
            owner: bob,
            recipient: charlie,
            amount: Amount::from_sprx_whole(30_000).unwrap(),
        },
    )
    .unwrap();

    let charlie_bal: Amount = serde_json::from_slice(
        &TokenContract::query(&storage, &env, TokenQueryMsg::Balance { address: charlie }).unwrap(),
    )
    .unwrap();
    assert_eq!(charlie_bal, Amount::from_sprx_whole(30_000).unwrap());

    // 6. Non-Admin (Bob) attempts to Mint -> Fails with Unauthorized
    let bad_mint = TokenContract::execute(
        &mut storage,
        &env,
        &bob_info,
        &mut gas,
        TokenExecuteMsg::Mint {
            recipient: bob,
            amount: Amount::from_sprx_whole(100).unwrap(),
        },
    );
    assert_eq!(
        bad_mint,
        Err(ContractError::Unauthorized(
            "only minter can mint new tokens".into()
        ))
    );

    // 7. Admin (Alice) Mints 100,000
    TokenContract::execute(
        &mut storage,
        &env,
        &info,
        &mut gas,
        TokenExecuteMsg::Mint {
            recipient: alice,
            amount: Amount::from_sprx_whole(100_000).unwrap(),
        },
    )
    .unwrap();

    // 8. Burn Tokens & Verify Total Supply
    TokenContract::execute(
        &mut storage,
        &env,
        &info,
        &mut gas,
        TokenExecuteMsg::Burn {
            amount: Amount::from_sprx_whole(50_000).unwrap(),
        },
    )
    .unwrap();

    let info_bytes = TokenContract::query(&storage, &env, TokenQueryMsg::TokenInfo {}).unwrap();
    let token_info: TokenInfo = serde_json::from_slice(&info_bytes).unwrap();
    assert_eq!(
        token_info.total_supply,
        Amount::from_sprx_whole(1_050_000).unwrap() // 1M - 200k(bob) + 200k(bob) + 100k(mint) - 50k(burn) = 1.05M
    );
}

#[test]
fn test_escrow_payment_lifecycle_and_timeout_refund() {
    let vm = WasmContractEngine::new();
    let sender = Address::new([1u8; 20]);
    let recipient = Address::new([2u8; 20]);
    let arbiter = Address::new([3u8; 20]);
    let attacker = Address::new([4u8; 20]);

    let mut env = Env {
        block_height: 10,
        block_timestamp_unix: 1_700_000_000,
        chain_id: "sprax-devnet-1".into(),
        contract_address: Address::new([0xee; 20]),
    };

    let mut gas = GasMeter::new(100_000);
    let mut storage = vm.storage().write();

    let info = MessageInfo {
        sender,
        funds: Amount::from_sprx_whole(5_000).unwrap(),
    };

    // 1. Instantiate Escrow locked until block 100
    EscrowContract::instantiate(&mut storage, &env, &info, &mut gas, recipient, arbiter, 100)
        .unwrap();

    // 2. Attacker attempts to Release funds -> Unauthorized
    let bad_release = EscrowContract::execute(
        &mut storage,
        &env,
        &MessageInfo {
            sender: attacker,
            funds: Amount::ZERO,
        },
        &mut gas,
        EscrowExecuteMsg::Release {},
    );
    assert_eq!(
        bad_release,
        Err(ContractError::Unauthorized(
            "only arbiter or sender can release escrow funds".into()
        ))
    );

    // 3. Sender attempts to Refund before timeout (Height 10 < 100) -> Unauthorized
    let early_refund = EscrowContract::execute(
        &mut storage,
        &env,
        &info,
        &mut gas,
        EscrowExecuteMsg::Refund {},
    );
    assert_eq!(
        early_refund,
        Err(ContractError::Unauthorized(
            "refund requires arbiter authorization or timeout expiration".into()
        ))
    );

    // 4. Advance block height past timeout (Height 105 >= 100) -> Sender Refund succeeds!
    env.block_height = 105;
    let refund_res = EscrowContract::execute(
        &mut storage,
        &env,
        &info,
        &mut gas,
        EscrowExecuteMsg::Refund {},
    )
    .unwrap();

    assert_eq!(refund_res.messages.len(), 1);

    // 5. Query Escrow Status -> Refunded
    let state_bytes = EscrowContract::query(&storage, &env, EscrowQueryMsg::GetEscrow {}).unwrap();
    let state: EscrowState = serde_json::from_slice(&state_bytes).unwrap();
    assert_eq!(state.status, EscrowStatus::Refunded);
    assert_eq!(state.amount, Amount::ZERO);
}

#[test]
fn test_governance_proposal_and_voting_lifecycle() {
    let vm = WasmContractEngine::new();
    let proposer = Address::new([1u8; 20]);
    let voter1 = Address::new([2u8; 20]);
    let voter2 = Address::new([3u8; 20]);

    let mut env = Env {
        block_height: 1,
        block_timestamp_unix: 1_700_000_000,
        chain_id: "sprax-devnet-1".into(),
        contract_address: Address::new([0xaa; 20]),
    };

    let mut gas = GasMeter::new(100_000);
    let mut storage = vm.storage().write();

    // 1. Instantiate Governance with 10 block voting period and 50 required quorum votes
    GovernanceContract::instantiate(
        &mut storage,
        &env,
        &MessageInfo {
            sender: proposer,
            funds: Amount::ZERO,
        },
        &mut gas,
        10,
        50,
    )
    .unwrap();

    // 2. Submit Proposal
    GovernanceContract::execute(
        &mut storage,
        &env,
        &MessageInfo {
            sender: proposer,
            funds: Amount::ZERO,
        },
        &mut gas,
        GovExecuteMsg::SubmitProposal {
            title: "SIP-001: Upgrade Gas Parameters".into(),
            description: "Optimize smart contract base gas".into(),
        },
    )
    .unwrap();

    // 3. Voter 1 votes YES with 40 weight
    GovernanceContract::execute(
        &mut storage,
        &env,
        &MessageInfo {
            sender: voter1,
            funds: Amount::ZERO,
        },
        &mut gas,
        GovExecuteMsg::CastVote {
            proposal_id: 1,
            option: VoteOption::Yes,
            voting_weight: 40,
        },
    )
    .unwrap();

    // 4. Duplicate Vote by Voter 1 -> Fails with Unauthorized
    let dup_vote = GovernanceContract::execute(
        &mut storage,
        &env,
        &MessageInfo {
            sender: voter1,
            funds: Amount::ZERO,
        },
        &mut gas,
        GovExecuteMsg::CastVote {
            proposal_id: 1,
            option: VoteOption::No,
            voting_weight: 40,
        },
    );
    assert_eq!(
        dup_vote,
        Err(ContractError::Unauthorized(
            "voter has already cast a ballot".into()
        ))
    );

    // 5. Voter 2 votes YES with 25 weight (Total Yes: 65 > 50 Quorum)
    GovernanceContract::execute(
        &mut storage,
        &env,
        &MessageInfo {
            sender: voter2,
            funds: Amount::ZERO,
        },
        &mut gas,
        GovExecuteMsg::CastVote {
            proposal_id: 1,
            option: VoteOption::Yes,
            voting_weight: 25,
        },
    )
    .unwrap();

    // 6. Advance block height past voting end (Height 12 > 11) -> Tally & Execute
    env.block_height = 12;
    GovernanceContract::execute(
        &mut storage,
        &env,
        &MessageInfo {
            sender: proposer,
            funds: Amount::ZERO,
        },
        &mut gas,
        GovExecuteMsg::TallyAndExecute { proposal_id: 1 },
    )
    .unwrap();

    let prop_bytes =
        GovernanceContract::query(&storage, &env, GovQueryMsg::GetProposal { proposal_id: 1 })
            .unwrap();
    let prop: Proposal = serde_json::from_slice(&prop_bytes).unwrap();
    assert_eq!(prop.status, ProposalStatus::Passed);
    assert_eq!(prop.yes_votes, 65);
}

#[test]
fn test_reentrancy_lock_and_gas_exhaustion() {
    let vm = WasmContractEngine::new();
    let contract = Address::new([0xbb; 20]);

    // Test Reentrancy Guard
    vm.enter_call(&contract).unwrap();
    assert_eq!(
        vm.enter_call(&contract),
        Err(ContractError::ReentrancyDetected(contract.to_string()))
    );
    vm.exit_call(&contract);
    assert!(vm.enter_call(&contract).is_ok());

    // Test Gas Exhaustion
    let mut gas = GasMeter::new(100);
    assert!(gas.consume(50).is_ok());
    assert_eq!(
        gas.consume(60),
        Err(ContractError::OutOfGas {
            limit: 100,
            consumed: 110
        })
    );
}
