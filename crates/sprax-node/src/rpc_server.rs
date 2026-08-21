use crate::{error::NodeError, service::NodeService};
use axum::{
    extract::{Path, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sprax_crypto::Hasher;
use sprax_types::{
    Address, Amount, ChainId, Hash32, KeyType, Transaction, TxBody, TxFee, TxMessage,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct RpcServerState {
    pub node: NodeService,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: Option<String>,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
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

impl RpcServer {
    pub async fn start(node: NodeService, port: u16) -> Result<(), NodeError> {
        let state = Arc::new(RpcServerState { node });

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ]);

        let app = Router::new()
            // JSON-RPC entry points
            .route("/", post(handle_json_rpc))
            .route("/rpc", post(handle_json_rpc))
            // REST convenience endpoints for Wallet SDK and Explorer
            .route("/health", get(handle_health))
            .route("/status", get(handle_status))
            .route("/accounts/:address/balance", get(handle_account_balance))
            .route("/accounts/:address/nonce", get(handle_account_nonce))
            .route("/txs/broadcast", post(handle_rest_broadcast))
            .route("/txs/:hash", get(handle_rest_get_tx))
            .route("/blocks/latest", get(handle_rest_latest_block))
            .route("/blocks/:height_or_hash", get(handle_rest_get_block))
            .layer(cors)
            .with_state(state);

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            NodeError::RuntimeError(format!("failed to bind JSON-RPC server on {addr}: {e}"))
        })?;

        info!("SPRX JSON-RPC & REST HTTP Server listening on http://0.0.0.0:{port}");

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                error!("RPC server error: {e}");
            }
        });

        Ok(())
    }
}

// ==========================================
// JSON-RPC Handlers
// ==========================================

async fn handle_json_rpc(
    State(state): State<Arc<RpcServerState>>,
    Json(req): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    let id = req.id.clone();
    let method = req.method.as_str();

    let res = match method {
        "sprax_getStatus" => {
            let metrics = state.node.metrics();
            let val_count = state
                .node
                .staking()
                .read()
                .get_active_validator_set()
                .map(|vs| vs.validators().len())
                .unwrap_or(0);

            let result = json!({
                "chainId": metrics.chain_id,
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
                Some(b) => {
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
                None => JsonRpcResponse::success(id, Value::Null),
            }
        }

        "sprax_getTransaction" => {
            let hash_str = req
                .params
                .as_ref()
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match Hash32::from_hex(hash_str) {
                Ok(hash) => match state.node.get_transaction(&hash) {
                    Some((tx, receipt, height)) => {
                        let val = json!({
                            "txHash": hash.to_hex(),
                            "height": height,
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
                    }
                    None => JsonRpcResponse::success(id, Value::Null),
                },
                Err(_) => JsonRpcResponse::error(
                    id,
                    -32602,
                    "Invalid transaction hash format".to_string(),
                    None,
                ),
            }
        }

        "sprax_getAccount" => {
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

        "sprax_estimateFee" => {
            let val = json!({
                "fee": "500000000000000", // 0.0005 SPRX
                "gas_limit": 200_000,
            });
            JsonRpcResponse::success(id, val)
        }

        "sprax_broadcastTx" => {
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

        "sprax_getValidators" => {
            let val_set_res = state.node.staking().read().get_active_validator_set();
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
                            })
                        })
                        .collect();
                    JsonRpcResponse::success(id, json!(vals))
                }
                Err(e) => JsonRpcResponse::error(id, -32000, e.to_string(), None),
            }
        }

        "sprax_getDelegations" => {
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

        "sprax_getStaking" => {
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

        _ => JsonRpcResponse::error(id, -32601, format!("Method not found: {method}"), None),
    };

    Json(res)
}

// ==========================================
// REST Handlers
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
                "balance_atto": acc.balance.to_string(),
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
    let s = s.trim();
    if s.starts_with("sprx") {
        Address::from_bech32(s).ok()
    } else {
        Address::from_hex(s).ok()
    }
}

fn parse_transaction(val: Value) -> Result<Transaction, String> {
    // 1. Try direct serde deserialization
    if let Ok(tx) = serde_json::from_value::<Transaction>(val.clone()) {
        return Ok(tx);
    }

    // 2. Parse flexible camelCase / snake_case wallet format
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

    #[tokio::test]
    async fn test_json_rpc_status_and_account_handlers() {
        let temp_dir = tempfile::tempdir().unwrap();
        let service = NodeService::new_or_load(temp_dir.path().to_path_buf()).unwrap();
        let state = Arc::new(RpcServerState {
            node: service.clone(),
        });

        // Test sprax_getStatus
        let req = JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(1)),
            method: "sprax_getStatus".into(),
            params: None,
        };
        let res = handle_json_rpc(State(state.clone()), Json(req)).await;
        assert!(res.0.result.is_some());
        assert_eq!(service.chain_id(), "sprax-devnet-1");

        // Test health endpoint
        let health_res = handle_health(State(state.clone())).await;
        assert_eq!(
            health_res.0.get("status").and_then(|v| v.as_str()),
            Some("healthy")
        );
    }
}
