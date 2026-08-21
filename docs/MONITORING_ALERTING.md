# SPRX Protocol: Monitoring, Metrics & Alerting
**Document Version:** 1.0.0  
**Target:** SRE, Validator Operators, Network Health

---

## 1. Prometheus Metric Instrumentation

The SPRX Node exposes Prometheus metrics at `:26660/metrics`:

| Metric Name | Type | Description |
| :--- | :--- | :--- |
| `sprax_validator_uptime` | Gauge | Percent of blocks signed by validator ($0.0 - 100.0\%$) |
| `sprax_block_height` | Counter | Latest committed block height |
| `sprax_block_production_seconds` | Histogram | Round trip block creation and consensus latency |
| `sprax_connected_peers` | Gauge | Count of active P2P TCP connections |
| `sprax_total_transactions` | Counter | Cumulative processed transactions |
| `sprax_mempool_size` | Gauge | Unconfirmed transactions currently in mempool |
| `sprax_rpc_requests_total` | Counter | Cumulative incoming JSON-RPC queries |
| `sprax_system_cpu_usage` | Gauge | Host CPU utilization percentage |
| `sprax_system_memory_bytes` | Gauge | Node memory RSS bytes |
| `sprax_sync_status` | Gauge | $1 = \text{Synced}, 0 = \text{Catching Up}$ |

---

## 2. Alerting Rules

### 2.1 Critical Alerts (PagerDuty / Slack)
1. **Validator Liveness Drop**:
   - Condition: `sprax_validator_uptime < 95%` for $> 5\text{ minutes}$.
   - Action: Check validator key storage and network connection to prevent jailing.
2. **Consensus Halt**:
   - Condition: `rate(sprax_block_height[2m]) == 0`.
   - Action: Investigate Byzantine equivocations or multi-node network partition.
3. **Mempool Congestion**:
   - Condition: `sprax_mempool_size > 10000`.
   - Action: Scale RPC nodes and investigate potential spam attack.
