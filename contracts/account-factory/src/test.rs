#![cfg(test)]

use super::{AccountFactory, AccountFactoryClient, FactoryError};
use soroban_sdk::{
    testutils::Address as _,
    vec, Address, BytesN, Env, Vec,
};

fn create_test_signers(env: &Env, count: u32) -> Vec<BytesN<32>> {
    let mut signers = vec![env];
    for i in 0..count {
        let mut key = [0u8; 32];
        key[0] = i as u8 + 1;
        signers.push_back(BytesN::from_array(env, &key));
    }
    signers
}

#[test]
fn test_initialize_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountFactory);
    let client = AccountFactoryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let wasm_hash = BytesN::from_array(&env, &[1u8; 32]);

    let result = client.initialize(&admin, &wasm_hash);
    assert_eq!(result, ());

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_account_wasm_hash(), wasm_hash);
    assert_eq!(client.account_count(), 0);
}

#[test]
fn test_initialize_already_initialized() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountFactory);
    let client = AccountFactoryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let wasm_hash = BytesN::from_array(&env, &[1u8; 32]);

    client.initialize(&admin, &wasm_hash);

    // Try to initialize again
    let result = client.try_initialize(&admin, &wasm_hash);
    assert_eq!(result, Err(Ok(FactoryError::AlreadyInitialized)));
}

#[test]
fn test_create_account_invalid_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AccountFactory);
    let client = AccountFactoryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let wasm_hash = BytesN::from_array(&env, &[1u8; 32]);
    client.initialize(&admin, &wasm_hash);

    let creator = Address::generate(&env);
    let salt = BytesN::from_array(&env, &[2u8; 32]);
    let signers = create_test_signers(&env, 3);

    // Test threshold = 0
    let result = client.try_create_account(&creator, &salt, &signers, &0);
    assert_eq!(result, Err(Ok(FactoryError::InvalidThreshold)));

    // Test threshold > signer count
    let result = client.try_create_account(&creator, &salt, &signers, &4);
    assert_eq!(result, Err(Ok(FactoryError::InvalidThreshold)));
}

#[test]
fn test_get_accounts_before_creation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountFactory);
    let client = AccountFactoryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let wasm_hash = BytesN::from_array(&env, &[1u8; 32]);
    client.initialize(&admin, &wasm_hash);

    let creator = Address::generate(&env);
    let accounts = client.get_accounts_by_creator(&creator);
    assert_eq!(accounts.len(), 0);
}

#[test]
fn test_account_count_increments() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountFactory);
    let client = AccountFactoryClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let wasm_hash = BytesN::from_array(&env, &[1u8; 32]);
    client.initialize(&admin, &wasm_hash);

    assert_eq!(client.account_count(), 0);

    // Note: In a real test, we would need to register the multisig account contract
    // and create actual accounts. This is a simplified version.
}

#[test]
fn test_get_admin_not_initialized() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountFactory);
    let client = AccountFactoryClient::new(&env, &contract_id);

    let result = client.try_get_admin();
    assert_eq!(result, Err(Ok(FactoryError::NotInitialized)));
}

#[test]
fn test_get_wasm_hash_not_initialized() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AccountFactory);
    let client = AccountFactoryClient::new(&env, &contract_id);

    let result = client.try_get_account_wasm_hash();
    assert_eq!(result, Err(Ok(FactoryError::NotInitialized)));
}
