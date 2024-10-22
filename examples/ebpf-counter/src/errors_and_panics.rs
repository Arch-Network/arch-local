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
use common::{constants::NODE1_ADDRESS, helper::read_account_info};
use sdk::processed_transaction::Status;
use serial_test::serial;

#[ignore]
#[serial]
#[test]
fn counter_inc_single_instruction_fail() {
    init_logging();

    log_scenario_start(5,
        "Counter Initialization and Increase Failure ( One Instruction to Increase should fail )",
        "Initializing the counter to (1,1), then increasing it in a single instruction, the state shouldn't be updated"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, true, false, None);

    let increase_transaction = build_transaction(vec![account_keypair], vec![increase_istruction]);

    let block_transactions = build_and_send_block(vec![increase_transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed { .. }
    ));

    assert_eq!(final_account_data, CounterData::new(1, 1));

    log_scenario_end(5, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_inc_single_instruction_panic() {
    init_logging();

    log_scenario_start(6,
        "Counter Initialization and Increase Failure ( One Instruction to Increase should panic )",
        "Initializing the counter to (1,1), then increasing it in a single instruction, the state shouldn't be updated"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, true, None);

    let increase_transaction = build_transaction(vec![account_keypair], vec![increase_istruction]);

    let block_transactions = build_and_send_block(vec![increase_transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed { .. }
    ));

    assert_eq!(final_account_data, CounterData::new(1, 1));

    log_scenario_end(6, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_inc_two_instructions_1st_fail() {
    init_logging();

    log_scenario_start(7,
        "Counter Initialization and Increase Failure ( Two Instructions to Increase, first instruction should fail )",
        "Initializing the counter to (1,1), then increasing it twice within the same transaction, with the first instruction failing. The state shouldn't be updated"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let first_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, true, false, None);

    let second_increase_instruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let increase_transaction = build_transaction(
        vec![account_keypair],
        vec![first_increase_istruction, second_increase_instruction],
    );

    let block_transactions = build_and_send_block(vec![increase_transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed { .. }
    ));

    assert_eq!(final_account_data, CounterData::new(1, 1));

    log_scenario_end(7, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_inc_two_instructions_2nd_fail() {
    init_logging();

    log_scenario_start(8,
        "Counter Initialization and Increase Failure ( Two Instructions to Increase, second instruction should fail )",
        "Initializing the counter to (1,1), then increasing it twice within the same transaction, with the first instruction failing. The state shouldn't be updated"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let first_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let second_increase_instruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, true, false, None);

    let increase_transaction = build_transaction(
        vec![account_keypair],
        vec![first_increase_istruction, second_increase_instruction],
    );

    let block_transactions = build_and_send_block(vec![increase_transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed { .. }
    ));

    assert_eq!(final_account_data, CounterData::new(1, 1));

    log_scenario_end(8, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_inc_two_instructions_1st_panic() {
    init_logging();

    log_scenario_start(9,
        "Counter Initialization and Increase Failure ( Two Instructions to Increase, first instruction should panic )",
        "Initializing the counter to (1,1), then increasing it twice within the same transaction, with the first instruction panicking. The state shouldn't be updated"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let first_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, true, None);

    let second_increase_instruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let increase_transaction = build_transaction(
        vec![account_keypair],
        vec![first_increase_istruction, second_increase_instruction],
    );

    let block_transactions = build_and_send_block(vec![increase_transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed { .. }
    ));

    assert_eq!(final_account_data, CounterData::new(1, 1));

    log_scenario_end(9, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_inc_two_instructions_2nd_panic() {
    init_logging();

    log_scenario_start(10,
        "Counter Initialization and Increase Failure ( Two Instructions to Increase, second instruction should panic )",
        "Initializing the counter to (1,1), then increasing it twice within the same transaction, with the first instruction panicking. The state shouldn't be updated"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let first_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let second_increase_instruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, true, None);

    let increase_transaction = build_transaction(
        vec![account_keypair],
        vec![first_increase_istruction, second_increase_instruction],
    );

    let block_transactions = build_and_send_block(vec![increase_transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed { .. }
    ));

    assert_eq!(final_account_data, CounterData::new(1, 1));

    log_scenario_end(10, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_inc_two_transactions_1st_fail() {
    init_logging();

    log_scenario_start(13,
        "Counter Initialization and Increase Failure ( Two separate transactions to Increase, first transaction should fail )",
        "Initializing the counter to (1,1), then increasing it twice in two separate transactions, with the first transaction failing. The state should be updated by 2nd transaction"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let first_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, true, false, None);

    let second_increase_instruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let first_increase_transaction =
        build_transaction(vec![account_keypair], vec![first_increase_istruction]);

    let second_increase_transaction =
        build_transaction(vec![account_keypair], vec![second_increase_instruction]);

    let block_transactions = build_and_send_block(vec![
        first_increase_transaction,
        second_increase_transaction,
    ]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed { .. }
    ));

    assert!(matches!(
        processed_transactions[1].status,
        Status::Processed
    ));

    assert_eq!(final_account_data, CounterData::new(2, 1));

    log_scenario_end(11, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_inc_two_transactions_2nd_fail() {
    init_logging();

    log_scenario_start(13,
        "Counter Initialization and Increase Failure ( Two separate transactions to Increase, second transaction should fail )",
        "Initializing the counter to (1,1), then increasing it twice in two separate transactions, with the second transaction failing. The state should be updated by 1st transaction"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let first_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let second_increase_instruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, true, false, None);

    let first_increase_transaction =
        build_transaction(vec![account_keypair], vec![first_increase_istruction]);

    let second_increase_transaction =
        build_transaction(vec![account_keypair], vec![second_increase_instruction]);

    let block_transactions = build_and_send_block(vec![
        first_increase_transaction,
        second_increase_transaction,
    ]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert!(matches!(
        processed_transactions[1].status,
        Status::Failed { .. }
    ));

    assert!(matches!(
        processed_transactions[0].status,
        Status::Processed
    ));

    assert_eq!(final_account_data, CounterData::new(2, 1));

    log_scenario_end(12, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_inc_two_transactions_1st_panic() {
    init_logging();

    log_scenario_start(13,
        "Counter Initialization and Increase Failure ( Two separate transactions to Increase, first transaction should panic )",
        "Initializing the counter to (1,1), then increasing it twice in two separate transactions, with the first transaction panicking. The state should be updated by 2nd transaction"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let first_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, true, None);

    let second_increase_instruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let first_increase_transaction =
        build_transaction(vec![account_keypair], vec![first_increase_istruction]);

    let second_increase_transaction =
        build_transaction(vec![account_keypair], vec![second_increase_instruction]);

    let block_transactions = build_and_send_block(vec![
        first_increase_transaction,
        second_increase_transaction,
    ]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed { .. }
    ));

    assert!(matches!(
        processed_transactions[1].status,
        Status::Processed
    ));

    assert_eq!(final_account_data, CounterData::new(2, 1));

    log_scenario_end(13, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_inc_two_transactions_2nd_panic() {
    init_logging();

    log_scenario_start(14,
        "Counter Initialization and Increase Failure ( Two separate transactions to Increase, second transaction should panic )",
        "Initializing the counter to (1,1), then increasing it twice in two separate transactions, with the first transaction panicking. The state should be updated by 1st transaction"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let first_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let second_increase_instruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, true, None);

    let first_increase_transaction =
        build_transaction(vec![account_keypair], vec![first_increase_istruction]);

    let second_increase_transaction =
        build_transaction(vec![account_keypair], vec![second_increase_instruction]);

    let block_transactions = build_and_send_block(vec![
        first_increase_transaction,
        second_increase_transaction,
    ]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert!(matches!(
        processed_transactions[1].status,
        Status::Failed { .. }
    ));

    assert!(matches!(
        processed_transactions[0].status,
        Status::Processed
    ));

    assert_eq!(final_account_data, CounterData::new(2, 1));

    log_scenario_end(14, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_init_and_inc_anchored_fail() {
    init_logging();

    log_scenario_start(16,
        "Counter Initialization and Increase ( 1 Anchored Instruction )",
        "Happy Path Scenario : Initializing the counter to (1,1), then increasing it with a Bitcoin Transaction Anchoring, the BTC anchoring should fail, and the state shouldn't change"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let anchoring = generate_anchoring(&account_pubkey);

    let increase_istruction = get_counter_increase_instruction(
        &program_pubkey,
        &account_pubkey,
        false,
        false,
        Some((anchoring.0, anchoring.1, true)),
    );

    let transaction = build_transaction(vec![account_keypair], vec![increase_istruction]);

    let block_transactions = build_and_send_block(vec![transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    //assert!(processed_transactions[0].bitcoin_txid.is_none());

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed(_)
    ));

    println!();

    println!("\x1b[1m\x1B[34m Bitcoin transaction failed !");

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert_eq!(final_account_data, CounterData::new(1, 1));

    log_scenario_end(16, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_init_and_inc_anchored_fail_inc_state() {
    init_logging();

    log_scenario_start(17,
        "Counter Initialization and Increase (  1 Anchored Instruction, 1 State only )",
        "Happy Path Scenario : Initializing the counter to (1,1), then increasing it with a Bitcoin Transaction Anchoring, the BTC anchoring should fail, the second instruction should be rolled back, and the state shouldn't change"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let anchoring = generate_anchoring(&account_pubkey);

    let first_increase_istruction = get_counter_increase_instruction(
        &program_pubkey,
        &account_pubkey,
        false,
        false,
        Some((anchoring.0, anchoring.1, true)),
    );

    let second_increase_instruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let transaction = build_transaction(
        vec![account_keypair],
        vec![first_increase_istruction, second_increase_instruction],
    );

    let block_transactions = build_and_send_block(vec![transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    //assert!(processed_transactions[0].bitcoin_txid.is_none());

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed(_)
    ));

    println!();

    println!("\x1b[1m\x1B[34m Bitcoin transaction failed !");

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert_eq!(final_account_data, CounterData::new(1, 1));

    log_scenario_end(17, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_init_and_two_inc_anchored_fail() {
    init_logging();

    log_scenario_start(18,
        "Counter Initialization and Increase ( 1 Anchored Instruction, 1 State only Instruction )",
        "Happy Path Scenario : Initializing the counter to (1,1), then increasing it with a failing Bitcoin Transaction Anchoring, and a succeeding state only instruction, the entire Runtime transaction and the state shouldn't change"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let anchoring = generate_anchoring(&account_pubkey);

    let anchoring_2 = generate_anchoring(&account_pubkey);

    let first_increase_istruction = get_counter_increase_instruction(
        &program_pubkey,
        &account_pubkey,
        false,
        false,
        Some((anchoring.0, anchoring.1, true)),
    );

    let second_increase_instruction = get_counter_increase_instruction(
        &program_pubkey,
        &account_pubkey,
        false,
        false,
        Some((anchoring_2.0, anchoring_2.1, false)),
    );

    let transaction = build_transaction(
        vec![account_keypair],
        vec![first_increase_istruction, second_increase_instruction],
    );

    let block_transactions = build_and_send_block(vec![transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    //assert!(processed_transactions[0].bitcoin_txid.is_none());

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed(_)
    ));

    println!();

    println!("\x1b[1m\x1B[34m Bitcoin transaction failed !");

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert_eq!(final_account_data, CounterData::new(1, 1));

    log_scenario_end(18, &format!("{:?}", final_account_data));
}

#[ignore]
#[serial]
#[test]
fn counter_init_and_two_inc_tx_anchored_fail() {
    init_logging();

    log_scenario_start(19,
        "Counter Initialization and Increase ( 1 Anchored transaction signaled to fail, 1 Anchored Transaction signaled to succeed )",
        "Happy Path Scenario : Initializing the counter to (1,1), then increasing it with a Bitcoin Transaction Anchoring, the BTC anchoring should fail, and the state shouldn't change, the second transaction will also try to change the state with an anchoring it should fail"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let account_info = read_account_info(NODE1_ADDRESS, account_pubkey).unwrap();

    let utxo_before_block = account_info.utxo.clone();

    let anchoring = generate_anchoring(&account_pubkey);

    let anchoring_2 = generate_anchoring(&account_pubkey);

    let first_increase_istruction = get_counter_increase_instruction(
        &program_pubkey,
        &account_pubkey,
        false,
        false,
        Some((anchoring.0, anchoring.1, true)),
    );

    let second_increase_instruction = get_counter_increase_instruction(
        &program_pubkey,
        &account_pubkey,
        false,
        false,
        Some((anchoring_2.0, anchoring_2.1, false)),
    );

    let first_transaction =
        build_transaction(vec![account_keypair], vec![first_increase_istruction]);

    let second_transaction =
        build_transaction(vec![account_keypair], vec![second_increase_instruction]);

    let block_transactions = build_and_send_block(vec![first_transaction, second_transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let account_info = read_account_info(NODE1_ADDRESS, account_pubkey).unwrap();

    let utxo_after_block = account_info.utxo.clone();

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed(_)
    ));

    println!(
        "1st BTC transaction Result : {:?}",
        processed_transactions[0].bitcoin_txid.clone()
    );

    assert!(processed_transactions[1].bitcoin_txid.is_none());

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed(_)
    ));

    assert!(matches!(
        processed_transactions[1].status,
        Status::Failed(_)
    ));

    //rpc.get_raw_transaction(&tx.txid(), None);

    assert_eq!(utxo_after_block, utxo_before_block);

    println!();

    println!("\x1b[1m\x1B[34m Both Bitcoin transactions failed !");

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    println!("Account data {:?}", final_account_data);

    assert_eq!(final_account_data, CounterData::new(1, 1));

    log_scenario_end(19, &format!("{:?}", final_account_data));
}
#[ignore]
#[serial]
#[test]
fn counter_init_and_two_inc_second_anchored_fail() {
    init_logging();

    log_scenario_start(20,
        "Counter Initialization and Increase (  1 State only Instruction succeeding,1 Anchored Instruction failing)",
        "Happy Path Scenario : Initializing the counter to (1,1), then increasing it with a succeeding state only instruction, and a failing anchored instruction, the entire Runtime transaction and the state shouldn't change"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (account_pubkey, account_keypair) = start_new_counter(&program_pubkey, 1, 1).unwrap();

    let account_info = read_account_info(NODE1_ADDRESS, account_pubkey).unwrap();

    let utxo_before_block = account_info.utxo.clone();

    let anchoring = generate_anchoring(&account_pubkey);

    let first_increase_istruction =
        get_counter_increase_instruction(&program_pubkey, &account_pubkey, false, false, None);

    let second_increase_instruction = get_counter_increase_instruction(
        &program_pubkey,
        &account_pubkey,
        false,
        false,
        Some((anchoring.0, anchoring.1, true)),
    );

    let transaction = build_transaction(
        vec![account_keypair],
        vec![first_increase_istruction, second_increase_instruction],
    );

    let block_transactions = build_and_send_block(vec![transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    //assert!(processed_transactions[0].bitcoin_txid.is_none());

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed(_)
    ));

    println!();

    println!("\x1b[1m\x1B[34m Bitcoin transaction failed !");

    let final_account_data = get_account_counter(&account_pubkey).unwrap();

    assert_eq!(final_account_data, CounterData::new(1, 1));

    let account_info = read_account_info(NODE1_ADDRESS, account_pubkey).unwrap();

    let utxo_after_block = account_info.utxo.clone();

    assert_eq!(utxo_after_block, utxo_before_block);

    log_scenario_end(20, &format!("{:?}", final_account_data));
}
#[ignore]
#[serial]
#[test]
fn counter_init_and_two_inc_tx_anchored_fail_2nd_succeed() {
    init_logging();

    log_scenario_start(21,
        "Counter Initialization and Increase ( 1 Anchored transaction signaled to fail, 1 Anchored Transaction signaled to succeed (TWO DIFFERENT STATE ACCOUNTS) )",
        "Happy Path Scenario : Initializing the counter to (1,1), then increasing it with a Bitcoin Transaction Anchoring, the BTC anchoring should fail, and the state shouldn't change. The second transaction will try to change another state with an anchoring it should succeed"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (first_account_pubkey, first_account_keypair) =
        start_new_counter(&program_pubkey, 1, 1).unwrap();

    let (second_account_pubkey, second_account_keypair) =
        start_new_counter(&program_pubkey, 1, 1).unwrap();

    let first_account_info = read_account_info(NODE1_ADDRESS, first_account_pubkey).unwrap();

    let second_account_info = read_account_info(NODE1_ADDRESS, second_account_pubkey).unwrap();

    let first_utxo_before_block = first_account_info.utxo.clone();

    let second_utxo_before_block = second_account_info.utxo.clone();

    let first_anchoring = generate_anchoring(&first_account_pubkey);

    let second_anchoring = generate_anchoring(&second_account_pubkey);

    let first_increase_istruction = get_counter_increase_instruction(
        &program_pubkey,
        &first_account_pubkey,
        false,
        false,
        Some((first_anchoring.0, first_anchoring.1, true)),
    );

    let second_increase_instruction = get_counter_increase_instruction(
        &program_pubkey,
        &second_account_pubkey,
        false,
        false,
        Some((second_anchoring.0, second_anchoring.1, false)),
    );

    let first_transaction =
        build_transaction(vec![first_account_keypair], vec![first_increase_istruction]);

    let second_transaction = build_transaction(
        vec![second_account_keypair],
        vec![second_increase_instruction],
    );

    let block_transactions = build_and_send_block(vec![first_transaction, second_transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let first_account_info = read_account_info(NODE1_ADDRESS, first_account_pubkey).unwrap();

    let second_account_info = read_account_info(NODE1_ADDRESS, second_account_pubkey).unwrap();

    let first_utxo_after_block = first_account_info.utxo.clone();

    let second_utxo_after_block = second_account_info.utxo.clone();

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed(_)
    ));

    assert!(processed_transactions[1].bitcoin_txid.is_some());

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed(_)
    ));

    assert!(matches!(
        processed_transactions[1].status,
        Status::Processed
    ));

    //rpc.get_raw_transaction(&tx.txid(), None);

    assert_eq!(first_utxo_after_block, first_utxo_before_block);

    assert_ne!(second_utxo_after_block, second_utxo_before_block);
    println!();

    println!("\x1b[1m\x1B[34m Both Bitcoin transactions failed !");

    let final_first_account_data = get_account_counter(&first_account_pubkey).unwrap();

    println!("First Account data {:?}", final_first_account_data);

    assert_eq!(final_first_account_data, CounterData::new(1, 1));

    let final_second_account_data = get_account_counter(&second_account_pubkey).unwrap();

    println!("First Account data {:?}", final_second_account_data);

    assert_eq!(final_second_account_data, CounterData::new(2, 1));

    log_scenario_end(
        21,
        &format!(
            "{:?} === {:?}",
            final_first_account_data, final_second_account_data
        ),
    );
}
#[ignore]
#[serial]
#[test]
fn counter_init_and_two_inc_tx_anchored_fail_2nd_state_only_succeed() {
    init_logging();

    log_scenario_start(22,
        "Counter Initialization and Increase ( 1 Anchored transaction signaled to fail, 1 state only Transaction signaled to succeed (TWO DIFFERENT STATE ACCOUNTS) )",
        "Happy Path Scenario : Initializing the counter to (1,1), then increasing it with a Bitcoin Transaction Anchoring, the BTC anchoring should fail, and the state shouldn't change. The second transaction will try to change another state without an anchoring it should succeed"
    );

    let program_pubkey = deploy_counter_program().unwrap();

    let (first_account_pubkey, first_account_keypair) =
        start_new_counter(&program_pubkey, 1, 1).unwrap();

    let (second_account_pubkey, second_account_keypair) =
        start_new_counter(&program_pubkey, 1, 1).unwrap();

    let first_account_info = read_account_info(NODE1_ADDRESS, first_account_pubkey).unwrap();

    let second_account_info = read_account_info(NODE1_ADDRESS, second_account_pubkey).unwrap();

    let first_utxo_before_block = first_account_info.utxo.clone();

    let second_utxo_before_block = second_account_info.utxo.clone();

    let first_anchoring = generate_anchoring(&first_account_pubkey);

    let first_increase_istruction = get_counter_increase_instruction(
        &program_pubkey,
        &first_account_pubkey,
        false,
        false,
        Some((first_anchoring.0, first_anchoring.1, true)),
    );

    let second_increase_instruction = get_counter_increase_instruction(
        &program_pubkey,
        &second_account_pubkey,
        false,
        false,
        None,
    );

    let first_transaction =
        build_transaction(vec![first_account_keypair], vec![first_increase_istruction]);

    let second_transaction = build_transaction(
        vec![second_account_keypair],
        vec![second_increase_instruction],
    );

    let block_transactions = build_and_send_block(vec![first_transaction, second_transaction]);

    let processed_transactions = fetch_processed_transactions(block_transactions).unwrap();

    let first_account_info = read_account_info(NODE1_ADDRESS, first_account_pubkey).unwrap();

    let second_account_info = read_account_info(NODE1_ADDRESS, second_account_pubkey).unwrap();

    let first_utxo_after_block = first_account_info.utxo.clone();

    let second_utxo_after_block = second_account_info.utxo.clone();

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed(_)
    ));

    assert!(processed_transactions[1].bitcoin_txid.is_none());

    assert!(matches!(
        processed_transactions[0].status,
        Status::Failed(_)
    ));

    assert!(matches!(
        processed_transactions[1].status,
        Status::Processed
    ));

    //rpc.get_raw_transaction(&tx.txid(), None);

    assert_eq!(first_utxo_after_block, first_utxo_before_block);

    assert_eq!(second_utxo_after_block, second_utxo_before_block);
    println!();

    println!("\x1b[1m\x1B[34m Both Bitcoin transactions failed !");

    let final_first_account_data = get_account_counter(&first_account_pubkey).unwrap();

    println!("First Account data {:?}", final_first_account_data);

    assert_eq!(final_first_account_data, CounterData::new(1, 1));

    let final_second_account_data = get_account_counter(&second_account_pubkey).unwrap();

    println!("First Account data {:?}", final_second_account_data);

    assert_eq!(final_second_account_data, CounterData::new(2, 1));

    log_scenario_end(
        22,
        &format!(
            "{:?} === {:?}",
            final_first_account_data, final_second_account_data
        ),
    );
}
