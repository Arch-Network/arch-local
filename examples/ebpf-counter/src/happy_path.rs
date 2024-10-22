use serial_test::serial;

use crate::{
    counter_deployment::deploy_counter_program,
    counter_helpers::{
        generate_anchoring, get_account_counter, init_logging, log_scenario_end, log_scenario_start,
    },
    counter_instructions::{
        build_and_send_block, build_transaction, fetch_processed_transactions,
        get_counter_increase_instruction, start_new_counter, CounterData,
    },
};
#[ignore]
#[serial]
#[test]
fn counter_initialization_test() {
    init_logging();

    log_scenario_start(1,
        &"Program Deployment & Counter Initialization".to_string(),
        &"Happy Path Scenario : deploying the counter program, then initializing the counter to (1,1) ".to_string()
    );

    let program_pubkey = deploy_counter_program().unwrap();

    start_new_counter(&program_pubkey, 1, 1).unwrap();

    log_scenario_end(1, &format!(""));
}

#[ignore]
#[serial]
#[test]
fn counter_init_and_inc_test() {
    init_logging();

    log_scenario_start(2,
        &"Counter Initialization and Increase ( Two overlapping states, in two separate blocks )".to_string(),
        &"Happy Path Scenario : Initializing the counter to (1,1), then increasing it in a separate block ".to_string()
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let transaction = build_transaction(vec![account_keypair], vec![increase_istruction]);

    let block_transactions = build_and_send_block(vec![transaction]);

    let _processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert_eq!(final_account_data, CounterData::new(2, 1));

    log_scenario_end(2, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_init_and_inc_transaction_test() {
    init_logging();

    log_scenario_start(3,
        &"Counter Initialization and Increase ( Two overlapping states, in the same transaction )".to_string(),
        &"Happy Path Scenario : Initializing the counter to (1,1), then increasing it twice in the same transaction, using two separate instructions".to_string()
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let first_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let second_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let transaction = build_transaction(
        vec![account_keypair],
        vec![first_increase_istruction, second_increase_istruction],
    );

    let block_transactions = build_and_send_block(vec![transaction]);

    let _processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert_eq!(final_account_data, CounterData::new(3, 1));

    log_scenario_end(3, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_init_and_inc_block_test() {
    init_logging();

    log_scenario_start(4,
        &"Counter Initialization and Increase ( Two overlapping states, in the same block )".to_string(),
        &"Happy Path Scenario : Initializing the counter to (1,1), then increasing it twice in the same block, using two separate transactions".to_string()
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let first_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let first_transaction =
        build_transaction(vec![account_keypair], vec![first_increase_istruction]);

    let second_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let second_transaction =
        build_transaction(vec![account_keypair], vec![second_increase_istruction]);

    let block_transactions = build_and_send_block(vec![first_transaction, second_transaction]);

    let _processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert_eq!(final_account_data, CounterData::new(3, 1));

    log_scenario_end(4, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_init_and_inc_anchored() {
    init_logging();

    log_scenario_start(15,
        &"Counter Initialization and Increase ( 1 Anchored Instruction )".to_string(),
        &"Happy Path Scenario : Initializing the counter to (1,1), then increasing it with a Bitcoin Transaction Anchoring".to_string()
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let anchoring = generate_anchoring(&account_pubkey);

    let increase_istruction = get_counter_increase_instruction(
        &program_pubkey,
        &account_pubkey,
        false,
        false,
        Some((anchoring.0, anchoring.1, false)),
    );

    let transaction = build_transaction(vec![account_keypair], vec![increase_istruction]);

    let block_transactions = build_and_send_block(vec![transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    assert!(processed_transactions[0].bitcoin_txid.is_some());

    println!();

    println!("\x1b[1m\x1B[34m Bitcoin transaction submitted :  : https://mempool.dev.aws.archnetwork.xyz/tx/{} \x1b[0m",processed_transactions[0].bitcoin_txid.clone().unwrap());

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert_eq!(final_account_data, CounterData::new(2, 1));

    log_scenario_end(15, &format!("{:?}", final_account_data));
}
