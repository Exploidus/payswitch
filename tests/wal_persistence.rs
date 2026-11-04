// Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
// Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.

use payswitch::{AccountType, Amount, Currency, WalRecord, WriteAheadLog};
use uuid::Uuid;

#[test]
fn test_wal_append_and_recovery() {
    let temp_wal_path = std::env::temp_dir().join(format!("test_wal_{}.wal", Uuid::new_v4()));

    // 1. Open new WAL and write operations
    let wal = WriteAheadLog::open(&temp_wal_path).unwrap();

    let account_id = Uuid::new_v4();
    wal.append(&WalRecord::CreateAccount {
        id: account_id,
        account_type: AccountType::Asset,
        currency: Currency::USD,
    })
    .unwrap();

    wal.append(&WalRecord::CreditInbound {
        account_id,
        amount: Amount(500_000), // $5,000.00
        currency: Currency::USD,
    })
    .unwrap();

    drop(wal); // Simulate node shutdown/crash

    // 2. Re-open WAL and recover state
    let recovered_wal = WriteAheadLog::open(&temp_wal_path).unwrap();
    let records = recovered_wal.recover().unwrap();

    assert_eq!(records.len(), 2);
    match &records[0] {
        WalRecord::CreateAccount { id, account_type, currency } => {
            assert_eq!(*id, account_id);
            assert_eq!(*account_type, AccountType::Asset);
            assert_eq!(*currency, Currency::USD);
        }
        _ => panic!("Expected CreateAccount record"),
    }

    match &records[1] {
        WalRecord::CreditInbound { account_id: id, amount, currency } => {
            assert_eq!(*id, account_id);
            assert_eq!(amount.0, 500_000);
            assert_eq!(*currency, Currency::USD);
        }
        _ => panic!("Expected CreditInbound record"),
    }

    // Clean up temporary test file
    let _ = std::fs::remove_file(temp_wal_path);
}
