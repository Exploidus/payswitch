// Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
// Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.

use std::sync::Arc;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use payswitch::{create_router, AppState, ComplianceEngine, CompliancePolicy, Ledger};
use tower::ServiceExt;

#[tokio::test]
async fn test_api_health_endpoint() {
    let ledger = Arc::new(Ledger::new());
    let compliance = Arc::new(ComplianceEngine::new(CompliancePolicy::default()));
    let state = Arc::new(AppState {
        ledger,
        wal: None,
        compliance,
    });

    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_account_provisioning_and_audit_tip() {
    let ledger = Arc::new(Ledger::new());
    let compliance = Arc::new(ComplianceEngine::new(CompliancePolicy::default()));
    let state = Arc::new(AppState {
        ledger,
        wal: None,
        compliance,
    });

    let app = create_router(state);

    // 1. Create account via POST /api/v1/accounts
    let create_payload = serde_json::json!({
        "account_type": "Asset",
        "currency": "USD"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/accounts")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Query Merkle audit tip via GET /api/v1/audit/tip
    let tip_response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/audit/tip")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(tip_response.status(), StatusCode::OK);
}
