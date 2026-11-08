#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, IntoVal, Symbol, Vec,
};

mod test;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FactoryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidThreshold = 3,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    AccountWasmHash,
    AccountCount,
    CreatorAccounts(Address), // Maps creator address to their accounts
    Initialized,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FactoryEvent {
    Initialized,
    AccountCreated,
}

#[contract]
pub struct AccountFactory;

#[contractimpl]
impl AccountFactory {
    /// Initialize the factory with admin and account contract WASM hash
    pub fn initialize(
        env: Env,
        admin: Address,
        account_wasm_hash: BytesN<32>,
    ) -> Result<(), FactoryError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(FactoryError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::AccountWasmHash, &account_wasm_hash);
        env.storage().instance().set(&DataKey::AccountCount, &0u64);
        env.storage().instance().set(&DataKey::Initialized, &true);

        env.events()
            .publish((FactoryEvent::Initialized,), (admin, account_wasm_hash));

        Ok(())
    }

    /// Create a new multisig account
    pub fn create_account(
        env: Env,
        creator: Address,
        salt: BytesN<32>,
        signers: Vec<BytesN<32>>,
        threshold: u32,
    ) -> Result<Address, FactoryError> {
        creator.require_auth();

        let account_wasm_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::AccountWasmHash)
            .ok_or(FactoryError::NotInitialized)?;

        // Validate threshold
        let signer_count = signers.len();
        if threshold == 0 || threshold > signer_count {
            return Err(FactoryError::InvalidThreshold);
        }

        // Deploy the account contract
        let deployed_address = env
            .deployer()
            .with_current_contract(salt)
            .deploy(account_wasm_hash);

        // Initialize the deployed account
        env.invoke_contract::<()>(
            &deployed_address,
            &Symbol::new(&env, "init"),
            (signers.clone(), threshold).into_val(&env),
        );

        // Track the account
        let mut creator_accounts: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::CreatorAccounts(creator.clone()))
            .unwrap_or(soroban_sdk::vec![&env]);

        creator_accounts.push_back(deployed_address.clone());

        env.storage()
            .instance()
            .set(&DataKey::CreatorAccounts(creator.clone()), &creator_accounts);

        // Increment account count
        let mut count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AccountCount)
            .unwrap_or(0);
        count += 1;
        env.storage().instance().set(&DataKey::AccountCount, &count);

        env.events().publish(
            (FactoryEvent::AccountCreated,),
            (creator, deployed_address.clone(), threshold, signer_count),
        );

        Ok(deployed_address)
    }

    /// Get all accounts created by a specific creator
    pub fn get_accounts_by_creator(env: Env, creator: Address) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::CreatorAccounts(creator))
            .unwrap_or(soroban_sdk::vec![&env])
    }

    /// Get total number of accounts created
    pub fn account_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AccountCount)
            .unwrap_or(0)
    }

    /// Get the admin address
    pub fn get_admin(env: Env) -> Result<Address, FactoryError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(FactoryError::NotInitialized)
    }

    /// Get the account WASM hash
    pub fn get_account_wasm_hash(env: Env) -> Result<BytesN<32>, FactoryError> {
        env.storage()
            .instance()
            .get(&DataKey::AccountWasmHash)
            .ok_or(FactoryError::NotInitialized)
    }
}
