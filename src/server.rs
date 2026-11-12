// Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
// Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::compliance::ComplianceEngine;
use crate::currency::Currency;
use crate::ledger::{AccountType, Amount, Ledger, Transfer};
use crate::router::{CrossBorderRoute, PaymentRouter};
use crate::wal::{WalRecord, WriteAheadLog};

/// Shared state container for all API routes.
pub struct AppState {
    pub ledger: Arc<Ledger>,
    pub wal: Option<Arc<WriteAheadLog>>,
    pub compliance: Arc<ComplianceEngine>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

// Request & Response DTOs
#[derive(Deserialize)]
pub struct CreateAccountRequest {
    pub account_type: AccountType,
    pub currency: Currency,
}

#[derive(Deserialize)]
pub struct InboundCreditRequest {
    pub amount: Amount,
    pub currency: Currency,
}

#[derive(Deserialize)]
pub struct TransferRequest {
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub amount: Amount,
    pub currency: Currency,
    pub counterparty_name: Option<String>,
}

#[derive(Deserialize)]
pub struct CrossBorderSettlementRequest {
    pub inbound_nostro_id: Uuid,
    pub platform_fee_id: Uuid,
    pub merchant_account_id: Uuid,
    pub inbound_amount: Amount,
    pub source_currency: Currency,
    pub target_currency: Currency,
    pub rate_numerator: u128,
    pub rate_denominator: u128,
    pub fee_bps: u128,
    pub counterparty_name: Option<String>,
}

#[derive(Serialize)]
pub struct CrossBorderSettlementResponse {
    pub transfer: Transfer,
    pub target_currency_amount: Amount,
}

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: &'static str,
    pub version: &'static str,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Builds the Axum router for the PaySwitch node.
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/v1/accounts", post(create_account_handler))
        .route("/api/v1/accounts/:id", get(get_account_handler))
        .route("/api/v1/accounts/:id/credit", post(credit_inbound_handler))
        .route("/api/v1/transfers", post(execute_transfer_handler))
        .route("/api/v1/settlements/cross-border", post(cross_border_handler))
        .route("/api/v1/audit/tip", get(get_audit_tip_handler))
        .with_state(state)
}

async fn health_handler() -> impl IntoResponse {
    Json(ApiResponse::ok(HealthStatus {
        status: "OPERATIONAL",
        version: "0.1.0",
        timestamp: chrono::Utc::now(),
    }))
}

async fn create_account_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAccountRequest>,
) -> impl IntoResponse {
    let id = state.ledger.create_account(req.account_type, req.currency);

    if let Some(ref wal) = state.wal {
        let _ = wal.append(&WalRecord::CreateAccount {
            id,
            account_type: req.account_type,
            currency: req.currency,
        });
    }

    (StatusCode::CREATED, Json(ApiResponse::ok(id)))
}

async fn get_account_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.ledger.get_account(id) {
        Ok(account) => (StatusCode::OK, Json(ApiResponse::ok(account))),
        Err(e) => (StatusCode::NOT_FOUND, Json(ApiResponse::err(e.to_string()))),
    }
}

async fn credit_inbound_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<InboundCreditRequest>,
) -> impl IntoResponse {
    match state.ledger.credit_inbound_settlement(id, req.amount, req.currency) {
        Ok(_) => {
            if let Some(ref wal) = state.wal {
                let _ = wal.append(&WalRecord::CreditInbound {
                    account_id: id,
                    amount: req.amount,
                    currency: req.currency,
                });
            }
            (StatusCode::OK, Json(ApiResponse::ok("Funds credited to Nostro reserve")))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.to_string()))),
    }
}

async fn execute_transfer_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TransferRequest>,
) -> impl IntoResponse {
    // Compliance screening check
    let counterparty = req.counterparty_name.as_deref().unwrap_or("Standard Counterparty");
    if let Err(e) = state.compliance.screen_transaction(req.debit_account_id, req.amount, counterparty) {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::err(e.to_string())));
    }

    match state.ledger.execute_transfer(
        req.debit_account_id,
        req.credit_account_id,
        req.amount,
        req.currency,
    ) {
        Ok(transfer) => {
            if let Some(ref wal) = state.wal {
                let _ = wal.append(&WalRecord::Transfer(transfer.clone()));
            }
            (StatusCode::OK, Json(ApiResponse::ok(transfer)))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.to_string()))),
    }
}

async fn cross_border_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CrossBorderSettlementRequest>,
) -> impl IntoResponse {
    // Compliance screening check
    let counterparty = req.counterparty_name.as_deref().unwrap_or("Cross Border Merchant");
    if let Err(e) = state.compliance.screen_transaction(req.inbound_nostro_id, req.inbound_amount, counterparty) {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::err(e.to_string())));
    }

    let route = CrossBorderRoute::new(
        req.source_currency,
        req.target_currency,
        req.rate_numerator,
        req.rate_denominator,
        req.fee_bps,
    );

    let router = PaymentRouter::new(&state.ledger);
    match router.route_cross_border_settlement(
        req.inbound_nostro_id,
        req.platform_fee_id,
        req.merchant_account_id,
        req.inbound_amount,
        &route,
    ) {
        Ok((transfer, target_amount)) => {
            if let Some(ref wal) = state.wal {
                let _ = wal.append(&WalRecord::Transfer(transfer.clone()));
            }
            (
                StatusCode::OK,
                Json(ApiResponse::ok(CrossBorderSettlementResponse {
                    transfer,
                    target_currency_amount: target_amount,
                })),
            )
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.to_string()))),
    }
}

async fn get_audit_tip_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let tip = state.ledger.audit_chain_tip();
    Json(ApiResponse::ok(format!("{:x?}", tip)))
}
