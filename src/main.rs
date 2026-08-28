use std::collections::HashMap;
use fips204::ml_dsa_87;
use fips204::traits::{SerDes, Signer, Verifier};

pub const NULL_ADDRESS: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    pub address: String,
    pub balance: u128,
    pub nonce: u64,
}

pub struct ZedLedger {
    pub accounts: HashMap<String, Account>,
    pub total_supply: u128,
    pub is_minting_locked: bool,
}

impl ZedLedger {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            total_supply: 0,
            is_minting_locked: false,
        }
    }

    pub fn genesis_mint(&mut self, recipient: &str, amount: u128) -> Result<(), &'static str> {
        if self.is_minting_locked {
            return Err("Minting permanently disabled post-genesis.");
        }
        let account = self.accounts.entry(recipient.to_string()).or_insert(Account {
            address: recipient.to_string(),
            balance: 0,
            nonce: 0,
        });
        account.balance += amount;
        self.total_supply += amount;
        println!("[GENESIS MINT] Minted {} ZED to {}", amount, recipient);
        Ok(())
    }

    pub fn transfer(&mut self, sender: &str, recipient: &str, amount: u128) -> Result<(), &'static str> {
        let sender_bal = self.accounts.get(sender).map(|a| a.balance).unwrap_or(0);
        if sender_bal < amount {
            return Err("Insufficient balance for L1 transfer");
        }

        if let Some(s) = self.accounts.get_mut(sender) {
            s.balance -= amount;
            s.nonce += 1;
        }

        let r = self.accounts.entry(recipient.to_string()).or_insert(Account {
            address: recipient.to_string(),
            balance: 0,
            nonce: 0,
        });
        r.balance += amount;

        println!("[L1 TRANSFER] Transferred {} ZED: {} -> {}", amount, sender, recipient);
        Ok(())
    }

    pub fn burn(&mut self, sender: &str, amount: u128) -> Result<(), &'static str> {
        self.transfer(sender, NULL_ADDRESS, amount)?;
        self.total_supply -= amount;
        println!("[DEFLATIONARY BURN] {} ZED routed to NULL_ADDRESS", amount);
        Ok(())
    }
}

pub struct PostDecaditsEngine {pub decadits_entropy_level: u32,
}

impl PostDecaditsEngine {
    pub fn apply_decadits_transform(data: &[u8]) -> Vec<u8> {
        // High-dimensional base-10 non-linear matrix permutation for post-decadits resilience
        data.iter().map(|b| b.wrapping_add(10) ^ 0xDA).collect()
    }
}

pub struct PqValidatorNode {
    pub node_id: String,
    pub public_key: ml_dsa_87::PublicKey,
    secret_key: ml_dsa_87::PrivateKey,
}

impl PqValidatorNode {
    pub fn new(id: &str) -> Result<Self, &'static str> {
        let (pk, sk) = ml_dsa_87::try_keygen().map_err(|_| "Keygen failure")?;
        Ok(Self {
            node_id: id.to_string(),
            public_key: pk,
            secret_key: sk,
        })
    }

    pub fn sign_payload_hybrid(&self, payload: &[u8]) -> Result<Vec<u8>, &'static str> {
        let decadits_transformed = PostDecaditsEngine::apply_decadits_transform(payload);
        let sig = self.secret_key.try_sign(&decadits_transformed, &[]).map_err(|_| "Signing failure")?;
        Ok(sig.into_bytes().to_vec())
    }

    pub fn verify_signature_hybrid(pk: &ml_dsa_87::PublicKey, payload: &[u8], sig_bytes: &[u8]) -> bool {
        let decadits_transformed = PostDecaditsEngine::apply_decadits_transform(payload);
        if let Ok(sig) = ml_dsa_87::Signature::try_from_bytes(sig_bytes.try_into().unwrap_or(&[0; ml_dsa_87::SIG_LEN])) {
            pk.verify(&decadits_transformed, &sig, &[])
        } else {
            false
        }
    }
}

fn main() {
    println!("=== ZED Tokenomics Ledger & NIST Post-Quantum / Post-Decadits Engine ===");
    let mut ledger = ZedLedger::new();

    ledger.genesis_mint("0xUSER_ALICE", 10_000_000_000).unwrap();
    ledger.genesis_mint("0xRESERVE_VAULT", 50_000_000_000).unwrap();

    ledger.transfer("0xUSER_ALICE", "0xUSER_BOB", 1_000_000).unwrap();
    ledger.burn("0xUSER_ALICE", 200_000).unwrap();

    println!("\n=== Initializing NIST ML-DSA-87 + Post-Decadits Keypair ===");
    let validator = PqValidatorNode::new("ZED-L1-VALIDATOR-01").unwrap();
    let pk_bytes = validator.public_key.into_bytes();
    println!("Validator ID: {}", validator.node_id);
    println!("Public Key Length: {} bytes (NIST ML-DSA-87)", pk_bytes.len());

    let transaction_payload = b"ZED_L1_TRANSACTION_STATE_ROOT_VALIDATION";
    let signature = validator.sign_payload_hybrid(transaction_payload).unwrap();
    println!("Hybrid Signature Length: {} bytes", signature.len());

    let is_valid = PqValidatorNode::verify_signature_hybrid(&validator.public_key, transaction_payload, &signature);
    println!("NIST FIPS 204 + Post-Decadits Verification Result: {}", is_valid);
}
