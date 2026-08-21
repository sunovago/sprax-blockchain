use crate::{engine::FaucetService, models::ClaimRequest};
use axum::{
    extract::{ConnectInfo, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde_json::json;
use sprax_core::ledger::ChainLedger;
use sprax_storage::RedbStore;
use std::{net::SocketAddr, sync::Arc, time::SystemTime, time::UNIX_EPOCH};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct FaucetServerState {
    pub faucet: Arc<FaucetService>,
    pub ledger: Arc<parking_lot::RwLock<ChainLedger<RedbStore>>>,
}

#[derive(Debug)]
pub struct FaucetServer;

impl FaucetServer {
    pub async fn start(
        faucet: Arc<FaucetService>,
        ledger: Arc<parking_lot::RwLock<ChainLedger<RedbStore>>>,
        port: u16,
    ) -> Result<(), String> {
        let state = FaucetServerState { faucet, ledger };

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ]);

        let app = Router::new()
            .route("/health", get(handle_health))
            .route("/stats", get(handle_stats))
            .route("/claim", post(handle_claim))
            .layer(cors)
            .with_state(state);

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("failed to bind faucet server on {addr}: {e}"))?;

        info!("SPRX Faucet HTTP Server listening on http://0.0.0.0:{port}");

        tokio::spawn(async move {
            if let Err(e) = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            {
                error!("Faucet server error: {e}");
            }
        });

        Ok(())
    }
}

async fn handle_health(State(state): State<FaucetServerState>) -> impl IntoResponse {
    let ledger_guard = state.ledger.read();
    let stats = state.faucet.get_stats(&ledger_guard);
    Json(json!({
        "status": "healthy",
        "faucet_address": state.faucet.faucet_address().to_hex(),
        "total_claims": stats.total_claims_count,
        "total_disbursed": stats.total_disbursed.to_string(),
        "available_balance": stats.available_balance.to_string(),
    }))
}

async fn handle_stats(State(state): State<FaucetServerState>) -> impl IntoResponse {
    let ledger_guard = state.ledger.read();
    Json(state.faucet.get_stats(&ledger_guard))
}

async fn handle_claim(
    State(state): State<FaucetServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<ClaimRequest>,
) -> impl IntoResponse {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let client_ip = req.client_ip.unwrap_or_else(|| addr.ip().to_string());
    let mut ledger_guard = state.ledger.write();

    match state.faucet.request_funds(
        &mut ledger_guard,
        &req.recipient,
        req.amount_sprx,
        &client_ip,
        now,
    ) {
        Ok(claim_res) => (StatusCode::OK, Json(json!(claim_res))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": e.to_string(),
            })),
        ),
    }
}
