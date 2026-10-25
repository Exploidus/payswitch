// Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
// Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::PaySwitchError;
use crate::ledger::Amount;

/// Clearing rails supported within the Bangladesh national clearing jurisdiction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BangladeshClearingRail {
    /// National Payment Switch Bangladesh (ATM, POS, IBFT).
    Npsb,
    /// Bangladesh Electronic Funds Transfer Network (Direct Credit / Debit).
    Beftn,
    /// Real Time Gross Settlement (High-value instant interbank transfer).
    Rtgs,
    /// Mobile Financial Services Interoperability Rail (bKash, Nagad, Rocket).
    MfsBinimoy,
}

/// Routing channel for domestic payout destination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayoutDestination {
    /// Bank account routing with 9-digit routing number and account number.
    BankAccount {
        bank_name: String,
        routing_number: String,
        account_number: String,
        account_title: String,
    },
    /// Mobile wallet routing via national MFS gateway.
    MobileWallet {
        wallet_provider: String, // e.g. "bKash", "Nagad", "Rocket"
        wallet_number: String,
        account_title: String,
    },
}

/// Outbound domestic settlement instruction generated from an inbound cross-border transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomesticSettlementInstruction {
    pub settlement_id: Uuid,
    pub rail: BangladeshClearingRail,
    pub source_transfer_id: Uuid,
    pub destination: PayoutDestination,
    pub settled_bdt_amount: Amount,
    pub narration: String,
    pub dispatch_time: DateTime<Utc>,
}

impl DomesticSettlementInstruction {
    pub fn new(
        rail: BangladeshClearingRail,
        source_transfer_id: Uuid,
        destination: PayoutDestination,
        settled_bdt_amount: Amount,
        narration: impl Into<String>,
    ) -> Result<Self, PaySwitchError> {
        if settled_bdt_amount.0 == 0 {
            return Err(PaySwitchError::Internal("Settlement amount cannot be zero".into()));
        }

        // Validate destination identifiers
        match &destination {
            PayoutDestination::BankAccount { routing_number, account_number, .. } => {
                if routing_number.trim().len() != 9 {
                    return Err(PaySwitchError::Internal(
                        "Bangladesh Bank routing number must be exactly 9 digits".into(),
                    ));
                }
                if account_number.trim().is_empty() {
                    return Err(PaySwitchError::Internal("Bank account number cannot be empty".into()));
                }
            }
            PayoutDestination::MobileWallet { wallet_number, .. } => {
                let clean_wallet = wallet_number.trim().replace("+88", "");
                if clean_wallet.len() != 11 || !clean_wallet.starts_with("01") {
                    return Err(PaySwitchError::Internal(
                        "Invalid Bangladesh MSISDN wallet number format".into(),
                    ));
                }
            }
        }

        Ok(Self {
            settlement_id: Uuid::new_v4(),
            rail,
            source_transfer_id,
            destination,
            settled_bdt_amount,
            narration: narration.into(),
            dispatch_time: Utc::now(),
        })
    }
}
