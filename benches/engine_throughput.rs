// Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
// Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use payswitch::{AccountType, Amount, Currency, Ledger};

fn bench_ledger_transfers(c: &mut Criterion) {
    let ledger = Ledger::new();
    let usd = Currency::USD;

    let source = ledger.create_account(AccountType::Asset, usd);
    let dest = ledger.create_account(AccountType::Liability, usd);

    ledger
        .credit_inbound_settlement(source, Amount(1_000_000_000_000), usd)
        .unwrap();

    c.bench_function("ledger_transfer_throughput", |b| {
        b.iter(|| {
            ledger
                .execute_transfer(
                    black_box(source),
                    black_box(dest),
                    black_box(Amount(10)),
                    black_box(usd),
                )
                .unwrap();
        })
    });
}

criterion_group!(benches, bench_ledger_transfers);
criterion_main!(benches);
