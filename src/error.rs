use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum PaySwitchError {
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    #[error("Insufficient funds in account {0}: requested {1}, available {2}")]
    InsufficientFunds(Uuid, u128, u128),

    #[error("Currency mismatch: expected {0}, got {1}")]
    CurrencyMismatch(String, String),

    #[error("Ledger entry unbalanced: total debit {0} != total credit {1}")]
    UnbalancedEntry(u128, u128),

    #[error("Account is frozen or closed: {0}")]
    AccountSuspended(Uuid),

    #[error("Debit and credit accounts cannot be identical: {0}")]
    IdenticalAccounts(Uuid),

    #[error("Duplicate transfer ID: {0}")]
    DuplicateTransfer(Uuid),

    #[error("Transfer already committed or canceled: {0}")]
    InvalidTransferState(Uuid),

    #[error("Internal engine error: {0}")]
    Internal(String),
}
