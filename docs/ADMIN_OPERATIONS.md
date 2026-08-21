# SPRX Ecosystem: Admin Operations Manual
**Document Version:** 1.0.0  

---

## 1. Routine Operational Workflows

### 1.1 Indexer Sync Monitoring & Catchup
1. Open **Indexer Monitor** tab (`/admin/indexer`).
2. Verify `Chain Height` matches `Indexed Height` (Lag should be 0 or < 5 blocks).
3. If an indexer lag develops or `last_error` is reported, verify PostgreSQL connectivity and click **Trigger Sync Retry**.

### 1.2 App Version Upgrades & Maintenance
1. Open **App Versions** tab (`/admin/app/versions`).
2. Update `Latest Released Version` and provide release APK URL.
3. If a protocol breaking change occurs, update `Minimum Supported Version` and enable `Force App Update`.
4. In case of scheduled network maintenance, enable `App Maintenance Mode` with clear downtime timing.

### 1.3 Ecosystem Project Curation
1. Open **Discover & dApps** tab (`/admin/discover`).
2. Review community submitted dApps and toggle `Verified` badge after smart contract security review.
3. Toggle `Featured` to spotlight projects on the mobile wallet Discover carousel.
