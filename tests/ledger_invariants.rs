// Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
// Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.

use payswitch::{
    AccountType, Amount, BangladeshClearingRail, CrossBorderRoute, Currency,
    DomesticSettlementInstruction, Ledger, Pacs008CreditTransfer, PaymentRouter, PayoutDestination,
};
use uuid::Uuid;

#[test]
fn test_double_entry_balance_conservation() {
    let ledger = Ledger::new();
    let usd = Currency::USD;

    let source_id = ledger.create_account(AccountType::Asset, usd);
    let dest_id = ledger.create_account(AccountType::Liability, usd);

    // Initial deposit
    ledger.credit_inbound_settlement(source_id, Amount(1_000_000), usd).unwrap(); // $10,000.00

    assert_eq!(ledger.get_account(source_id).unwrap().settled_balance.0, 1_000_000);
    assert_eq!(ledger.get_account(dest_id).unwrap().settled_balance.0, 0);

    // Transfer $250.00
    let transfer = ledger.execute_transfer(source_id, dest_id, Amount(25_000), usd).unwrap();
    assert_eq!(transfer.amount.0, 25_000);

    // Invariant: Source decreased by 25_000, Dest increased by 25_000
    assert_eq!(ledger.get_account(source_id).unwrap().settled_balance.0, 975_000);
    assert_eq!(ledger.get_account(dest_id).unwrap().settled_balance.0, 25_000);

    // Total circulating units remain exactly conserved
    let total_circulation = ledger.get_account(source_id).unwrap().settled_balance.0
        + ledger.get_account(dest_id).unwrap().settled_balance.0;
    assert_eq!(total_circulation, 1_000_000);
}

#[test]
fn test_overdraft_prevention() {
    let ledger = Ledger::new();
    let usd = Currency::USD;

    let acc_a = ledger.create_account(AccountType::Asset, usd);
    let acc_b = ledger.create_account(AccountType::Liability, usd);

    ledger.credit_inbound_settlement(acc_a, Amount(5_000), usd).unwrap(); // $50.00

    // Attempting to withdraw $100.00 when only $50.00 is available must fail
    let result = ledger.execute_transfer(acc_a, acc_b, Amount(10_000), usd);
    assert!(result.is_err());

    // Balance must be unmodified
    assert_eq!(ledger.get_account(acc_a).unwrap().settled_balance.0, 5_000);
    assert_eq!(ledger.get_account(acc_b).unwrap().settled_balance.0, 0);
}

#[test]
fn test_currency_segregation_enforcement() {
    let ledger = Ledger::new();
    let acc_usd = ledger.create_account(AccountType::Asset, Currency::USD);
    let acc_bdt = ledger.create_account(AccountType::Liability, Currency::BDT);

    ledger.credit_inbound_settlement(acc_usd, Amount(500_000), Currency::USD).unwrap();

    // Direct ledger transfer across different currencies must be rejected
    let result = ledger.execute_transfer(acc_usd, acc_bdt, Amount(10_000), Currency::USD);
    assert!(result.is_err());
}

#[test]
fn test_merkle_chain_integrity_verification() {
    let ledger = Ledger::new();
    let usd = Currency::USD;

    let vault = ledger.create_account(AccountType::Asset, usd);
    let customer = ledger.create_account(AccountType::Liability, usd);

    ledger.credit_inbound_settlement(vault, Amount(100_000_000), usd).unwrap();

    let mut hashes = Vec::new();

    // Perform 100 consecutive transfers
    for _ in 0..100 {
        let tx = ledger.execute_transfer(vault, customer, Amount(1_000), usd).unwrap();
        hashes.push((tx.previous_entry_hash, tx.entry_hash));
    }

    assert_eq!(hashes.len(), 100);

    // Verify cryptographic linkage: entry[i].previous == entry[i-1].hash
    for i in 1..hashes.len() {
        assert_eq!(hashes[i].0, hashes[i - 1].1, "Merkle chain linkage broken at index {}", i);
    }
}

#[test]
fn test_cross_border_fx_router() {
    let ledger = Ledger::new();
    let usd = Currency::USD;

    let nostro_id = ledger.create_account(AccountType::Asset, usd);
    let platform_fee_id = ledger.create_account(AccountType::Equity, usd);
    let merchant_id = ledger.create_account(AccountType::Liability, usd);

    ledger.credit_inbound_settlement(nostro_id, Amount(100_000), usd).unwrap(); // $1,000.00

    let router = PaymentRouter::new(&ledger);

    // Rate: 1 USD = 121.50 BDT (12150 / 100), Fee: 75 bps (0.75%)
    let route = CrossBorderRoute::new(Currency::USD, Currency::BDT, 12150, 100, 75);

    let (transfer, bdt_equiv) = router
        .route_cross_border_settlement(
            nostro_id,
            platform_fee_id,
            merchant_id,
            Amount(100_000),
            &route,
        )
        .unwrap();

    // Fee = 100,000 * 75 / 10,000 = 750 cents ($7.50)
    // Merchant Net = 100,000 - 750 = 99,250 cents ($992.50)
    // BDT Equiv = 99,250 * 12150 / 100 = 12,058,875 poisha (120,588.75 BDT)
    assert_eq!(transfer.amount.0, 99_250);
    assert_eq!(bdt_equiv.0, 12_058_875);

    assert_eq!(ledger.get_account(platform_fee_id).unwrap().settled_balance.0, 750);
    assert_eq!(ledger.get_account(merchant_id).unwrap().settled_balance.0, 99_250);
    assert_eq!(ledger.get_account(nostro_id).unwrap().settled_balance.0, 0);
}

#[test]
fn test_iso20022_and_domestic_payout_generation() {
    let pacs = Pacs008CreditTransfer::new(
        "CHASUS33XXX",
        "Apex Global Corp",
        "US12345678901234567890",
        Amount(500_000), // $5,000.00 USD
        Currency::USD,
        "SEBDBD22XXX",
        "Rahad Bhuiya Software Exports",
        "BD001122334455",
        Some("Invoice EX-2026-88".into()),
    );

    assert!(pacs.validate().is_ok());

    let bdt_settlement = Amount(607_500_00); // 607,500.00 BDT
    let domestic_instruction = DomesticSettlementInstruction::new(
        BangladeshClearingRail::MfsBinimoy,
        Uuid::new_v4(),
        PayoutDestination::MobileWallet {
            wallet_provider: "bKash".into(),
            wallet_number: "01712345678".into(),
            account_title: "Rahad Bhuiya".into(),
        },
        bdt_settlement,
        "Cross-Border Export Remittance via PaySwitch",
    );

    assert!(domestic_instruction.is_ok());
}
