// Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
// Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.

use std::net::SocketAddr;
use std::sync::Arc;
use payswitch::{
    create_router, AccountType, Amount, AppState, ComplianceEngine, CompliancePolicy,
    Currency, Ledger, WalRecord, WriteAheadLog,
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Initializing PaySwitch Sovereign Financial Node v0.1.0...");

    let ledger = Arc::new(Ledger::new());

    // 1. Initialize Durable Write-Ahead Log
    let wal_path = std::env::var("PAYSWITCH_WAL_PATH").unwrap_or_else(|_| "payswitch.wal".to_string());
    let wal = match WriteAheadLog::open(&wal_path) {
        Ok(w) => {
            info!("WAL storage initialized at '{}'", wal_path);
            let records = w.recover()?;
            if !records.is_empty() {
                info!("Replaying {} WAL records to reconstruct state...", records.len());
                for record in records {
                    match record {
                        WalRecord::CreateAccount { id, account_type, currency } => {
                            ledger.create_account_with_id(id, account_type, currency);
                        }
                        WalRecord::CreditInbound { account_id, amount, currency } => {
                            let _ = ledger.credit_inbound_settlement(account_id, amount, currency);
                        }
                        WalRecord::Transfer(tx) => {
                            let _ = ledger.execute_transfer(
                                tx.debit_account_id,
                                tx.credit_account_id,
                                tx.amount,
                                tx.currency,
                            );
                        }
                    }
                }
                info!("WAL replay completed successfully.");
            }
            Some(Arc::new(w))
        }
        Err(e) => {
            tracing::warn!("Failed to initialize WAL (running in-memory only): {}", e);
            None
        }
    };

    // 2. Initialize Compliance & AML Engine
    let compliance = Arc::new(ComplianceEngine::new(CompliancePolicy::default()));
    info!("Compliance and AML Watchlist Screening Engine active.");

    // 3. Establish Default Platform Accounts for Demonstration
    let nostro_usd = ledger.create_account(AccountType::Asset, Currency::USD);
    let platform_revenue_usd = ledger.create_account(AccountType::Equity, Currency::USD);
    let merchant_usd = ledger.create_account(AccountType::Liability, Currency::USD);

    let initial_reserve = Amount(100_000 * Currency::USD.atomic_multiplier()); // $100,000.00
    ledger.credit_inbound_settlement(nostro_usd, initial_reserve, Currency::USD)?;

    info!("Demonstration Platform Accounts Provisioned:");
    info!("  - Inbound Nostro Reserve [USD]: {}", nostro_usd);
    info!("  - Platform Fee Account   [USD]: {}", platform_revenue_usd);
    info!("  - Demonstration Merchant [USD]: {}", merchant_usd);
    info!("  - Initial Nostro Reserve: $100,000.00 USD");

    // 4. Assemble Server State and Start High-Throughput HTTP REST API
    let state = Arc::new(AppState {
        ledger,
        wal,
        compliance,
    });

    let app = create_router(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("PaySwitch Ingress API server listening on http://{}", addr);
    info!("Available API Endpoints:");
    info!("  - GET  /health");
    info!("  - POST /api/v1/accounts");
    info!("  - GET  /api/v1/accounts/:id");
    info!("  - POST /api/v1/accounts/:id/credit");
    info!("  - POST /api/v1/transfers");
    info!("  - POST /api/v1/settlements/cross-border");
    info!("  - GET  /api/v1/audit/tip");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
