// Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
// Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::currency::Currency;
use crate::error::PaySwitchError;
use crate::ledger::Amount;

/// Financial Institutional Customer Credit Transfer message according to ISO 20022 pacs.008.001.10.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pacs008CreditTransfer {
    pub message_id: String,
    pub creation_date_time: DateTime<Utc>,
    pub end_to_end_id: String,
    pub instructing_bic: String,
    pub debtor_name: String,
    pub debtor_account_iban: String,
    pub instructed_amount: Amount,
    pub currency: Currency,
    pub creditor_bic: String,
    pub creditor_name: String,
    pub creditor_account: String,
    pub remittance_information: Option<String>,
}

/// Payment Status Report according to ISO 20022 pacs.002.001.12.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pacs002StatusReport {
    pub original_message_id: String,
    pub original_end_to_end_id: String,
    pub transaction_status: TransactionStatus,
    pub status_reason_code: Option<String>,
    pub clearing_timestamp: DateTime<Utc>,
    pub internal_ledger_tx: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Accepted Settlement Completed (ACTC)
    AcceptedSettlementCompleted,
    /// Pending Settlement (PDNG)
    PendingSettlement,
    /// Rejected / Failed (RJCT)
    Rejected,
}

impl Pacs008CreditTransfer {
    pub fn new(
        instructing_bic: impl Into<String>,
        debtor_name: impl Into<String>,
        debtor_account_iban: impl Into<String>,
        instructed_amount: Amount,
        currency: Currency,
        creditor_bic: impl Into<String>,
        creditor_name: impl Into<String>,
        creditor_account: impl Into<String>,
        remittance: Option<String>,
    ) -> Self {
        let now = Utc::now();
        let message_id = format!("PAYSW-PACS008-{}-{}", now.format("%Y%m%d%H%M%S"), Uuid::new_v4().simple());
        let end_to_end_id = format!("E2E-{}", Uuid::new_v4());

        Self {
            message_id,
            creation_date_time: now,
            end_to_end_id,
            instructing_bic: instructing_bic.into(),
            debtor_name: debtor_name.into(),
            debtor_account_iban: debtor_account_iban.into(),
            instructed_amount,
            currency,
            creditor_bic: creditor_bic.into(),
            creditor_name: creditor_name.into(),
            creditor_account: creditor_account.into(),
            remittance_information: remittance,
        }
    }

    /// Validates message conformance against ISO 20022 constraints.
    pub fn validate(&self) -> Result<(), PaySwitchError> {
        if self.instructing_bic.trim().is_empty() || self.creditor_bic.trim().is_empty() {
            return Err(PaySwitchError::Internal("BIC identifier cannot be empty".into()));
        }
        if self.instructed_amount.0 == 0 {
            return Err(PaySwitchError::Internal("Transaction amount must be strictly positive".into()));
        }
        Ok(())
    }
}
