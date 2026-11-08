// Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
// Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::PaySwitchError;
use crate::ledger::Amount;

/// Risk status determination for a transaction screening check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreeningDecision {
    Approved,
    FlaggedForManualReview,
    Blocked,
}

/// Compliance screening policy limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePolicy {
    /// Maximum single transaction amount allowed without enhanced diligence (in cents, e.g. 1000000 = $10,000.00).
    pub max_single_transaction_limit: u128,
    /// 24-hour aggregate transaction volume limit (in cents).
    pub daily_velocity_limit: u128,
    /// Maximum number of transactions allowed within 1 hour.
    pub max_hourly_velocity_count: usize,
}

impl Default for CompliancePolicy {
    fn default() -> Self {
        Self {
            max_single_transaction_limit: 1_000_000,   // $10,000.00
            daily_velocity_limit: 5_000_000,          // $50,000.00
            max_hourly_velocity_count: 50,
        }
    }
}

/// Sanctions screening, AML velocity monitoring, and fraud prevention engine.
pub struct ComplianceEngine {
    policy: CompliancePolicy,
    sanctions_list: RwLock<HashSet<String>>,
    account_history: RwLock<HashMap<Uuid, Vec<(DateTime<Utc>, u128)>>>,
}

impl ComplianceEngine {
    pub fn new(policy: CompliancePolicy) -> Self {
        let engine = Self {
            policy,
            sanctions_list: RwLock::new(HashSet::new()),
            account_history: RwLock::new(HashMap::new()),
        };
        // Seed default high-risk global sanctions indicators
        engine.add_sanctioned_entity("AL-QAEDA");
        engine.add_sanctioned_entity("DAESH");
        engine.add_sanctioned_entity("TERRORIST ORGANIZATION");
        engine
    }

    /// Adds an entity or individual name to the active sanctions watch list.
    pub fn add_sanctioned_entity(&self, name: &str) {
        self.sanctions_list
            .write()
            .insert(name.trim().to_uppercase());
    }

    /// Screens an entity name against international sanctions watchlists.
    pub fn screen_entity_name(&self, name: &str) -> ScreeningDecision {
        let normalized = name.trim().to_uppercase();
        let list = self.sanctions_list.read();

        for sanctioned in list.iter() {
            if normalized.contains(sanctioned) || sanctioned.contains(&normalized) {
                return ScreeningDecision::Blocked;
            }
        }

        ScreeningDecision::Approved
    }

    /// Evaluates transaction amount and velocity against AML policies.
    pub fn screen_transaction(
        &self,
        account_id: Uuid,
        amount: Amount,
        counterparty_name: &str,
    ) -> Result<ScreeningDecision, PaySwitchError> {
        // 1. Sanctions Check
        if self.screen_entity_name(counterparty_name) == ScreeningDecision::Blocked {
            return Err(PaySwitchError::Internal(format!(
                "Transaction blocked: counterparty '{}' matched international sanctions watchlist",
                counterparty_name
            )));
        }

        let now = Utc::now();
        let one_day_ago = now - Duration::hours(24);
        let one_hour_ago = now - Duration::hours(1);

        let mut history_map = self.account_history.write();
        let history = history_map.entry(account_id).or_default();

        // Prune entries older than 24 hours
        history.retain(|(timestamp, _)| *timestamp > one_day_ago);

        // 2. Velocity Checks
        let hourly_count = history
            .iter()
            .filter(|(ts, _)| *ts > one_hour_ago)
            .count();

        if hourly_count >= self.policy.max_hourly_velocity_count {
            return Err(PaySwitchError::Internal(
                "Transaction velocity exceeded: too many transfers in 1 hour".into(),
            ));
        }

        let daily_volume: u128 = history.iter().map(|(_, amt)| *amt).sum();
        if daily_volume + amount.0 > self.policy.daily_velocity_limit {
            return Err(PaySwitchError::Internal(
                "Transaction limit exceeded: 24-hour aggregate volume threshold breached".into(),
            ));
        }

        // Record approved transaction in velocity window
        history.push((now, amount.0));

        // 3. Single Transaction Limit Check
        if amount.0 > self.policy.max_single_transaction_limit {
            Ok(ScreeningDecision::FlaggedForManualReview)
        } else {
            Ok(ScreeningDecision::Approved)
        }
    }
}
