#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, BytesN, Env, Vec,
};

mod test;

/// Error codes for the multisig account contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MultisigError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    InvalidThreshold = 3,
    UnknownSigner = 4,
    DuplicateSigner = 5,
    BadSignature = 6,
    ThresholdNotMet = 7,
    SignaturesNotSorted = 8,
}

/// A signature from one signer
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignerSig {
    pub public_key: BytesN<32>,
    pub signature: BytesN<64>,
}

/// Storage keys
#[contracttype]
#[derive(Clone)]
enum DataKey {
    Signers,
    Threshold,
    Initialized,
}

/// Events emitted by the contract
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MultisigEvent {
    Initialized,
    SignerAdded,
    SignerRemoved,
    ThresholdChanged,
}

#[contract]
pub struct MultisigAccount;

#[contractimpl]
impl MultisigAccount {
    /// Initialize the multisig account with signers and threshold
    /// Can only be called once
    pub fn init(env: Env, signers: Vec<BytesN<32>>, threshold: u32) -> Result<(), MultisigError> {
        // Check if already initialized
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(MultisigError::AlreadyInitialized);
        }

        let signer_count = signers.len();

        // Validate threshold
        if threshold == 0 || threshold > signer_count {
            return Err(MultisigError::InvalidThreshold);
        }

        // Check for duplicate signers (manual sort since Vec doesn't have sort in 20.5.0)
        let mut sorted_signers = signers.clone();
        // Bubble sort for simplicity
        for i in 0..sorted_signers.len() {
            for j in (i + 1)..sorted_signers.len() {
                let a = sorted_signers.get(i).unwrap();
                let b = sorted_signers.get(j).unwrap();
                if a > b {
                    sorted_signers.set(i, b.clone());
                    sorted_signers.set(j, a);
                }
            }
        }
        
        for i in 1..sorted_signers.len() {
            if sorted_signers.get(i - 1).unwrap() == sorted_signers.get(i).unwrap() {
                return Err(MultisigError::DuplicateSigner);
            }
        }

        // Store signers and threshold
        env.storage().instance().set(&DataKey::Signers, &sorted_signers);
        env.storage().instance().set(&DataKey::Threshold, &threshold);
        env.storage().instance().set(&DataKey::Initialized, &true);

        // Emit event
        env.events()
            .publish((MultisigEvent::Initialized,), (threshold, signer_count));

        Ok(())
    }

    /// Add a new signer (requires multisig authorization)
    pub fn add_signer(env: Env, signer: BytesN<32>) -> Result<(), MultisigError> {
        // Require authorization from this contract (multisig)
        env.current_contract_address().require_auth();

        let signers: Vec<BytesN<32>> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(MultisigError::NotInitialized)?;

        // Check if signer already exists
        if signers.contains(&signer) {
            return Err(MultisigError::DuplicateSigner);
        }

        let mut new_signers = signers.clone();
        new_signers.push_back(signer.clone());
        
        // Sort the signers manually
        for i in 0..new_signers.len() {
            for j in (i + 1)..new_signers.len() {
                let a = new_signers.get(i).unwrap();
                let b = new_signers.get(j).unwrap();
                if a > b {
                    new_signers.set(i, b.clone());
                    new_signers.set(j, a);
                }
            }
        }

        env.storage().instance().set(&DataKey::Signers, &new_signers);

        env.events().publish((MultisigEvent::SignerAdded,), signer);

        Ok(())
    }

    /// Remove a signer (requires multisig authorization)
    pub fn remove_signer(env: Env, signer: BytesN<32>) -> Result<(), MultisigError> {
        // Require authorization from this contract (multisig)
        env.current_contract_address().require_auth();

        let signers: Vec<BytesN<32>> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(MultisigError::NotInitialized)?;

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .ok_or(MultisigError::NotInitialized)?;

        // Find and remove the signer
        let mut found = false;
        let mut new_signers = vec![&env];
        for s in signers.iter() {
            if s == signer {
                found = true;
            } else {
                new_signers.push_back(s);
            }
        }

        if !found {
            return Err(MultisigError::UnknownSigner);
        }

        // Ensure threshold is still valid after removal
        if threshold > new_signers.len() {
            return Err(MultisigError::InvalidThreshold);
        }

        env.storage().instance().set(&DataKey::Signers, &new_signers);

        env.events()
            .publish((MultisigEvent::SignerRemoved,), signer);

        Ok(())
    }

    /// Set a new threshold (requires multisig authorization)
    pub fn set_threshold(env: Env, threshold: u32) -> Result<(), MultisigError> {
        // Require authorization from this contract (multisig)
        env.current_contract_address().require_auth();

        let signers: Vec<BytesN<32>> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(MultisigError::NotInitialized)?;

        // Validate new threshold
        if threshold == 0 || threshold > signers.len() {
            return Err(MultisigError::InvalidThreshold);
        }

        env.storage().instance().set(&DataKey::Threshold, &threshold);

        env.events()
            .publish((MultisigEvent::ThresholdChanged,), threshold);

        Ok(())
    }

    /// Get the list of signers
    pub fn get_signers(env: Env) -> Result<Vec<BytesN<32>>, MultisigError> {
        env.storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(MultisigError::NotInitialized)
    }

    /// Get the current threshold
    pub fn get_threshold(env: Env) -> Result<u32, MultisigError> {
        env.storage()
            .instance()
            .get(&DataKey::Threshold)
            .ok_or(MultisigError::NotInitialized)
    }

    /// Custom account authentication check
    /// This is called by the Soroban host when require_auth() is invoked
    pub fn __check_auth(
        env: Env,
        signature_payload: BytesN<32>,
        signatures: Vec<SignerSig>,
        _auth_contexts: Vec<soroban_sdk::Val>,
    ) -> Result<(), MultisigError> {
        let signers: Vec<BytesN<32>> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(MultisigError::NotInitialized)?;

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .ok_or(MultisigError::NotInitialized)?;

        // Check signatures are sorted and count valid signatures
        let mut valid_count: u32 = 0;
        let mut prev_key: Option<BytesN<32>> = None;

        for sig in signatures.iter() {
            // Check for proper sorting and no duplicates
            if let Some(ref prev) = prev_key {
                if sig.public_key <= *prev {
                    return Err(MultisigError::SignaturesNotSorted);
                }
            }
            prev_key = Some(sig.public_key.clone());

            // Check if this public key is a registered signer
            if !signers.contains(&sig.public_key) {
                return Err(MultisigError::UnknownSigner);
            }

            // Verify the signature
            // Note: In tests, ed25519_verify may fail if not using proper test keys
            // TODO: Add proper cryptographic test infrastructure
            env.crypto().ed25519_verify(
                &sig.public_key,
                &signature_payload.clone().into(),
                &sig.signature,
            );

            valid_count += 1;
        }

        // Check if threshold is met
        if valid_count < threshold {
            return Err(MultisigError::ThresholdNotMet);
        }

        Ok(())
    }
}
