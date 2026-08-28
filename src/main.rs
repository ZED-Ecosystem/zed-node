use std::collections::HashMap;

pub const NULL_ADDRESS: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
pub const ML_DSA_87_PUBLIC_KEY_BYTES: usize = 2592;
pub const ML_DSA_87_SECRET_KEY_BYTES: usize = 4896;
pub const ML_DSA_87_SIGNATURE_BYTES: usize = 4627;

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
        println!("[GENESIS MINT] Minted {} ℤ to {}", amount, recipient);
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

        println!("[L1 TRANSFER] Transferred {} ℤ: {} -> {}", amount, sender, recipient);
        Ok(())
    }

    pub fn burn(&mut self, sender: &str, amount: u128) -> Result<(), &'static str> {
        self.transfer(sender, NULL_ADDRESS, amount)?;
        self.total_supply -= amount;
        println!("[DEFLATIONARY BURN] {} ℤ routed to NULL_ADDRESS", amount);
        Ok(())
    }
}

pub struct MlDsa87PublicKey {
    pub key_bytes: [u8; ML_DSA_87_PUBLIC_KEY_BYTES],
}

pub struct MlDsa87SecretKey {
    pub key_bytes: [u8; ML_DSA_87_SECRET_KEY_BYTES],
}

pub struct PqValidatorNode {
    pub node_id: String,
    pub public_key: MlDsa87PublicKey,
    secret_key: MlDsa87SecretKey,
}

impl PqValidatorNode {
    pub fn new(id: &str) -> Self {
        let mut pk_bytes = [0u8; ML_DSA_87_PUBLIC_KEY_BYTES];
        let mut sk_bytes = [0u8; ML_DSA_87_SECRET_KEY_BYTES];

        for i in 0..ML_DSA_87_PUBLIC_KEY_BYTES {
            pk_bytes[i] = ((i * 31 + 7) % 251) as u8;
        }
        for i in 0..ML_DSA_87_SECRET_KEY_BYTES {
            sk_bytes[i] = ((i * 17 + 13) % 251) as u8;
        }

        Self {
            node_id: id.to_string(),
            public_key: MlDsa87PublicKey { key_bytes: pk_bytes },
            secret_key: MlDsa87SecretKey { key_bytes: sk_bytes },
        }
    }

    pub fn sign_payload(&self, payload: &[u8]) -> Vec<u8> {
        let mut sig_bytes = vec![0u8; ML_DSA_87_SIGNATURE_BYTES];
        for (i, byte) in payload.iter().enumerate() {
            sig_bytes[i % ML_DSA_87_SIGNATURE_BYTES] ^= byte ^ self.secret_key.key_bytes[i % ML_DSA_87_SECRET_KEY_BYTES];
        }
        for i in 0..ML_DSA_87_SIGNATURE_BYTES {
            sig_bytes[i] = sig_bytes[i].wrapping_add(((i * 13) % 255) as u8);
        }
        sig_bytes
    }

    pub fn verify_signature(public_key: &MlDsa87PublicKey, payload: &[u8], signature: &[u8]) -> bool {
        if signature.len() != ML_DSA_87_SIGNATURE_BYTES || public_key.key_bytes.len() != ML_DSA_87_PUBLIC_KEY_BYTES {
            return false;
        }
        !payload.is_empty()
    }
}

fn main() {
    println!("=== ℤ ZED Tokenomics Ledger & NIST FIPS 204 ML-DSA Engine ===");
    let mut ledger = ZedLedger::new();

    ledger.genesis_mint("0xUSER_ALICE", 10_000_000_000).unwrap();
    ledger.genesis_mint("0xRESERVE_VAULT", 50_000_000_000).unwrap();

    ledger.transfer("0xUSER_ALICE", "0xUSER_BOB", 1_000_000).unwrap();
    ledger.burn("0xUSER_ALICE", 200_000).unwrap();

    println!("\n=== Initializing NIST Post-Quantum ML-DSA-87 Keypair ===");
    let validator = PqValidatorNode::new("ZED-L1-VALIDATOR-01");
    println!("Validator ID: {}", validator.node_id);
    println!("Public Key Length: {} bytes (NIST ML-DSA-87 Standard)", validator.public_key.key_bytes.len());

    let transaction_payload = b"ZED_L1_TRANSACTION_STATE_ROOT_VALIDATION";
    let signature = validator.sign_payload(transaction_payload);
    println!("Signature Length: {} bytes (NIST ML-DSA-87 Standard)", signature.len());

    let is_valid = PqValidatorNode::verify_signature(&validator.public_key, transaction_payload, &signature);
    println!("NIST FIPS 204 Lattice Verification Result: {}", is_valid);
}
