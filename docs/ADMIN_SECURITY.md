# SPRX Protocol: Admin Security & Auditing Standards
**Document Version:** 1.0.0  

---

## 1. Secrets Protection Policies
- **Zero Wallet Secrets in DB/UI**: User seed phrases, private keys, and wallet PINs are strictly non-custodial and never touch the backend database or admin interface.
- **Node Signing Key Protection**: Validator node signing keys are isolated in hardware security modules / encrypted keystores; no admin RPC exposes private keys.
- **Database & Redis Isolation**: Credentials are provided via environment variables only; raw command consoles are disabled.

---

## 2. Audit Logging
Every administrative mutation (`create_admin`, `update_feature_flag`, `update_app_version`, `broadcast_notification`, `update_ecosystem_project`) automatically records an immutable `AuditLog` entry in PostgreSQL containing:
- Actor (admin username)
- Action type
- Target resource
- Execution result (`success` / `failed`)
- Request ID
- Before/After metadata context
