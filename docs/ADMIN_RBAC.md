# SPRX Protocol: Admin RBAC & Permissions Matrix
**Document Version:** 1.0.0  

---

## 1. Role Hierarchy

| Role | Scope | Description |
| :--- | :--- | :--- |
| `super_admin` | `*` | Full unrestricted access to all ecosystem modules, admin creation, and feature flags. |
| `admin` | `blockchain.*`, `explorer.*`, `markets.*`, `discover.*`, `validators.*`, `staking.*`, `notifications.*`, `users.*`, `operations.*`, `feature_flags.*` | Full management of ecosystem content, feature flags, and announcements. |
| `operations` | `blockchain.read`, `explorer.*`, `indexer.retry`, `operations.*`, `notifications.send` | Node & indexer maintenance, service health monitoring, and broadcast notifications. |
| `viewer` | `*.read` | Read-only analytics, health dashboard, and audit inspection. |

---

## 2. Safety Invariants
- Frontend route hiding is **NOT** treated as authorization. The FastAPI backend unconditionally verifies JWT role and permissions on every protected endpoint.
- Feature flags controlling `perps_enabled` and `mainnet_enabled` cannot be enabled through simple API mutations; they strictly require explicit multi-sig production signing.
