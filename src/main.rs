use std::collections::HashMap;

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

    /// Genesis minting operation before admin keys are permanently renounced
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

    /// Execute state balance transfer
    pub fn transfer(&mut self, sender: &str, recipient: &str, amount: u128) -> Result<(), &'static str> {
        let sender_bal = self.accounts.get(sender).map(|a| a.balance).unwrap_or(0);
        if sender_bal < amount {
            return Err("Insufficient balance for L1 transfer");
        }

        // Deduct from sender
        if let Some(s) = self.accounts.get_mut(sender) {
            s.balance -= amount;
            s.nonce += 1;
        }

        // Credit to recipient
        let r = self.accounts.entry(recipient.to_string()).or_insert(Account {
            address: recipient.to_string(),
            balance: 0,
            nonce: 0,
        });
        r.balance += amount;

        println!("[L1 TRANSFER] Transferred {} ZED: {} -> {}", amount, sender, recipient);
        Ok(())
    }

    /// Permanent burn (routes tokens directly to unspendable NULL address)
    pub fn burn(&mut self, sender: &str, amount: u128) -> Result<(), &'static str> {
        self.transfer(sender, NULL_ADDRESS, amount)?;
        self.total_supply -= amount;
        println!("[DEFLATIONARY BURN] {} ZED routed to NULL_ADDRESS", amount);
        Ok(())
    }
}

fn main() {
    println!("=== ZED Tokenomics & Ledger Engine ===");
    let mut ledger = ZedLedger::new();

    // Initialize pre-launch genesis supply allocation
    ledger.genesis_mint("0xUSER_ALICE", 10_000_000_000).unwrap();
    ledger.genesis_mint("0xRESERVE_VAULT", 50_000_000_000).unwrap();

    // Transfer test
    ledger.transfer("0xUSER_ALICE", "0xUSER_BOB", 1_000_000).unwrap();
    
    // Deflationary burn test
    ledger.burn("0xUSER_ALICE", 200_000).unwrap();

    println!("Total Active Supply: {} ZED", ledger.total_supply);
    println!("Null Address Balance: {} ZED", ledger.accounts.get(NULL_ADDRESS).unwrap().balance);
}
