// Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
// Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.

use payswitch::{Amount, ComplianceEngine, CompliancePolicy, ScreeningDecision};
use uuid::Uuid;

#[test]
fn test_sanctions_entity_screening() {
    let policy = CompliancePolicy::default();
    let engine = ComplianceEngine::new(policy);

    // Legitimate merchant passes
    let decision = engine.screen_entity_name("Rahad Bhuiya Tech Exports");
    assert_eq!(decision, ScreeningDecision::Approved);

    // Sanctioned entity is blocked
    let blocked_decision = engine.screen_entity_name("Daesh Global Operations");
    assert_eq!(blocked_decision, ScreeningDecision::Blocked);
}

#[test]
fn test_aml_velocity_and_threshold_limits() {
    let policy = CompliancePolicy {
        max_single_transaction_limit: 100_000, // $1,000.00
        daily_velocity_limit: 250_000,        // $2,500.00
        max_hourly_velocity_count: 3,
    };
    let engine = ComplianceEngine::new(policy);
    let account_id = Uuid::new_v4();

    // 1. Normal transfer under threshold
    let res1 = engine.screen_transaction(account_id, Amount(50_000), "Safe Corp");
    assert!(res1.is_ok());
    assert_eq!(res1.unwrap(), ScreeningDecision::Approved);

    // 2. Transfer above single limit is flagged for manual review
    let res2 = engine.screen_transaction(account_id, Amount(120_000), "Safe Corp");
    assert!(res2.is_ok());
    assert_eq!(res2.unwrap(), ScreeningDecision::FlaggedForManualReview);

    // 3. Hourly velocity count reached (3 transfers)
    let res3 = engine.screen_transaction(account_id, Amount(50_000), "Safe Corp");
    assert!(res3.is_ok());

    // 4th transfer within 1 hour exceeds velocity limit
    let res4 = engine.screen_transaction(account_id, Amount(10_000), "Safe Corp");
    assert!(res4.is_err());
}
