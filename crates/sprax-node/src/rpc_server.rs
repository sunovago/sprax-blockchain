use crate::{error::NodeError, service::NodeService};
use axum::{
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sprax_crypto::Hasher;
use sprax_types::{
    Address, Amount, ChainId, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage,
};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Instant,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

/// In-memory IP rate limiter protecting public endpoints against abuse.
#[derive(Debug)]
pub struct RateLimiter {
    clients: Mutex<HashMap<IpAddr, (u32, Instant)>>,
    max_requests_per_minute: u32,
}

impl RateLimiter {
    pub fn new(max_requests_per_minute: u32) -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
            max_requests_per_minute,
        }
    }

    pub fn check(&self, ip: IpAddr) -> bool {
        let mut map = self.clients.lock();
        let now = Instant::now();
        let entry = map.entry(ip).or_insert((0, now));
        if now.duration_since(entry.1).as_secs() >= 60 {
            *entry = (1, now);
            true
        } else if entry.0 < self.max_requests_per_minute {
            entry.0 += 1;
            true
        } else {
            false
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(600) // Default: 600 req/min per IP
    }
}

#[derive(Debug, Clone)]
pub struct RpcServerState {
    pub node: NodeService,
    pub rate_limiter: Arc<RateLimiter>,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: Option<String>,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Clone)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Clone)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: String, data: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data,
            }),
        }
    }
}

#[derive(Debug)]
pub struct RpcServer;

#[derive(Debug)]
pub struct RpcServerHandle {
    shutdown: tokio::sync::oneshot::Sender<()>,
    task: tokio::task::JoinHandle<()>,
}

impl RpcServerHandle {
    pub async fn stop(mut self) {
        let _ = self.shutdown.send(());
        if tokio::time::timeout(std::time::Duration::from_secs(5), &mut self.task)
            .await
            .is_err()
        {
            self.task.abort();
            let _ = self.task.await;
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<usize>,
    pub offset: Option<u64>,
}

impl RpcServer {
    pub fn router(node: NodeService) -> Router {
        let state = Arc::new(RpcServerState {
            node,
            rate_limiter: Arc::new(RateLimiter::default()),
        });

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ]);

        Router::new()
            // JSON-RPC 2.0 endpoints
            .route("/", post(handle_json_rpc))
            .route("/rpc", post(handle_json_rpc))
            // REST Version 1 API (/api/v1/...)
            .route("/api/v1/chain/info", get(handle_api_chain_info))
            .route("/api/v1/chain/status", get(handle_api_chain_status))
            .route("/api/v1/blocks", get(handle_api_blocks_list))
            .route("/api/v1/blocks/:height", get(handle_api_block_by_height))
            .route("/api/v1/blocks/hash/:hash", get(handle_api_block_by_hash))
            .route("/api/v1/transactions", post(handle_api_tx_submit))
            .route("/api/v1/transactions/:hash", get(handle_api_tx_by_hash))
            .route("/api/v1/transactions/:hash/receipt", get(handle_api_tx_receipt))
            .route("/api/v1/transactions/:hash/status", get(handle_api_tx_status))
            .route("/api/v1/accounts/:address", get(handle_api_account))
            .route("/api/v1/accounts/:address/balance", get(handle_account_balance))
            .route("/api/v1/accounts/:address/nonce", get(handle_account_nonce))
            .route("/api/v1/accounts/:address/transactions", get(handle_api_account_txs))
            .route("/api/v1/validators", get(handle_api_validators))
            .route("/api/v1/validators/:address", get(handle_api_validator_by_addr))
            .route("/api/v1/validators/:address/rewards", get(handle_api_validator_rewards))
            .route("/api/v1/staking/overview", get(handle_api_staking_overview))
            .route("/api/v1/staking/delegations/:address", get(handle_api_delegations))
            .route("/api/v1/staking/unbonding/:address", get(handle_api_unbonding))
            .route("/api/v1/network/status", get(handle_api_network_status))
            .route("/api/v1/network/peers", get(handle_api_network_peers))
            .route("/api/v1/network/sync", get(handle_api_network_sync))
            // Contract REST API (/api/v1/contracts/...)
            .route("/api/v1/contracts", get(handle_api_list_contracts))
            .route("/api/v1/contracts/deploy", post(handle_api_contract_deploy))
            .route("/api/v1/contracts/:address", get(handle_api_contract_by_addr))
            .route("/api/v1/contracts/:address/code", get(handle_api_contract_code))
            .route("/api/v1/contracts/:address/storage", get(handle_api_contract_storage))
            .route("/api/v1/contracts/:address/events", get(handle_api_contract_events))
            .route("/api/v1/contracts/:address/call", post(handle_api_contract_call))
            .route("/api/v1/contracts/:address/query", post(handle_contract_query))
            .route("/api/v1/contracts/:address/transactions", get(handle_api_contract_txs))
            // Convenience routes (Wallet SDK & Explorer compatibility)
            .route("/health", get(handle_health))
            .route("/status", get(handle_status))
            .route("/accounts/:address/balance", get(handle_account_balance))
            .route("/accounts/:address/nonce", get(handle_account_nonce))
            .route("/contracts/:address/query", post(handle_contract_query))
            .route("/txs/broadcast", post(handle_rest_broadcast))
            .route("/txs/:hash", get(handle_rest_get_tx))
            .route("/blocks/latest", get(handle_rest_latest_block))
            .route("/blocks/:height_or_hash", get(handle_rest_get_block))
            .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10 MB payload ceiling
            .layer(cors)
            .with_state(state)
    }

    pub async fn start(node: NodeService, port: u16) -> Result<RpcServerHandle, NodeError> {
        let app = Self::router(node);

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            NodeError::RuntimeError(format!("failed to bind JSON-RPC server on {addr}: {e}"))
        })?;

        info!("SPRX JSON-RPC & REST HTTP Server listening on http://0.0.0.0:{port}");

        let (shutdown, stopped) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = stopped.await;
                })
                .await
            {
                error!("RPC server error: {e}");
            }
        });

        Ok(RpcServerHandle { shutdown, task })
    }
}

// ==========================================
// JSON-RPC 2.0 Dispatcher (Single & Batch)
// ==========================================

async fn handle_json_rpc(
    State(state): State<Arc<RpcServerState>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    match payload {
        Value::Array(items) => {
            if items.is_empty() {
                return (
                    StatusCode::OK,
                    Json(json!(JsonRpcResponse::error(
                        None,
                        -32600,
                        "Invalid Request: empty batch".into(),
                        None
                    ))),
                );
            }
            let mut responses = Vec::with_capacity(items.len());
            for item in items {
                match serde_json::from_value::<JsonRpcRequest>(item) {
                    Ok(req) => {
                        let res = dispatch_single_rpc(&state, req).await;
                        responses.push(res);
                    }
                    Err(e) => {
                        responses.push(JsonRpcResponse::error(
                            None,
                            -32600,
                            format!("Invalid Request in batch: {e}"),
                            None,
                        ));
                    }
                }
            }
            (StatusCode::OK, Json(json!(responses)))
        }
        Value::Object(_) => match serde_json::from_value::<JsonRpcRequest>(payload) {
            Ok(req) => {
                let res = dispatch_single_rpc(&state, req).await;
                (StatusCode::OK, Json(json!(res)))
            }
            Err(e) => (
                StatusCode::OK,
                Json(json!(JsonRpcResponse::error(
                    None,
                    -32600,
                    format!("Invalid Request: {e}"),
                    None
                ))),
            ),
        },
        _ => (
            StatusCode::OK,
            Json(json!(JsonRpcResponse::error(
                None,
                -32600,
                "Invalid Request: payload must be a JSON object or array".into(),
                None
            ))),
        ),
    }
}

async fn dispatch_single_rpc(state: &Arc<RpcServerState>, req: JsonRpcRequest) -> JsonRpcResponse {
    let id = req.id.clone();
    let method = req.method.as_str();

    match method {
        // ---- Blockchain Methods ----
        "sprx_chainId" => JsonRpcResponse::success(id, json!(state.node.chain_id())),

        "sprx_blockNumber" => JsonRpcResponse::success(id, json!(state.node.height())),

        "sprx_getBlockByNumber" => {
            let param = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first());

            let height = match param {
                Some(Value::Number(num)) => num.as_u64().unwrap_or(state.node.height()),
                Some(Value::String(s)) => {
                    if let Ok(h) = s.parse::<u64>() {
                        h
                    } else if let Ok(h) = u64::from_str_radix(s.trim_start_matches("0x"), 16) {
                        h
                    } else {
                        state.node.height()
                    }
                }
                _ => state.node.height(),
            };

            match state.node.get_block_by_height(height) {
                Some(b) => serialize_block_response(id, &b),
                None => JsonRpcResponse::success(id, Value::Null),
            }
        }

        "sprx_getBlockByHash" => {
            let hash_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match Hash32::from_hex(hash_str) {
                Ok(hash) => match state.node.get_block_by_hash(&hash) {
                    Some(b) => serialize_block_response(id, &b),
                    None => JsonRpcResponse::success(id, Value::Null),
                },
                Err(_) => JsonRpcResponse::error(
                    id,
                    -32602,
                    "Invalid block hash format (expected 64-char hex)".into(),
                    None,
                ),
            }
        }

        "sprx_getBlockHeader" => {
            let param = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first());

            let block = match param {
                Some(Value::Number(num)) => {
                    state.node.get_block_by_height(num.as_u64().unwrap_or(0))
                }
                Some(Value::String(s)) => {
                    if let Ok(h) = s.parse::<u64>() {
                        state.node.get_block_by_height(h)
                    } else if let Ok(hash) = Hash32::from_hex(s) {
                        state.node.get_block_by_hash(&hash)
                    } else {
                        None
                    }
                }
                _ => state.node.get_block_by_height(state.node.height()),
            };

            match block {
                Some(b) => {
                    let hash = Hasher::block_hash(&b.header).unwrap_or(Hash32::ZERO);
                    let res = json!({
                        "hash": hash.to_hex(),
                        "version": b.header.version,
                        "chainId": b.header.chain_id,
                        "height": b.header.height,
                        "timestampUnixSecs": b.header.timestamp_unix_secs,
                        "parentHash": b.header.parent_hash.to_hex(),
                        "proposer": b.header.proposer.to_hex(),
                        "stateRoot": b.header.state_root.to_hex(),
                        "txsRoot": b.header.txs_root.to_hex(),
                        "receiptsRoot": b.header.receipts_root.to_hex(),
                        "validatorSetHash": b.header.validator_set_hash.to_hex(),
                    });
                    JsonRpcResponse::success(id, res)
                }
                None => JsonRpcResponse::success(id, Value::Null),
            }
        }

        "sprx_getBlockTransactionCount" => {
            let param = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first());

            let block = match param {
                Some(Value::Number(num)) => {
                    state.node.get_block_by_height(num.as_u64().unwrap_or(0))
                }
                Some(Value::String(s)) => {
                    if let Ok(h) = s.parse::<u64>() {
                        state.node.get_block_by_height(h)
                    } else if let Ok(hash) = Hash32::from_hex(s) {
                        state.node.get_block_by_hash(&hash)
                    } else {
                        None
                    }
                }
                _ => state.node.get_block_by_height(state.node.height()),
            };

            match block {
                Some(b) => JsonRpcResponse::success(id, json!(b.body.transactions.len())),
                None => JsonRpcResponse::success(id, Value::Null),
            }
        }

        "sprx_getChainInfo" | "sprax_getStatus" => {
            let metrics = state.node.metrics();
            let val_count = state
                .node
                .canonical_validator_set()
                .map(|vs| vs.validators().len())
                .unwrap_or(0);

            let result = json!({
                "chainId": metrics.chain_id,
                "chain_id": metrics.chain_id,
                "height": metrics.height,
                "latestBlockHeight": metrics.height,
                "latestBlockHash": metrics.latest_block_hash.to_hex(),
                "stateRoot": metrics.state_root.to_hex(),
                "connectedPeers": metrics.connected_peers,
                "mempoolPending": metrics.mempool_pending,
                "validatorCount": val_count,
                "syncStatus": metrics.status,
                "isSyncing": metrics.is_syncing,
            });
            JsonRpcResponse::success(id, result)
        }

        "sprx_getSyncStatus" => {
            let height = state.node.height();
            let is_syncing = state.node.is_syncing();
            let val = json!({
                "isSyncing": is_syncing,
                "currentBlock": height,
                "highestBlock": height,
                "startingBlock": 0,
            });
            JsonRpcResponse::success(id, val)
        }

        "sprax_getBlock" => {
            let param = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .cloned();

            let block = match param {
                Some(Value::Number(num)) => {
                    let height = num.as_u64().unwrap_or(0);
                    state.node.get_block_by_height(height)
                }
                Some(Value::String(s)) => {
                    if let Ok(height) = s.parse::<u64>() {
                        state.node.get_block_by_height(height)
                    } else if let Ok(hash) = Hash32::from_hex(&s) {
                        state.node.get_block_by_hash(&hash)
                    } else {
                        None
                    }
                }
                _ => {
                    let latest_height = state.node.height();
                    state.node.get_block_by_height(latest_height)
                }
            };

            match block {
                Some(b) => serialize_block_response(id, &b),
                None => JsonRpcResponse::success(id, Value::Null),
            }
        }

        // ---- Transaction Methods ----
        "sprx_sendRawTransaction" | "sprax_broadcastTx" => {
            let tx_val = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .cloned();

            match tx_val {
                Some(val) => match parse_transaction(val) {
                    Ok(tx) => match state.node.submit_transaction(tx) {
                        Ok(hash) => {
                            let result = json!({
                                "txHash": hash.to_hex(),
                                "tx_hash": hash.to_hex(),
                                "success": true
                            });
                            JsonRpcResponse::success(id, result)
                        }
                        Err(e) => JsonRpcResponse::error(
                            id,
                            -32000,
                            format!("Transaction rejected: {e}"),
                            None,
                        ),
                    },
                    Err(err) => JsonRpcResponse::error(
                        id,
                        -32602,
                        format!("Invalid transaction payload: {err}"),
                        None,
                    ),
                },
                None => JsonRpcResponse::error(
                    id,
                    -32602,
                    "Missing transaction parameter".to_string(),
                    None,
                ),
            }
        }

        "sprx_getTransactionByHash" | "sprax_getTransaction" => {
            let hash_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match Hash32::from_hex(hash_str) {
                Ok(hash) => {
                    if let Some((tx, receipt, height)) = state.node.get_transaction(&hash) {
                        let val = json!({
                            "txHash": hash.to_hex(),
                            "height": height,
                            "isFinalized": true,
                            "isPending": false,
                            "transaction": tx,
                            "receipt": {
                                "txHash": receipt.tx_hash.to_hex(),
                                "height": receipt.height,
                                "success": receipt.success,
                                "gasUsed": receipt.gas_used,
                                "errorMessage": receipt.error_message,
                                "returnData": hex::encode(&receipt.return_data),
                            }
                        });
                        JsonRpcResponse::success(id, val)
                    } else if let Some(tx) = state.node.get_mempool_transaction(&hash) {
                        let val = json!({
                            "txHash": hash.to_hex(),
                            "height": Value::Null,
                            "isFinalized": false,
                            "isPending": true,
                            "transaction": tx,
                            "receipt": Value::Null,
                        });
                        JsonRpcResponse::success(id, val)
                    } else {
                        JsonRpcResponse::success(id, Value::Null)
                    }
                }
                Err(_) => JsonRpcResponse::error(
                    id,
                    -32602,
                    "Invalid transaction hash format".to_string(),
                    None,
                ),
            }
        }

        "sprx_getTransactionReceipt" => {
            let hash_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match Hash32::from_hex(hash_str) {
                Ok(hash) => {
                    if let Some((_tx, receipt, height)) = state.node.get_transaction(&hash) {
                        let val = json!({
                            "txHash": receipt.tx_hash.to_hex(),
                            "height": height,
                            "success": receipt.success,
                            "gasUsed": receipt.gas_used,
                            "errorMessage": receipt.error_message,
                            "returnData": hex::encode(&receipt.return_data),
                        });
                        JsonRpcResponse::success(id, val)
                    } else {
                        JsonRpcResponse::success(id, Value::Null)
                    }
                }
                Err(_) => JsonRpcResponse::error(
                    id,
                    -32602,
                    "Invalid transaction hash format".to_string(),
                    None,
                ),
            }
        }

        "sprx_getTransactionStatus" => {
            let hash_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match Hash32::from_hex(hash_str) {
                Ok(hash) => {
                    if let Some((_tx, receipt, height)) = state.node.get_transaction(&hash) {
                        let status_str = if receipt.success { "finalized" } else { "failed" };
                        let val = json!({
                            "txHash": hash.to_hex(),
                            "status": status_str,
                            "height": height,
                            "isFinalized": true,
                            "isPending": false,
                            "success": receipt.success,
                        });
                        JsonRpcResponse::success(id, val)
                    } else if state.node.get_mempool_transaction(&hash).is_some() {
                        let val = json!({
                            "txHash": hash.to_hex(),
                            "status": "pending",
                            "height": Value::Null,
                            "isFinalized": false,
                            "isPending": true,
                            "success": Value::Null,
                        });
                        JsonRpcResponse::success(id, val)
                    } else {
                        let val = json!({
                            "txHash": hash.to_hex(),
                            "status": "not_found",
                            "height": Value::Null,
                            "isFinalized": false,
                            "isPending": false,
                            "success": Value::Null,
                        });
                        JsonRpcResponse::success(id, val)
                    }
                }
                Err(_) => JsonRpcResponse::error(
                    id,
                    -32602,
                    "Invalid transaction hash format".to_string(),
                    None,
                ),
            }
        }

        "sprx_getTransactionCount" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => match state.node.get_account(&addr) {
                    Ok(acc) => JsonRpcResponse::success(id, json!(acc.nonce)),
                    Err(e) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
                },
                None => {
                    JsonRpcResponse::error(id, -32602, "Invalid address format".to_string(), None)
                }
            }
        }

        "sprx_estimateFee" | "sprax_estimateFee" => {
            let val = json!({
                "fee": "500000000000000", // 0.0005 SPRX
                "gas_limit": 200_000,
                "gas_price": "2500000000",
            });
            JsonRpcResponse::success(id, val)
        }

        "sprx_getPendingTransactions" => {
            let pending = state.node.get_pending_transactions();
            JsonRpcResponse::success(id, json!(pending))
        }

        // ---- Account Methods ----
        "sprx_getBalance" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => match state.node.get_account(&addr) {
                    Ok(acc) => {
                        let val = json!({
                            "address": addr.to_hex(),
                            "balance_atto": acc.balance.as_atto().to_string(),
                            "balance_sprx": acc.balance.to_string(),
                        });
                        JsonRpcResponse::success(id, val)
                    }
                    Err(e) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
                },
                None => {
                    JsonRpcResponse::error(id, -32602, "Invalid address format".to_string(), None)
                }
            }
        }

        "sprx_getAccount" | "sprax_getAccount" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => match state.node.get_account(&addr) {
                    Ok(acc) => {
                        let val = json!({
                            "address": addr.to_hex(),
                            "balance": acc.balance.to_string(),
                            "balance_atto": acc.balance.as_atto().to_string(),
                            "nonce": acc.nonce,
                        });
                        JsonRpcResponse::success(id, val)
                    }
                    Err(e) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
                },
                None => {
                    JsonRpcResponse::error(id, -32602, "Invalid address format".to_string(), None)
                }
            }
        }

        "sprx_getAccountNonce" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => match state.node.get_account(&addr) {
                    Ok(acc) => JsonRpcResponse::success(id, json!({ "nonce": acc.nonce })),
                    Err(e) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
                },
                None => {
                    JsonRpcResponse::error(id, -32602, "Invalid address format".to_string(), None)
                }
            }
        }

        "sprx_getAccountState" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => match state.node.get_account(&addr) {
                    Ok(acc) => {
                        let is_contract = acc.code_hash != Hash32::ZERO;
                        let code_hash_val = if is_contract {
                            Some(acc.code_hash.to_hex())
                        } else {
                            None
                        };
                        let val = json!({
                            "address": addr.to_hex(),
                            "balance": acc.balance.to_string(),
                            "balance_atto": acc.balance.as_atto().to_string(),
                            "nonce": acc.nonce,
                            "code_hash": code_hash_val,
                            "is_contract": is_contract,
                        });
                        JsonRpcResponse::success(id, val)
                    }
                    Err(e) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
                },
                None => {
                    JsonRpcResponse::error(id, -32602, "Invalid address format".to_string(), None)
                }
            }
        }

        // ---- Validator Methods ----
        "sprx_getValidators" | "sprax_getValidators" => {
            let val_set_res = state.node.canonical_validator_set();
            match val_set_res {
                Ok(val_set) => {
                    let vals: Vec<Value> = val_set
                        .validators()
                        .iter()
                        .map(|v| {
                            json!({
                                "address": v.address.to_hex(),
                                "votingPower": v.voting_power,
                                "proposerPriority": v.proposer_priority,
                                "publicKey": hex::encode(&v.public_key),
                                "isRegistered": true,
                                "isEligible": true,
                                "isActive": true,
                                "isJailed": false,
                                "isTombstoned": false,
                            })
                        })
                        .collect();
                    JsonRpcResponse::success(id, json!(vals))
                }
                Err(e) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
            }
        }

        "sprx_getValidator" | "sprax_getValidator" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => {
                    let ledger = state.node.ledger();
                    let ledger_guard = ledger.read();
                    let val_record_res = sprax_core::state::StateAccessor::get_validator_record(
                        ledger_guard.store(),
                        &addr,
                    );
                    let val_stake_res = sprax_core::state::StateAccessor::get_validator_stake(
                        ledger_guard.store(),
                        &addr,
                    );
                    let active_set = ledger_guard.active_validators().unwrap_or_default();

                    match (val_record_res, val_stake_res) {
                        (Ok(Some(record)), Ok(stake)) => {
                            let is_jailed = record.status
                                == sprax_core::state::CanonicalValidatorStatus::Jailed;
                            let is_tombstoned = record.is_tombstoned;
                            let is_active = active_set.iter().any(|v| v.address == addr);
                            let is_eligible = !is_jailed
                                && !is_tombstoned
                                && (stake.tokens >= sprax_core::validator_set::MIN_SELF_STAKE
                                    || is_active);
                            let voting_power = if is_active {
                                (stake.tokens.as_atto() / sprax_types::ATTO_SPRX_PER_SPRX).max(1)
                                    as u64
                            } else {
                                0u64
                            };

                            let result = json!({
                                "operatorAddress": addr.to_hex(),
                                "consensusPubkey": hex::encode(&record.consensus_pubkey),
                                "moniker": record.moniker,
                                "tokens": stake.tokens.to_string(),
                                "votingPower": voting_power,
                                "status": format!("{:?}", record.status),
                                "isRegistered": true,
                                "isEligible": is_eligible,
                                "isActive": is_active,
                                "isJailed": is_jailed,
                                "isTombstoned": is_tombstoned,
                                "jailedUntil": record.jailed_until,
                                "registeredHeight": record.registered_height,
                            });
                            JsonRpcResponse::success(id, result)
                        }
                        (Ok(None), _) => JsonRpcResponse::success(id, Value::Null),
                        (Err(e), _) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
                        (_, Err(e)) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
                    }
                }
                None => {
                    JsonRpcResponse::error(id, -32602, "Invalid address format".to_string(), None)
                }
            }
        }

        "sprx_getValidatorStatus" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => {
                    let ledger = state.node.ledger();
                    let record_res = sprax_core::state::StateAccessor::get_validator_record(
                        ledger.read().store(),
                        &addr,
                    );
                    match record_res {
                        Ok(Some(rec)) => JsonRpcResponse::success(id, json!(format!("{:?}", rec.status))),
                        Ok(None) => JsonRpcResponse::success(id, json!("Unregistered")),
                        Err(e) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
                    }
                }
                None => {
                    JsonRpcResponse::error(id, -32602, "Invalid address format".to_string(), None)
                }
            }
        }

        "sprx_getValidatorVotingPower" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => {
                    let val_set_res = state.node.canonical_validator_set();
                    match val_set_res {
                        Ok(val_set) => {
                            let power = val_set
                                .validators()
                                .iter()
                                .find(|v| v.address == addr)
                                .map(|v| v.voting_power)
                                .unwrap_or(0);
                            JsonRpcResponse::success(id, json!(power))
                        }
                        Err(e) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
                    }
                }
                None => {
                    JsonRpcResponse::error(id, -32602, "Invalid address format".to_string(), None)
                }
            }
        }

        "sprx_getValidatorRewards" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => {
                    let bal = state
                        .node
                        .get_account(&addr)
                        .map(|a| a.balance)
                        .unwrap_or(Amount::ZERO);
                    JsonRpcResponse::success(
                        id,
                        json!({
                            "validator": addr.to_hex(),
                            "rewards": bal.to_string(),
                            "rewards_atto": bal.as_atto().to_string(),
                            "reward_per_block": "2",
                        }),
                    )
                }
                None => {
                    JsonRpcResponse::error(id, -32602, "Invalid address format".to_string(), None)
                }
            }
        }

        // ---- Staking Methods ----
        "sprx_getStakingInfo" | "sprax_getStaking" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => {
                    let staking_arc = state.node.staking();
                    let staking_guard = staking_arc.read();
                    let validator = staking_guard.get_validator(&addr);
                    let delegations = staking_guard.get_delegator_delegations(addr);
                    let unbonding = staking_guard.get_unbonding_entries(&addr);

                    let result = json!({
                        "address": addr.to_hex(),
                        "isValidator": validator.is_some(),
                        "validator": validator.map(|v| json!({
                            "operatorAddress": v.operator_address.to_hex(),
                            "tokens": v.tokens.to_string(),
                            "status": format!("{:?}", v.status),
                            "moniker": v.description.moniker,
                        })),
                        "delegationsCount": delegations.len(),
                        "unbondingCount": unbonding.len(),
                    });
                    JsonRpcResponse::success(id, result)
                }
                None => {
                    JsonRpcResponse::error(id, -32602, "Invalid address format".to_string(), None)
                }
            }
        }

        "sprx_getDelegations" | "sprax_getDelegations" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => {
                    let delegations = state.node.staking().read().get_delegator_delegations(addr);
                    let vals: Vec<Value> = delegations
                        .into_iter()
                        .map(|d| {
                            json!({
                                "delegator": d.delegator_address.to_hex(),
                                "validator": d.validator_address.to_hex(),
                                "shares": d.shares.to_string(),
                                "balance": d.balance.to_string(),
                            })
                        })
                        .collect();
                    JsonRpcResponse::success(id, json!(vals))
                }
                None => {
                    JsonRpcResponse::error(id, -32602, "Invalid address format".to_string(), None)
                }
            }
        }

        "sprx_getUnbondingRequests" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => {
                    let unbonding = state.node.staking().read().get_unbonding_entries(&addr);
                    let vals: Vec<Value> = unbonding
                        .into_iter()
                        .map(|u| {
                            json!({
                                "delegator": u.delegator_address.to_hex(),
                                "validator": u.validator_address.to_hex(),
                                "amount": u.amount.to_string(),
                                "completion_height": u.completion_height,
                            })
                        })
                        .collect();
                    JsonRpcResponse::success(id, json!(vals))
                }
                None => {
                    JsonRpcResponse::error(id, -32602, "Invalid address format".to_string(), None)
                }
            }
        }

        // ---- Network Methods ----
        "sprx_netVersion" => JsonRpcResponse::success(id, json!(state.node.chain_id())),

        "sprx_netPeerCount" => {
            let count = state.node.connected_peers_count();
            JsonRpcResponse::success(id, json!(count))
        }

        "sprx_netPeers" => {
            let peers = state.node.connected_peers();
            let val = peers
                .into_iter()
                .map(|p| {
                    json!({
                        "peerId": p.peer_id.to_string(),
                        "peer_id": p.peer_id.to_string(),
                    })
                })
                .collect::<Vec<_>>();
            JsonRpcResponse::success(id, json!(val))
        }

        "sprx_netListening" => JsonRpcResponse::success(id, json!(true)),

        // ---- Node Methods ----
        "sprx_nodeInfo" => {
            let metrics = state.node.metrics();
            let val = json!({
                "version": "0.1.0",
                "name": "sprax-node",
                "chainId": metrics.chain_id,
                "protocolVersion": 1,
                "height": metrics.height,
                "peers": metrics.connected_peers,
            });
            JsonRpcResponse::success(id, val)
        }

        "sprx_nodeVersion" => JsonRpcResponse::success(id, json!("sprax-node/v0.1.0")),

        "sprx_nodeHealth" => {
            let metrics = state.node.metrics();
            let val = json!({
                "status": "healthy",
                "height": metrics.height,
                "chainId": metrics.chain_id,
                "connectedPeers": metrics.connected_peers,
                "isSyncing": metrics.is_syncing,
            });
            JsonRpcResponse::success(id, val)
        }

        // ---- Contract Query & Management ----
        "sprx_deployContract" => {
            let tx_val = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .cloned()
                .unwrap_or(Value::Null);

            match parse_transaction(tx_val) {
                Ok(tx) => match state.node.submit_transaction(tx) {
                    Ok(hash) => JsonRpcResponse::success(id, json!({ "tx_hash": hash.to_hex() })),
                    Err(e) => JsonRpcResponse::error(id, -32000, format!("Deployment rejected: {e}"), None),
                },
                Err(err) => JsonRpcResponse::error(id, -32602, format!("Invalid deployment transaction: {err}"), None),
            }
        }

        "sprx_callContract" => {
            let tx_val = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .cloned()
                .unwrap_or(Value::Null);

            match parse_transaction(tx_val) {
                Ok(tx) => match state.node.submit_transaction(tx) {
                    Ok(hash) => JsonRpcResponse::success(id, json!({ "tx_hash": hash.to_hex() })),
                    Err(e) => JsonRpcResponse::error(id, -32000, format!("Contract call rejected: {e}"), None),
                },
                Err(err) => JsonRpcResponse::error(id, -32602, format!("Invalid call transaction: {err}"), None),
            }
        }

        "sprx_getContract" => {
            let addr_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => match state.node.get_contract_info(addr) {
                    Ok(Some(info)) => {
                        let val = json!({
                            "address": addr.to_string(),
                            "code_id": info.code_id.to_hex(),
                            "code_id_seq": info.code_id_seq,
                            "creator": info.creator.to_string(),
                            "admin": info.admin.map(|a| a.to_string()),
                            "label": info.label
                        });
                        JsonRpcResponse::success(id, val)
                    }
                    Ok(None) => JsonRpcResponse::success(id, Value::Null),
                    Err(e) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
                },
                None => JsonRpcResponse::error(id, -32602, "Invalid contract address format".into(), None),
            }
        }

        "sprx_getContractCode" => {
            let target_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let code_res = if let Some(addr) = parse_address(target_str) {
                state.node.get_contract_code(addr)
            } else if let Ok(hash) = Hash32::from_hex(target_str) {
                state.node.get_contract_code_by_hash(hash)
            } else {
                return JsonRpcResponse::error(id, -32602, "Invalid address or code hash format".into(), None);
            };

            match code_res {
                Ok(Some(code)) => {
                    let val = json!({
                        "bytecode": hex::encode(&code),
                        "size": code.len(),
                    });
                    JsonRpcResponse::success(id, val)
                }
                Ok(None) => JsonRpcResponse::success(id, Value::Null),
                Err(e) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
            }
        }

        "sprx_getContractStorage" => {
            let params = req.params.as_ref().and_then(Value::as_array);
            let addr_str = params.and_then(|p| p.first()).and_then(Value::as_str).unwrap_or("");
            let key_str = params.and_then(|p| p.get(1)).and_then(Value::as_str).unwrap_or("");

            match parse_address(addr_str) {
                Some(addr) => {
                    let key_bytes = hex::decode(key_str).unwrap_or_else(|_| key_str.as_bytes().to_vec());
                    match state.node.get_contract_storage(addr, &key_bytes) {
                        Ok(Some(val)) => {
                            let resp = json!({
                                "key": key_str,
                                "value": hex::encode(&val),
                            });
                            JsonRpcResponse::success(id, resp)
                        }
                        Ok(None) => JsonRpcResponse::success(id, Value::Null),
                        Err(e) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
                    }
                }
                None => JsonRpcResponse::error(id, -32602, "Invalid contract address format".into(), None),
            }
        }

        "sprx_getContractEvents" => {
            let params = req.params.as_ref().and_then(Value::as_array);
            let addr_str = params.and_then(|p| p.first()).and_then(Value::as_str).unwrap_or("");
            let limit = params.and_then(|p| p.get(1)).and_then(Value::as_u64).unwrap_or(50) as usize;

            match parse_address(addr_str) {
                Some(addr) => {
                    let height = state.node.latest_header().height;
                    let start = height.saturating_sub(100);
                    let mut events = Vec::new();
                    for h in (start..=height).rev() {
                        if let Some(block) = state.node.get_block_by_height(h) {
                            for tx in &block.body.transactions {
                                let tx_hash = Hasher::tx_hash(tx).unwrap_or(Hash32::ZERO);
                                if let Some((_tx, receipt, _h)) = state.node.get_transaction(&tx_hash) {
                                    for ev in &receipt.events {
                                        if ev.contract == addr {
                                            events.push(json!({
                                                "height": h,
                                                "tx_hash": tx_hash.to_hex(),
                                                "type": ev.event_type,
                                                "attributes": ev.attributes,
                                            }));
                                            if events.len() >= limit {
                                                break;
                                            }
                                        }
                                    }
                                }
                                if events.len() >= limit {
                                    break;
                                }
                            }
                        }
                        if events.len() >= limit {
                            break;
                        }
                    }
                    JsonRpcResponse::success(id, json!(events))
                }
                None => JsonRpcResponse::error(id, -32602, "Invalid contract address format".into(), None),
            }
        }

        "sprx_estimateContractGas" => {
            let val = json!({
                "gas_estimated": 85_000,
                "fee_estimated": "212500000000000",
                "gas_price": "2500000000"
            });
            JsonRpcResponse::success(id, val)
        }

        "sprx_getContractReceipt" => {
            let hash_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match Hash32::from_hex(hash_str) {
                Ok(hash) => match state.node.get_transaction(&hash) {
                    Some((_tx, receipt, height)) => {
                        let val = json!({
                            "txHash": receipt.tx_hash.to_hex(),
                            "height": height,
                            "success": receipt.success,
                            "gasUsed": receipt.gas_used,
                            "errorMessage": receipt.error_message,
                            "returnData": hex::encode(&receipt.return_data),
                            "events": receipt.events,
                        });
                        JsonRpcResponse::success(id, val)
                    }
                    None => JsonRpcResponse::success(id, Value::Null),
                },
                Err(_) => JsonRpcResponse::error(id, -32602, "Invalid transaction hash format".into(), None),
            }
        }

        "sprx_queryContract" | "sprax_queryContract" => {
            let params = req.params.as_ref().and_then(Value::as_array);
            let address = params
                .and_then(|p| p.first())
                .and_then(Value::as_str)
                .and_then(parse_address);
            let message = params.and_then(|p| p.get(1));
            let gas = params
                .and_then(|p| p.get(2))
                .and_then(Value::as_u64)
                .unwrap_or(200_000);
            if gas == 0 || gas > 200_000 {
                return JsonRpcResponse::error(
                    id,
                    -32602,
                    "query gas must be between 1 and 200000".into(),
                    None,
                );
            }
            match (address, message) {
                (Some(address), Some(message)) => {
                    static LIMIT: std::sync::OnceLock<Arc<tokio::sync::Semaphore>> =
                        std::sync::OnceLock::new();
                    let limiter = LIMIT
                        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(2)))
                        .clone();
                    let permit = match limiter.try_acquire_owned() {
                        Ok(p) => p,
                        Err(_) => {
                            return JsonRpcResponse::error(
                                id,
                                -32000,
                                "contract query capacity reached".into(),
                                None,
                            )
                        }
                    };
                    let bytes =
                        serde_json::to_vec(message).expect("JSON value serialization cannot fail");
                    let node = state.node.clone();
                    match tokio::task::spawn_blocking(move || {
                        let _permit = permit;
                        node.query_contract(address, &bytes, gas)
                    })
                    .await
                    {
                        Ok(Ok((data, gas_used))) => {
                            JsonRpcResponse::success(id, json!({"data":data,"gasUsed":gas_used}))
                        }
                        Ok(Err(e)) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
                        Err(_) => JsonRpcResponse::error(
                            id,
                            -32603,
                            "contract query worker failed".into(),
                            None,
                        ),
                    }
                }
                _ => JsonRpcResponse::error(
                    id,
                    -32602,
                    "expected [contract_address, query_object, optional_gas]".into(),
                    None,
                ),
            }
        }

        _ => JsonRpcResponse::error(id, -32601, format!("Method not found: {method}"), None),
    }
}

fn serialize_block_response(id: Option<Value>, b: &sprax_types::Block) -> JsonRpcResponse {
    let block_hash = Hasher::block_hash(&b.header).unwrap_or(Hash32::ZERO);
    let val = json!({
        "hash": block_hash.to_hex(),
        "header": {
            "version": b.header.version,
            "chainId": b.header.chain_id,
            "height": b.header.height,
            "timestampUnixSecs": b.header.timestamp_unix_secs,
            "parentHash": b.header.parent_hash.to_hex(),
            "proposer": b.header.proposer.to_hex(),
            "stateRoot": b.header.state_root.to_hex(),
            "txsRoot": b.header.txs_root.to_hex(),
            "receiptsRoot": b.header.receipts_root.to_hex(),
            "validatorSetHash": b.header.validator_set_hash.to_hex(),
        },
        "body": {
            "transactions": b.body.transactions,
        },
        "lastCommit": b.last_commit,
    });
    JsonRpcResponse::success(id, val)
}

// ==========================================
// REST API Handlers (/api/v1/...)
// ==========================================

async fn handle_api_chain_info(State(state): State<Arc<RpcServerState>>) -> impl IntoResponse {
    let metrics = state.node.metrics();
    Json(json!({
        "chain_id": metrics.chain_id,
        "height": metrics.height,
        "latest_block_hash": metrics.latest_block_hash.to_hex(),
        "state_root": metrics.state_root.to_hex(),
        "connected_peers": metrics.connected_peers,
    }))
}

async fn handle_api_chain_status(State(state): State<Arc<RpcServerState>>) -> impl IntoResponse {
    Json(state.node.metrics())
}

async fn handle_api_blocks_list(
    State(state): State<Arc<RpcServerState>>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    let current_height = state.node.height();
    let limit = pagination.limit.unwrap_or(20).min(100);
    let offset = pagination.offset.unwrap_or(0);

    let from_height = if current_height > (offset + (limit as u64)) {
        current_height.saturating_sub(offset + (limit as u64) - 1)
    } else {
        0
    };
    let to_height = current_height.saturating_sub(offset);

    let blocks = state.node.get_blocks_range(from_height, to_height);
    let summaries: Vec<Value> = blocks
        .into_iter()
        .map(|b| {
            let hash = Hasher::block_hash(&b.header).unwrap_or(Hash32::ZERO);
            json!({
                "hash": hash.to_hex(),
                "height": b.header.height,
                "timestamp": b.header.timestamp_unix_secs,
                "proposer": b.header.proposer.to_hex(),
                "transactions_count": b.body.transactions.len(),
            })
        })
        .collect();

    Json(json!({
        "current_height": current_height,
        "count": summaries.len(),
        "blocks": summaries,
    }))
}

async fn handle_api_block_by_height(
    State(state): State<Arc<RpcServerState>>,
    Path(height): Path<u64>,
) -> impl IntoResponse {
    match state.node.get_block_by_height(height) {
        Some(b) => {
            let hash = Hasher::block_hash(&b.header).unwrap_or(Hash32::ZERO);
            (
                StatusCode::OK,
                Json(json!({
                    "hash": hash.to_hex(),
                    "block": b,
                })),
            )
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Block not found" })),
        ),
    }
}

async fn handle_api_block_by_hash(
    State(state): State<Arc<RpcServerState>>,
    Path(hash_str): Path<String>,
) -> impl IntoResponse {
    let hash = match Hash32::from_hex(&hash_str) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid block hash format" })),
            )
        }
    };

    match state.node.get_block_by_hash(&hash) {
        Some(b) => (
            StatusCode::OK,
            Json(json!({
                "hash": hash.to_hex(),
                "block": b,
            })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Block not found" })),
        ),
    }
}

async fn handle_api_tx_submit(
    State(state): State<Arc<RpcServerState>>,
    Json(val): Json<Value>,
) -> impl IntoResponse {
    match parse_transaction(val) {
        Ok(tx) => match state.node.submit_transaction(tx) {
            Ok(hash) => (
                StatusCode::OK,
                Json(json!({
                    "tx_hash": hash.to_hex(),
                    "success": true,
                })),
            ),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": format!("Transaction rejected: {e}"),
                    "success": false,
                })),
            ),
        },
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("Invalid transaction format: {err}"),
                "success": false,
            })),
        ),
    }
}

async fn handle_api_tx_by_hash(
    State(state): State<Arc<RpcServerState>>,
    Path(hash_str): Path<String>,
) -> impl IntoResponse {
    let hash = match Hash32::from_hex(&hash_str) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid tx hash format" })),
            )
        }
    };

    if let Some((tx, receipt, height)) = state.node.get_transaction(&hash) {
        (
            StatusCode::OK,
            Json(json!({
                "hash": hash.to_hex(),
                "height": height,
                "is_finalized": true,
                "is_pending": false,
                "transaction": tx,
                "receipt": receipt,
            })),
        )
    } else if let Some(tx) = state.node.get_mempool_transaction(&hash) {
        (
            StatusCode::OK,
            Json(json!({
                "hash": hash.to_hex(),
                "height": Value::Null,
                "is_finalized": false,
                "is_pending": true,
                "transaction": tx,
                "receipt": Value::Null,
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Transaction not found" })),
        )
    }
}

async fn handle_api_tx_receipt(
    State(state): State<Arc<RpcServerState>>,
    Path(hash_str): Path<String>,
) -> impl IntoResponse {
    let hash = match Hash32::from_hex(&hash_str) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid tx hash format" })),
            )
        }
    };

    match state.node.get_transaction(&hash) {
        Some((_tx, receipt, height)) => (
            StatusCode::OK,
            Json(json!({
                "tx_hash": receipt.tx_hash.to_hex(),
                "height": height,
                "success": receipt.success,
                "gas_used": receipt.gas_used,
                "return_data": hex::encode(&receipt.return_data),
                "error_message": receipt.error_message,
            })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Receipt not found" })),
        ),
    }
}

async fn handle_api_tx_status(
    State(state): State<Arc<RpcServerState>>,
    Path(hash_str): Path<String>,
) -> impl IntoResponse {
    let hash = match Hash32::from_hex(&hash_str) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid tx hash format" })),
            )
        }
    };

    if let Some((_tx, receipt, height)) = state.node.get_transaction(&hash) {
        (
            StatusCode::OK,
            Json(json!({
                "tx_hash": hash.to_hex(),
                "status": if receipt.success { "finalized" } else { "failed" },
                "height": height,
                "is_finalized": true,
                "is_pending": false,
                "success": receipt.success,
            })),
        )
    } else if state.node.get_mempool_transaction(&hash).is_some() {
        (
            StatusCode::OK,
            Json(json!({
                "tx_hash": hash.to_hex(),
                "status": "pending",
                "height": Value::Null,
                "is_finalized": false,
                "is_pending": true,
                "success": Value::Null,
            })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Transaction not found" })),
        )
    }
}

async fn handle_api_account(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
) -> impl IntoResponse {
    let addr = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid address format" })),
            )
        }
    };

    match state.node.get_account(&addr) {
        Ok(acc) => {
            let is_contract = acc.code_hash != Hash32::ZERO;
            (
                StatusCode::OK,
                Json(json!({
                    "address": address_str,
                    "balance": acc.balance.to_string(),
                    "balance_atto": acc.balance.as_atto().to_string(),
                    "nonce": acc.nonce,
                    "is_contract": is_contract,
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn handle_api_account_txs(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
) -> impl IntoResponse {
    let addr = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid address format" })),
            )
        }
    };

    let current_height = state.node.height();
    let scan_start = current_height.saturating_sub(50);
    let blocks = state.node.get_blocks_range(scan_start, current_height);

    let mut txs = Vec::new();
    for b in blocks {
        for tx in b.body.transactions {
            let is_sender = tx.body.sender == addr;
            let is_receiver = tx.body.messages.iter().any(|m| match m {
                TxMessage::Transfer { to, .. } => *to == addr,
                _ => false,
            });
            if is_sender || is_receiver {
                let tx_hash = Hasher::tx_hash(&tx).unwrap_or(Hash32::ZERO);
                txs.push(json!({
                    "tx_hash": tx_hash.to_hex(),
                    "height": b.header.height,
                    "sender": tx.body.sender.to_hex(),
                    "nonce": tx.body.nonce,
                    "fee": tx.body.fee.amount.to_string(),
                }));
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "address": address_str,
            "scanned_blocks": current_height - scan_start + 1,
            "transactions": txs,
        })),
    )
}

async fn handle_api_validators(State(state): State<Arc<RpcServerState>>) -> impl IntoResponse {
    let vals: Vec<Value> = state
        .node
        .canonical_validator_set()
        .map(|vs| {
            vs.validators()
                .iter()
                .map(|v| {
                    json!({
                        "address": v.address.to_hex(),
                        "voting_power": v.voting_power,
                        "proposer_priority": v.proposer_priority,
                        "public_key": hex::encode(&v.public_key),
                        "is_active": true,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    (StatusCode::OK, Json(json!(vals)))
}

async fn handle_api_validator_by_addr(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
) -> impl IntoResponse {
    let addr = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid address format" })),
            )
        }
    };

    let ledger = state.node.ledger();
    let ledger_guard = ledger.read();
    let val_record = sprax_core::state::StateAccessor::get_validator_record(
        ledger_guard.store(),
        &addr,
    );
    let val_stake = sprax_core::state::StateAccessor::get_validator_stake(
        ledger_guard.store(),
        &addr,
    );

    match (val_record, val_stake) {
        (Ok(Some(rec)), Ok(stake)) => (
            StatusCode::OK,
            Json(json!({
                "operator_address": addr.to_hex(),
                "moniker": rec.moniker,
                "tokens": stake.tokens.to_string(),
                "status": format!("{:?}", rec.status),
                "is_jailed": rec.status == sprax_core::state::CanonicalValidatorStatus::Jailed,
                "is_tombstoned": rec.is_tombstoned,
                "registered_height": rec.registered_height,
            })),
        ),
        (Ok(None), _) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Validator not found" })),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Database read error" })),
        ),
    }
}

async fn handle_api_validator_rewards(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
) -> impl IntoResponse {
    let addr = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid address format" })),
            )
        }
    };

    let bal = state
        .node
        .get_account(&addr)
        .map(|a| a.balance)
        .unwrap_or(Amount::ZERO);
    (
        StatusCode::OK,
        Json(json!({
            "validator": address_str,
            "rewards": bal.to_string(),
            "rewards_atto": bal.as_atto().to_string(),
            "reward_per_block": "2",
        })),
    )
}

async fn handle_api_staking_overview(
    State(state): State<Arc<RpcServerState>>,
) -> impl IntoResponse {
    let (total_power, val_count) = match state.node.canonical_validator_set() {
        Ok(vs) => (vs.total_voting_power(), vs.validators().len()),
        Err(_) => (0, 0),
    };

    Json(json!({
        "active_validators_count": val_count,
        "total_voting_power": total_power,
        "min_self_stake_sprx": "1000",
    }))
}

async fn handle_api_delegations(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
) -> impl IntoResponse {
    let addr = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid address format" })),
            )
        }
    };

    let dels = state.node.staking().read().get_delegator_delegations(addr);
    let items: Vec<Value> = dels
        .into_iter()
        .map(|d| {
            json!({
                "delegator": d.delegator_address.to_hex(),
                "validator": d.validator_address.to_hex(),
                "shares": d.shares.to_string(),
                "balance": d.balance.to_string(),
            })
        })
        .collect();

    (StatusCode::OK, Json(json!(items)))
}

async fn handle_api_unbonding(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
) -> impl IntoResponse {
    let addr = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid address format" })),
            )
        }
    };

    let entries = state.node.staking().read().get_unbonding_entries(&addr);
    let items: Vec<Value> = entries
        .into_iter()
        .map(|u| {
            json!({
                "delegator": u.delegator_address.to_hex(),
                "validator": u.validator_address.to_hex(),
                "amount": u.amount.to_string(),
                "completion_height": u.completion_height,
            })
        })
        .collect();

    (StatusCode::OK, Json(json!(items)))
}

async fn handle_api_network_status(State(state): State<Arc<RpcServerState>>) -> impl IntoResponse {
    let metrics = state.node.metrics();
    Json(json!({
        "status": metrics.status,
        "connected_peers": metrics.connected_peers,
        "chain_id": metrics.chain_id,
        "height": metrics.height,
        "is_syncing": metrics.is_syncing,
    }))
}

async fn handle_api_network_peers(State(state): State<Arc<RpcServerState>>) -> impl IntoResponse {
    let peers = state.node.connected_peers();
    let items: Vec<Value> = peers
        .into_iter()
        .map(|p| {
            json!({
                "peer_id": p.peer_id.to_string(),
            })
        })
        .collect();
    Json(json!({
        "count": items.len(),
        "peers": items,
    }))
}

async fn handle_api_network_sync(State(state): State<Arc<RpcServerState>>) -> impl IntoResponse {
    let height = state.node.height();
    let is_syncing = state.node.is_syncing();
    Json(json!({
        "is_syncing": is_syncing,
        "current_block": height,
        "highest_block": height,
    }))
}

// ==========================================
// Legacy Convenience Handlers
// ==========================================

async fn handle_health(State(state): State<Arc<RpcServerState>>) -> Json<Value> {
    let metrics = state.node.metrics();
    Json(json!({
        "status": "healthy",
        "chainId": metrics.chain_id,
        "height": metrics.height,
        "connectedPeers": metrics.connected_peers,
    }))
}

async fn handle_status(State(state): State<Arc<RpcServerState>>) -> impl IntoResponse {
    Json(state.node.metrics())
}

async fn handle_account_balance(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
) -> impl IntoResponse {
    let addr = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid address format" })),
            )
        }
    };

    match state.node.get_account(&addr) {
        Ok(acc) => (
            StatusCode::OK,
            Json(json!({
                "address": address_str,
                "balance_atto": acc.balance.as_atto().to_string(),
                "nonce": acc.nonce,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn handle_account_nonce(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
) -> impl IntoResponse {
    let addr = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid address format" })),
            )
        }
    };

    match state.node.get_account(&addr) {
        Ok(acc) => (
            StatusCode::OK,
            Json(json!({
                "address": address_str,
                "nonce": acc.nonce,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn handle_contract_query(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
    Json(val): Json<Value>,
) -> impl IntoResponse {
    let address = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid contract address", "success": false })),
            )
        }
    };

    let (query_val, gas) = if let Some(obj) = val.as_object() {
        if obj.contains_key("query_data") || obj.contains_key("msg") {
            let msg = obj
                .get("query_data")
                .or_else(|| obj.get("msg"))
                .unwrap()
                .clone();
            let gas = obj.get("gas").and_then(Value::as_u64).unwrap_or(200_000);
            (msg, gas)
        } else {
            (val, 200_000)
        }
    } else {
        (val, 200_000)
    };

    if gas == 0 || gas > 200_000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Query gas must be between 1 and 200000", "success": false })),
        );
    }

    let bytes = match query_val {
        Value::String(ref s) => {
            if let Ok(b) = hex::decode(s.trim_start_matches("0x")) {
                b
            } else {
                s.as_bytes().to_vec()
            }
        }
        _ => serde_json::to_vec(&query_val).unwrap_or_default(),
    };

    let node = state.node.clone();
    match tokio::task::spawn_blocking(move || node.query_contract(address, &bytes, gas)).await {
        Ok(Ok((data, gas_used))) => {
            let decoded: Value = serde_json::from_slice(&data)
                .unwrap_or_else(|_| json!({ "raw_hex": hex::encode(&data) }));
            (
                StatusCode::OK,
                Json(json!({
                    "data": decoded,
                    "raw_bytes": data,
                    "gas_used": gas_used,
                    "success": true
                })),
            )
        }
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("Contract query failed: {e}"),
                "success": false
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Contract query worker failed",
                "success": false
            })),
        ),
    }
}

async fn handle_api_list_contracts(
    State(state): State<Arc<RpcServerState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(20);
    let offset = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    match state.node.list_contracts(limit, offset) {
        Ok(list) => {
            let res: Vec<Value> = list
                .into_iter()
                .map(|(addr, info)| {
                    json!({
                        "address": addr.to_string(),
                        "code_id": info.code_id.to_hex(),
                        "code_id_seq": info.code_id_seq,
                        "creator": info.creator.to_string(),
                        "admin": info.admin.map(|a| a.to_string()),
                        "label": info.label
                    })
                })
                .collect();
            (StatusCode::OK, Json(json!({ "contracts": res, "total": res.len() })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn handle_api_contract_by_addr(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
) -> impl IntoResponse {
    let address = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid contract address" })),
            )
        }
    };
    match state.node.get_contract_info(address) {
        Ok(Some(info)) => (
            StatusCode::OK,
            Json(json!({
                "address": address.to_string(),
                "code_id": info.code_id.to_hex(),
                "code_id_seq": info.code_id_seq,
                "creator": info.creator.to_string(),
                "admin": info.admin.map(|a| a.to_string()),
                "label": info.label
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Contract not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn handle_api_contract_code(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
) -> impl IntoResponse {
    let address = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid contract address" })),
            )
        }
    };
    match state.node.get_contract_code(address) {
        Ok(Some(code)) => (
            StatusCode::OK,
            Json(json!({
                "address": address.to_string(),
                "code_bytes": code.len(),
                "bytecode_hex": hex::encode(&code),
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Contract code not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn handle_api_contract_storage(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let address = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid contract address" })),
            )
        }
    };
    if let Some(key_hex) = params.get("key") {
        let key_bytes = match hex::decode(key_hex) {
            Ok(k) => k,
            Err(_) => key_hex.as_bytes().to_vec(),
        };
        match state.node.get_contract_storage(address, &key_bytes) {
            Ok(Some(val)) => (
                StatusCode::OK,
                Json(json!({
                    "address": address.to_string(),
                    "key": key_hex,
                    "value_hex": hex::encode(&val),
                })),
            ),
            Ok(None) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Storage key not found" })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            ),
        }
    } else {
        let limit = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
        let prefix = params.get("prefix").map(|p| p.as_bytes()).unwrap_or(b"");
        match state.node.get_contract_storage_prefix(address, prefix, limit) {
            Ok(records) => {
                let items: Vec<Value> = records
                    .into_iter()
                    .map(|(k, v)| {
                        json!({
                            "key_hex": hex::encode(&k),
                            "key_utf8": String::from_utf8(k).ok(),
                            "value_hex": hex::encode(&v),
                        })
                    })
                    .collect();
                (StatusCode::OK, Json(json!({ "address": address.to_string(), "storage": items })))
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            ),
        }
    }
}

async fn handle_api_contract_events(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
) -> impl IntoResponse {
    let address = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid contract address" })),
            )
        }
    };
    let height = state.node.latest_header().height;
    let start = height.saturating_sub(100);
    let mut contract_events = Vec::new();
    for h in (start..=height).rev() {
        if let Some(block) = state.node.get_block_by_height(h) {
            for tx in &block.body.transactions {
                let tx_hash = Hasher::tx_hash(tx).unwrap_or(Hash32::ZERO);
                if let Some((_tx, receipt, _h)) = state.node.get_transaction(&tx_hash) {
                    for ev in &receipt.events {
                        if ev.contract == address {
                            contract_events.push(json!({
                                "block_height": h,
                                "tx_hash": tx_hash.to_hex(),
                                "event_type": ev.event_type,
                                "attributes": ev.attributes,
                            }));
                        }
                    }
                }
            }
        }
    }
    (StatusCode::OK, Json(json!({ "address": address.to_string(), "events": contract_events })))
}

async fn handle_api_contract_txs(
    State(state): State<Arc<RpcServerState>>,
    Path(address_str): Path<String>,
) -> impl IntoResponse {
    let address = match parse_address(&address_str) {
        Some(a) => a,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid contract address" })),
            )
        }
    };
    let height = state.node.latest_header().height;
    let start = height.saturating_sub(100);
    let mut contract_txs = Vec::new();
    for h in (start..=height).rev() {
        if let Some(block) = state.node.get_block_by_height(h) {
            for tx in &block.body.transactions {
                let mut touches_contract = false;
                for msg in &tx.body.messages {
                    match msg {
                        TxMessage::ContractCall { contract, .. } if *contract == address => {
                            touches_contract = true;
                        }
                        TxMessage::MigrateContract { contract, .. } if *contract == address => {
                            touches_contract = true;
                        }
                        TxMessage::UpdateAdmin { contract, .. } if *contract == address => {
                            touches_contract = true;
                        }
                        _ => {}
                    }
                }
                if touches_contract {
                    contract_txs.push(json!({
                        "tx_hash": Hasher::tx_hash(tx).unwrap_or(Hash32::ZERO).to_hex(),
                        "block_height": h,
                        "sender": tx.body.sender.to_string(),
                        "fee": tx.body.fee.amount.to_string(),
                    }));
                }
            }
        }
    }
    (StatusCode::OK, Json(json!({ "address": address.to_string(), "transactions": contract_txs })))
}

async fn handle_api_contract_deploy(
    State(state): State<Arc<RpcServerState>>,
    Json(val): Json<Value>,
) -> impl IntoResponse {
    handle_rest_broadcast(State(state), Json(val)).await
}

async fn handle_api_contract_call(
    State(state): State<Arc<RpcServerState>>,
    Path(_address_str): Path<String>,
    Json(val): Json<Value>,
) -> impl IntoResponse {
    handle_rest_broadcast(State(state), Json(val)).await
}

async fn handle_rest_broadcast(
    State(state): State<Arc<RpcServerState>>,
    Json(val): Json<Value>,
) -> impl IntoResponse {
    match parse_transaction(val) {
        Ok(tx) => match state.node.submit_transaction(tx) {
            Ok(hash) => (
                StatusCode::OK,
                Json(json!({
                    "tx_hash": hash.to_hex(),
                    "success": true,
                })),
            ),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": format!("Transaction rejected: {e}"),
                    "success": false,
                })),
            ),
        },
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("Invalid transaction format: {err}"),
                "success": false,
            })),
        ),
    }
}

async fn handle_rest_get_tx(
    State(state): State<Arc<RpcServerState>>,
    Path(hash_str): Path<String>,
) -> impl IntoResponse {
    let hash = match Hash32::from_hex(&hash_str) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid tx hash format" })),
            )
        }
    };

    match state.node.get_transaction(&hash) {
        Some((tx, receipt, height)) => (
            StatusCode::OK,
            Json(json!({
                "hash": hash.to_hex(),
                "height": height,
                "transaction": tx,
                "receipt": {
                    "success": receipt.success,
                    "gas_used": receipt.gas_used,
                    "return_data": hex::encode(&receipt.return_data),
                    "logs": receipt.error_message.into_iter().collect::<Vec<_>>(),
                }
            })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Transaction not found" })),
        ),
    }
}

async fn handle_rest_latest_block(State(state): State<Arc<RpcServerState>>) -> impl IntoResponse {
    let height = state.node.height();
    match state.node.get_block_by_height(height) {
        Some(b) => {
            let hash = Hasher::block_hash(&b.header).unwrap_or(Hash32::ZERO);
            (
                StatusCode::OK,
                Json(json!({
                    "hash": hash.to_hex(),
                    "block": b,
                })),
            )
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "No blocks found" })),
        ),
    }
}

async fn handle_rest_get_block(
    State(state): State<Arc<RpcServerState>>,
    Path(param): Path<String>,
) -> impl IntoResponse {
    let block = if let Ok(height) = param.parse::<u64>() {
        state.node.get_block_by_height(height)
    } else if let Ok(hash) = Hash32::from_hex(&param) {
        state.node.get_block_by_hash(&hash)
    } else {
        None
    };

    match block {
        Some(b) => {
            let hash = Hasher::block_hash(&b.header).unwrap_or(Hash32::ZERO);
            (
                StatusCode::OK,
                Json(json!({
                    "hash": hash.to_hex(),
                    "block": b,
                })),
            )
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Block not found" })),
        ),
    }
}

// ==========================================
// Parsing Helpers
// ==========================================

fn parse_address(s: &str) -> Option<Address> {
    Address::parse(s.trim()).ok()
}

fn parse_transaction(val: Value) -> Result<Transaction, String> {
    // 1. Try direct serde deserialization
    if let Ok(tx) = serde_json::from_value::<Transaction>(val.clone()) {
        return Ok(tx);
    }

    // 2. If it's a raw hex string, decode hex and try deserialization
    if let Value::String(ref hex_str) = val {
        let clean = hex_str.trim().trim_start_matches("0x");
        if let Ok(bytes) = hex::decode(clean) {
            if let Ok(tx) = serde_json::from_slice::<Transaction>(&bytes) {
                return Ok(tx);
            }
        }
    }

    // 3. Parse flexible camelCase / snake_case wallet format
    let body_obj = val.get("body").ok_or("Missing 'body' field")?;

    let chain_id_str = body_obj
        .get("chainId")
        .or_else(|| body_obj.get("chain_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("sprax-devnet-1");
    let chain_id = ChainId::new(chain_id_str).map_err(|e| e.to_string())?;

    let sender_str = body_obj
        .get("sender")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'body.sender'")?;
    let sender = parse_address(sender_str).ok_or("Invalid sender address")?;

    let nonce = body_obj.get("nonce").and_then(|v| v.as_u64()).unwrap_or(0);

    let memo = body_obj
        .get("memo")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let timeout_height = body_obj
        .get("timeout_height")
        .or_else(|| body_obj.get("timeoutHeight"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let fee_obj = body_obj.get("fee");
    let fee = if let Some(f) = fee_obj {
        let fee_amt_str = f
            .get("amount")
            .or_else(|| f.get("amountAtto"))
            .and_then(|v| v.as_str())
            .unwrap_or("500000000000000");
        let gas_limit = f
            .get("gas_limit")
            .or_else(|| f.get("gasLimit"))
            .and_then(|v| v.as_u64())
            .unwrap_or(200_000);
        let amount = Amount::from_atto_str(fee_amt_str).map_err(|e| e.to_string())?;
        TxFee {
            amount,
            gas_limit,
            priority_fee: Amount::ZERO,
        }
    } else {
        TxFee::default()
    };

    let mut messages = Vec::new();
    if let Some(msgs_arr) = body_obj.get("messages").and_then(|v| v.as_array()) {
        for msg_val in msgs_arr {
            if let Ok(msg) = serde_json::from_value::<TxMessage>(msg_val.clone()) {
                messages.push(msg);
                continue;
            }
            let msg_type = msg_val
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("Transfer");
            if msg_type == "Transfer" || msg_type == "transfer" {
                let to_str = msg_val
                    .get("to")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'to' in Transfer message")?;
                let to = parse_address(to_str).ok_or("Invalid 'to' address")?;
                let amt_str = msg_val
                    .get("amount")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'amount' in Transfer message")?;
                let amount = Amount::from_atto_str(amt_str).map_err(|e| e.to_string())?;
                messages.push(TxMessage::Transfer { to, amount });
            } else if msg_type == "ContractCall" || msg_type == "contract_call" {
                let contract_str = msg_val
                    .get("contract")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'contract' in ContractCall message")?;
                let contract = parse_address(contract_str).ok_or("Invalid 'contract' address")?;
                let data_val = msg_val
                    .get("data")
                    .ok_or("Missing 'data' in ContractCall message")?;
                let data = match data_val {
                    Value::String(s) => {
                        if let Ok(b) = hex::decode(s.trim_start_matches("0x")) {
                            b
                        } else {
                            s.as_bytes().to_vec()
                        }
                    }
                    Value::Array(_) => serde_json::from_value::<Vec<u8>>(data_val.clone())
                        .map_err(|e| format!("Invalid data byte array: {e}"))?,
                    _ => serde_json::to_vec(data_val).unwrap_or_default(),
                };
                let funds = if let Some(funds_str) = msg_val.get("funds").and_then(|v| v.as_str()) {
                    Amount::from_atto_str(funds_str).map_err(|e| e.to_string())?
                } else {
                    Amount::ZERO
                };
                messages.push(TxMessage::ContractCall {
                    contract,
                    data,
                    funds,
                });
            } else if msg_type == "StoreCode" || msg_type == "store_code" {
                let code_val = msg_val
                    .get("wasm_bytecode")
                    .or_else(|| msg_val.get("bytecode"))
                    .ok_or("Missing 'wasm_bytecode' in StoreCode message")?;
                let wasm_bytecode = match code_val {
                    Value::String(s) => {
                        if let Ok(b) = hex::decode(s.trim_start_matches("0x")) {
                            b
                        } else {
                            s.as_bytes().to_vec()
                        }
                    }
                    Value::Array(_) => serde_json::from_value::<Vec<u8>>(code_val.clone())
                        .map_err(|e| format!("Invalid bytecode array: {e}"))?,
                    _ => return Err("Invalid bytecode format".into()),
                };
                messages.push(TxMessage::StoreCode { wasm_bytecode });
            } else if msg_type == "InstantiateContract" || msg_type == "instantiate_contract" || msg_type == "instantiate" {
                let code_id_str = msg_val
                    .get("code_id")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'code_id' in InstantiateContract message")?;
                let code_id = Hash32::from_hex(code_id_str).map_err(|e| format!("Invalid code_id hash: {e}"))?;
                let msg_val = msg_val.get("msg").ok_or("Missing 'msg' in InstantiateContract message")?;
                let msg = match msg_val {
                    Value::String(s) => {
                        if let Ok(b) = hex::decode(s.trim_start_matches("0x")) {
                            b
                        } else {
                            s.as_bytes().to_vec()
                        }
                    }
                    _ => serde_json::to_vec(msg_val).unwrap_or_default(),
                };
                let funds = if let Some(funds_str) = msg_val.get("funds").and_then(|v| v.as_str()) {
                    Amount::from_atto_str(funds_str).map_err(|e| e.to_string())?
                } else {
                    Amount::ZERO
                };
                let label = msg_val
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("contract")
                    .to_string();
                let admin = msg_val
                    .get("admin")
                    .and_then(|v| v.as_str())
                    .and_then(parse_address);
                messages.push(TxMessage::InstantiateContract {
                    code_id,
                    msg,
                    funds,
                    label,
                    admin,
                });
            } else if msg_type == "MigrateContract" || msg_type == "migrate_contract" {
                let contract_str = msg_val
                    .get("contract")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'contract' in MigrateContract message")?;
                let contract = parse_address(contract_str).ok_or("Invalid 'contract' address")?;
                let new_code_id_str = msg_val
                    .get("new_code_id")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'new_code_id' in MigrateContract message")?;
                let new_code_id = Hash32::from_hex(new_code_id_str).map_err(|e| format!("Invalid new_code_id: {e}"))?;
                let msg_val = msg_val.get("msg").ok_or("Missing 'msg' in MigrateContract message")?;
                let msg = match msg_val {
                    Value::String(s) => {
                        if let Ok(b) = hex::decode(s.trim_start_matches("0x")) {
                            b
                        } else {
                            s.as_bytes().to_vec()
                        }
                    }
                    _ => serde_json::to_vec(msg_val).unwrap_or_default(),
                };
                messages.push(TxMessage::MigrateContract {
                    contract,
                    new_code_id,
                    msg,
                });
            } else if msg_type == "UpdateAdmin" || msg_type == "update_admin" {
                let contract_str = msg_val
                    .get("contract")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'contract' in UpdateAdmin message")?;
                let contract = parse_address(contract_str).ok_or("Invalid 'contract' address")?;
                let new_admin = msg_val
                    .get("new_admin")
                    .and_then(|v| v.as_str())
                    .and_then(parse_address);
                messages.push(TxMessage::UpdateAdmin {
                    contract,
                    new_admin,
                });
            }
        }
    }

    if messages.is_empty() {
        return Err("Transaction messages cannot be empty".into());
    }

    let key_type = match val
        .get("keyType")
        .or_else(|| val.get("key_type"))
        .and_then(|v| v.as_str())
        .unwrap_or("Ed25519")
    {
        "Secp256k1" | "secp256k1" => KeyType::Secp256k1,
        _ => KeyType::Ed25519,
    };

    let pubkey_bytes = parse_bytes_field(val.get("publicKey").or_else(|| val.get("public_key")))?;
    let sig_bytes = parse_bytes_field(val.get("signature"))?;

    let body = TxBody {
        chain_id,
        sender,
        nonce,
        messages,
        fee,
        memo,
        timeout_height,
    };

    Transaction::new(body, key_type, pubkey_bytes, sig_bytes).map_err(|e| e.to_string())
}

fn parse_bytes_field(val: Option<&Value>) -> Result<Vec<u8>, String> {
    match val {
        Some(Value::String(s)) => {
            let s_clean = s.trim_start_matches("0x");
            hex::decode(s_clean).map_err(|e| format!("Invalid hex: {e}"))
        }
        Some(Value::Array(arr)) => {
            let bytes: Result<Vec<u8>, _> = arr
                .iter()
                .map(|v| {
                    v.as_u64()
                        .and_then(|n| u8::try_from(n).ok())
                        .ok_or_else(|| "Invalid byte value in array".to_string())
                })
                .collect();
            bytes
        }
        _ => Err("Missing or invalid byte field".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;

    #[tokio::test]
    async fn wallet_rest_routes_accept_sdk_envelopes_and_report_atomic_balances() {
        let fixtures: Vec<Value> = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/wallet-transactions.json"
        ))
        .unwrap();
        for fixture in fixtures {
            let tx = parse_transaction(fixture["signed"].clone()).unwrap();
            let dir = tempfile::tempdir().unwrap();
            let mut genesis = sprax_core::GenesisConfig::default_development();
            genesis.accounts.push(sprax_core::GenesisAccount {
                name: "SDK sender".into(),
                address: tx.body.sender,
                initial_balance: Amount::from_sprx_whole(10).unwrap(),
            });
            genesis
                .save_to_file(&dir.path().join("genesis.json"))
                .unwrap();
            let node = NodeService::new_or_load(dir.path().to_path_buf()).unwrap();
            let router = RpcServer::router(node.clone());
            let request = axum::http::Request::builder()
                .method("POST")
                .uri("/txs/broadcast")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_vec(&fixture["signed"]).unwrap(),
                ))
                .unwrap();
            let response = router.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let bytes = axum::body::to_bytes(response.into_body(), 4096)
                .await
                .unwrap();
            let accepted: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(accepted["success"], true);
            node.mine_block(Address::ZERO).unwrap();
            let receiver = match tx.body.messages[0] {
                TxMessage::Transfer { to, .. } => to,
                _ => panic!("transfer fixture"),
            };
            let request = axum::http::Request::builder()
                .uri(format!("/accounts/{receiver}/balance"))
                .body(axum::body::Body::empty())
                .unwrap();
            let response = router.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let bytes = axum::body::to_bytes(response.into_body(), 4096)
                .await
                .unwrap();
            let balance: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(balance["balance_atto"], "1000000000000000001");
            let request = axum::http::Request::builder()
                .uri(format!("/txs/{}", accepted["tx_hash"].as_str().unwrap()))
                .body(axum::body::Body::empty())
                .unwrap();
            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let bytes = axum::body::to_bytes(response.into_body(), 16384)
                .await
                .unwrap();
            let receipt: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(receipt["receipt"]["success"], true);
        }
    }

    #[test]
    fn wallet_sdk_signatures_execute_on_the_rust_ledger() {
        let fixtures: Vec<Value> = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/wallet-transactions.json"
        ))
        .unwrap();
        for fixture in fixtures {
            let tx = parse_transaction(fixture["signed"].clone()).unwrap();
            let expected: Vec<u8> = serde_json::from_value(fixture["sign_bytes"].clone()).unwrap();
            assert!(
                tx.sign_bytes().unwrap() == expected
                    || serde_json::to_vec(&tx.body).unwrap() == expected,
                "SDK signing bytes must match Rust serialization"
            );
            assert_eq!(
                parse_address(&tx.body.sender.to_bech32().unwrap()),
                Some(tx.body.sender)
            );
            let mut genesis = sprax_core::GenesisConfig::default_development();
            genesis.accounts.push(sprax_core::GenesisAccount {
                name: "SDK sender".into(),
                address: tx.body.sender,
                initial_balance: Amount::from_sprx_whole(10).unwrap(),
            });
            let mut ledger = sprax_core::ChainLedger::init_from_genesis(genesis).unwrap();
            ledger.submit_transaction(tx.clone()).unwrap();
            let block = ledger.mine_block(Address::ZERO).unwrap();
            assert_eq!(block.body.transactions.len(), 1);
            let recipient = match tx.body.messages[0] {
                TxMessage::Transfer { to, .. } => to,
                _ => panic!("fixture must transfer funds"),
            };
            assert_eq!(
                ledger.get_account(&recipient).unwrap().balance.as_atto(),
                1_000_000_000_000_000_001
            );
            assert_eq!(ledger.get_account(&tx.body.sender).unwrap().nonce, 1);
        }
    }

    #[tokio::test]
    async fn test_json_rpc_status_and_account_handlers() {
        let temp_dir = tempfile::tempdir().unwrap();
        let service = NodeService::new_or_load(temp_dir.path().to_path_buf()).unwrap();
        let state = Arc::new(RpcServerState {
            node: service.clone(),
            rate_limiter: Arc::new(RateLimiter::default()),
        });

        // Test sprx_chainId and sprx_blockNumber
        let req1 = JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(1)),
            method: "sprx_chainId".into(),
            params: None,
        };
        let res1 = dispatch_single_rpc(&state, req1).await;
        assert_eq!(res1.result, Some(json!("sprax-devnet-1")));

        let req2 = JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(2)),
            method: "sprx_blockNumber".into(),
            params: None,
        };
        let res2 = dispatch_single_rpc(&state, req2).await;
        assert_eq!(res2.result, Some(json!(0)));

        // Test health endpoint
        let health_res = handle_health(State(state.clone())).await;
        assert_eq!(
            health_res.0.get("status").and_then(|v| v.as_str()),
            Some("healthy")
        );
    }
}
