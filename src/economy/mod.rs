//! Economy module: balances, payments, and top-balance queries.
//!
//! Balances are keyed by the player's UUID string and persisted through the
//! file-based [`Storage`]. All operations are synchronous; the WASM plugin API
//! delivers command handlers on a single thread.

use crate::storage::Storage;
use std::sync::Arc;

pub mod commands;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EconomyAccount {
    pub uuid: String,
    pub name: String,
    pub balance: f64,
    pub updated_at: i64,
}

pub struct Economy {
    storage: Arc<Storage>,
    starting_balance: f64,
    min_money: f64,
    max_money: f64,
    currency_symbol: String,
    currency_format: String,
}

impl Default for Economy {
    fn default() -> Self {
        Self::new(Arc::new(Storage::new(".")))
    }
}

impl Economy {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
            starting_balance: 0.0,
            min_money: 0.0,
            max_money: f64::MAX,
            currency_symbol: "$".to_string(),
            currency_format: "{symbol}{amount}".to_string(),
        }
    }

    fn key(&self, uuid: &str) -> String {
        uuid.to_string()
    }

    /// Returns the balance for `uuid`, defaulting to the configured starting balance.
    pub fn get_balance(&self, uuid: &str) -> f64 {
        if let Some(account) = self
            .storage
            .get::<EconomyAccount>("economy", &self.key(uuid))
        {
            return account.balance;
        }
        self.starting_balance
    }

    /// Sets the balance for `uuid`, clamped to the configured min/max.
    pub fn set_balance(&self, uuid: &str, name: &str, amount: f64) -> f64 {
        let clamped = amount.clamp(self.min_money, self.max_money);
        let account = EconomyAccount {
            uuid: uuid.to_string(),
            name: name.to_string(),
            balance: clamped,
            updated_at: crate::utils::time::now_timestamp(),
        };
        self.storage.set("economy", &self.key(uuid), &account);
        clamped
    }

    /// Adds `amount` to the balance and returns the new balance.
    pub fn add_balance(&self, uuid: &str, name: &str, amount: f64) -> f64 {
        let new_balance = (self.get_balance(uuid) + amount).clamp(self.min_money, self.max_money);
        self.set_balance(uuid, name, new_balance)
    }

    /// Subtracts `amount` from the balance and returns the new balance.
    pub fn take_balance(&self, uuid: &str, name: &str, amount: f64) -> f64 {
        self.add_balance(uuid, name, -amount)
    }

    /// Transfers `amount` from `from_uuid` to `to_uuid`.
    pub fn pay(
        &self,
        from_uuid: &str,
        from_name: &str,
        to_uuid: &str,
        to_name: &str,
        amount: f64,
    ) -> Result<f64, String> {
        if from_uuid == to_uuid {
            return Err("You cannot pay yourself".to_string());
        }
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }
        if self.get_balance(from_uuid) < amount {
            return Err("Insufficient funds".to_string());
        }
        self.take_balance(from_uuid, from_name, amount);
        Ok(self.add_balance(to_uuid, to_name, amount))
    }

    /// Returns up to `limit` accounts ordered by balance (descending).
    pub fn top_balances(&self, limit: usize) -> Vec<EconomyAccount> {
        let mut accounts: Vec<EconomyAccount> = self.storage.values("economy");
        accounts.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap_or(std::cmp::Ordering::Equal));
        accounts.truncate(limit);
        accounts
    }

    /// Formats an amount using the configured currency symbol/format.
    pub fn format_money(&self, amount: f64) -> String {
        self.currency_format
            .replace("{symbol}", &self.currency_symbol)
            .replace("{amount}", &format!("{amount:.2}"))
    }
}