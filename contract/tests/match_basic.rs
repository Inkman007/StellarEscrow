//! Unit test verifying get_trade returns exact stake_amount and token from create_trade
//!
//! Task #127: Ensures data integrity for trade/match queries.

#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, SymbolKey},
    token, Address, Env, Symbol,
};
use stellar_escrow_contract::{
    client::StellarEscrowContractClient,
    storage,
    types::{OptionalMetadata, TradeStatus},
    ContractError, StellarEscrowContract,
};

/// Test specific stake amount and token are preserved round-trip.
#[test]
fn test_get_trade_returns_correct_stake_amount_and_token() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup: deploy contract, register USDC token
    let admin = Address::generate(&env);
    let token_addr = env.register_stellar_asset_contract_v2(admin.clone());
    token::Client::new(&env, &token_addr).mint(&admin, &10_000_000_000i128); // Mint test USDC

    let contract_id = env.register_contract(None, StellarEscrowContract);
    let client = StellarEscrowContractClient::new(&env, &contract_id);

    // Initialize with specific token (our test USDC)
    client.initialize(&admin, &token_addr, &100u32); // 1% fee

    // Test data: specific stake_amount and token
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let expected_stake_amount: u64 = 1_234_567_890; // Distinct test value
    let expected_token = token_addr.clone();

    // Create trade (requires seller auth, mocked)
    let trade_id = client.with_create_trade(&seller, &buyer, &expected_stake_amount, &None::<Address>, &OptionalMetadata::None);

    // Retrieve and assert exact values match inputs
    let retrieved_trade = client.get_trade(&trade_id);

    assert_eq!(retrieved_trade.amount, expected_stake_amount, "stake_amount mismatch");
    assert_eq!(retrieved_trade.currency, expected_token, "token mismatch");
    assert_eq!(retrieved_trade.status, TradeStatus::Created, "status should be Created");
    assert_eq!(retrieved_trade.seller, seller, "seller mismatch");
    assert_eq!(retrieved_trade.buyer, buyer, "buyer mismatch");

    // Verify via raw storage (double-check integrity)
    let raw_trade = storage::get_trade(&env, trade_id).expect("storage get failed");
    assert_eq!(raw_trade.amount, expected_stake_amount);
    assert_eq!(raw_trade.currency, expected_token);
}

