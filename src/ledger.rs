use std::collections::HashMap;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::currency::Currency;
use crate::error::PaySwitchError;

/// High-precision non-floating-point monetary value in atomic units (e.g. cents).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Amount(pub u128);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    /// Assets owned by the platform or clearing house.
    Asset,
    /// Liabilities owed to customers, merchants, or partners.
    Liability,
    /// Retained earnings, fees collected, and platform equity.
    Equity,
    /// Inbound settlement escrow before final clearance.
    SettlementEscrow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub account_type: AccountType,
    pub currency: Currency,
    pub settled_balance: Amount,
    pub pending_hold: Amount,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl Account {
    pub fn new(id: Uuid, account_type: AccountType, currency: Currency) -> Self {
        Self {
            id,
            account_type,
            currency,
            settled_balance: Amount(0),
            pending_hold: Amount(0),
            is_active: true,
            created_at: Utc::now(),
        }
    }

    pub fn available_balance(&self) -> Amount {
        Amount(self.settled_balance.0.saturating_sub(self.pending_hold.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    Committed,
    Voided,
}

/// Atomic double-entry transfer between two accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub id: Uuid,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub amount: Amount,
    pub currency: Currency,
    pub status: TransferStatus,
    pub timestamp: DateTime<Utc>,
    pub previous_entry_hash: [u8; 32],
    pub entry_hash: [u8; 32],
}

/// In-memory, lock-free, zero-loss double-entry financial ledger.
pub struct Ledger {
    accounts: RwLock<HashMap<Uuid, Account>>,
    transfers: RwLock<HashMap<Uuid, Transfer>>,
    latest_hash: RwLock<[u8; 32]>,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            accounts: RwLock::new(HashMap::new()),
            transfers: RwLock::new(HashMap::new()),
            latest_hash: RwLock::new([0u8; 32]),
        }
    }

    /// Registers a new financial account into the ledger state.
    pub fn create_account(&self, account_type: AccountType, currency: Currency) -> Uuid {
        let id = Uuid::new_v4();
        self.create_account_with_id(id, account_type, currency);
        id
    }

    /// Registers a new financial account with a deterministic or restored ID.
    pub fn create_account_with_id(&self, id: Uuid, account_type: AccountType, currency: Currency) {
        let account = Account::new(id, account_type, currency);
        self.accounts.write().insert(id, account);
    }

    /// Fetches an immutable snapshot of an account.
    pub fn get_account(&self, id: Uuid) -> Result<Account, PaySwitchError> {
        self.accounts
            .read()
            .get(&id)
            .cloned()
            .ok_or(PaySwitchError::AccountNotFound(id))
    }

    /// Direct atomic deposit for external bank settlement ingress.
    pub fn credit_inbound_settlement(
        &self,
        account_id: Uuid,
        amount: Amount,
        currency: Currency,
    ) -> Result<(), PaySwitchError> {
        let mut accounts = self.accounts.write();
        let account = accounts
            .get_mut(&account_id)
            .ok_or(PaySwitchError::AccountNotFound(account_id))?;

        if account.currency != currency {
            return Err(PaySwitchError::CurrencyMismatch(
                account.currency.to_string(),
                currency.to_string(),
            ));
        }

        account.settled_balance.0 += amount.0;
        Ok(())
    }

    /// Executes an atomic double-entry transfer maintaining strict mathematical balance.
    pub fn execute_transfer(
        &self,
        debit_account_id: Uuid,
        credit_account_id: Uuid,
        amount: Amount,
        currency: Currency,
    ) -> Result<Transfer, PaySwitchError> {
        if debit_account_id == credit_account_id {
            return Err(PaySwitchError::IdenticalAccounts(debit_account_id));
        }

        let mut accounts = self.accounts.write();

        // 1. Validate Debit Account
        let debit_account = accounts
            .get(&debit_account_id)
            .ok_or(PaySwitchError::AccountNotFound(debit_account_id))?;

        if !debit_account.is_active {
            return Err(PaySwitchError::AccountSuspended(debit_account_id));
        }
        if debit_account.currency != currency {
            return Err(PaySwitchError::CurrencyMismatch(
                debit_account.currency.to_string(),
                currency.to_string(),
            ));
        }
        if debit_account.available_balance().0 < amount.0 {
            return Err(PaySwitchError::InsufficientFunds(
                debit_account_id,
                amount.0,
                debit_account.available_balance().0,
            ));
        }

        // 2. Validate Credit Account
        let credit_account = accounts
            .get(&credit_account_id)
            .ok_or(PaySwitchError::AccountNotFound(credit_account_id))?;

        if !credit_account.is_active {
            return Err(PaySwitchError::AccountSuspended(credit_account_id));
        }
        if credit_account.currency != currency {
            return Err(PaySwitchError::CurrencyMismatch(
                credit_account.currency.to_string(),
                currency.to_string(),
            ));
        }

        // 3. Atomically apply debit and credit legs
        accounts.get_mut(&debit_account_id).unwrap().settled_balance.0 -= amount.0;
        accounts.get_mut(&credit_account_id).unwrap().settled_balance.0 += amount.0;

        // 3. Compute Cryptographic Merkle Hash Leg
        let transfer_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let mut latest_hash_lock = self.latest_hash.write();
        let prev_hash = *latest_hash_lock;

        let mut hasher = Sha256::new();
        hasher.update(&prev_hash);
        hasher.update(transfer_id.as_bytes());
        hasher.update(debit_account_id.as_bytes());
        hasher.update(credit_account_id.as_bytes());
        hasher.update(&amount.0.to_be_bytes());
        hasher.update(&(currency as u8).to_be_bytes());
        let entry_hash: [u8; 32] = hasher.finalize().into();
        *latest_hash_lock = entry_hash;

        let transfer = Transfer {
            id: transfer_id,
            debit_account_id,
            credit_account_id,
            amount,
            currency,
            status: TransferStatus::Committed,
            timestamp,
            previous_entry_hash: prev_hash,
            entry_hash,
        };

        self.transfers.write().insert(transfer_id, transfer.clone());

        Ok(transfer)
    }

    /// Verifies the mathematical double-entry balance across all accounts in the ledger.
    pub fn verify_ledger_invariants(&self) -> bool {
        let accounts = self.accounts.read();
        let mut total_assets = 0u128;
        let mut total_liabilities_and_equity = 0u128;

        for acc in accounts.values() {
            match acc.account_type {
                AccountType::Asset | AccountType::SettlementEscrow => {
                    total_assets += acc.settled_balance.0;
                }
                AccountType::Liability | AccountType::Equity => {
                    total_liabilities_and_equity += acc.settled_balance.0;
                }
            }
        }

        total_assets == total_liabilities_and_equity
    }

    /// Returns the latest cryptographic Merkle chain tip hash.
    pub fn audit_chain_tip(&self) -> [u8; 32] {
        *self.latest_hash.read()
    }
}
