# SPRX Protocol: Admin API Contract & Endpoints
**Document Version:** 1.0.0  
**Namespace:** `/api/v1/admin/*`

---

## 1. Endpoints Specification

| Method | Endpoint | Description | Required Role |
| :--- | :--- | :--- | :--- |
| `POST` | `/api/v1/admin/login` | Authenticate admin staff credentials | Public (Admin UI) |
| `GET` | `/api/v1/admin/me` | Current admin identity & permissions | `viewer`+ |
| `GET` | `/api/v1/admin/dashboard` | Ecosystem KPI & chart aggregates | `viewer`+ |
| `GET` | `/api/v1/admin/system/health` | Deep health check of all services | `viewer`+ |
| `GET` | `/api/v1/admin/app/users` | List registered mobile/web app users | `admin`+ |
| `GET` | `/api/v1/admin/app/versions` | Query current version configurations | `admin`+ |
| `PATCH`| `/api/v1/admin/app/versions` | Mutate version & maintenance flags | `admin`+ |
| `GET` | `/api/v1/admin/blockchain/stats`| Deep node metrics and block gas | `viewer`+ |
| `GET` | `/api/v1/admin/indexer/status` | Real-time indexer sync lag and error state | `viewer`+ |
| `POST` | `/api/v1/admin/indexer/retry` | Trigger safe indexer task retry | `operations`+ |
| `GET` | `/api/v1/admin/markets` | Display assets and trading pairs | `viewer`+ |
| `GET` | `/api/v1/admin/discover` | Discover projects list & featured flags | `viewer`+ |
| `PATCH`| `/api/v1/admin/discover/{id}` | Toggle project verification/featured | `admin`+ |
| `GET` | `/api/v1/admin/validators` | Active & inactive validator metrics | `viewer`+ |
| `GET` | `/api/v1/admin/perps/status` | Perpetual orderbook and risk monitor | `viewer`+ |
| `GET` | `/api/v1/admin/notifications` | Broadcast announcements history | `viewer`+ |
| `POST` | `/api/v1/admin/notifications/broadcast` | Broadcast announcement to wallets | `operations`+ |
| `GET` | `/api/v1/admin/feature-flags` | Runtime ecosystem feature flags | `viewer`+ |
| `PATCH`| `/api/v1/admin/feature-flags/{name}` | Mutate runtime feature flag | `admin`+ / `super_admin` |
| `GET` | `/api/v1/admin/admins` | List registered staff admin accounts | `admin`+ |
| `POST` | `/api/v1/admin/admins` | Create new admin staff identity | `super_admin` |
| `GET` | `/api/v1/admin/audit-logs` | Immutable audit trail query | `admin`+ |
