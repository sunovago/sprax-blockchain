# SPRX Ecosystem: Admin Panel Deployment Guide
**Document Version:** 1.0.0  

---

## 1. Local Development
```bash
# 1. Start backend with admin APIs
cd backend
uvicorn app.main:app --host 0.0.0.0 --port 8000 --reload

# 2. Start admin frontend
cd apps/admin-panel
npm install
npm run dev
# Admin dashboard opens at http://localhost:3001
```

---

## 2. Production Build & Deployment
```bash
cd apps/admin-panel
npm install
npm run test
npm run build
# Output is generated into apps/admin-panel/dist/
```
The static assets in `dist/` can be served behind Nginx / Cloudflare CDN with strict Content Security Policies (CSP) and HTTPS.
