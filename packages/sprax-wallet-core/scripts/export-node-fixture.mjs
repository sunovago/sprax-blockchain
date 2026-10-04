import { createServer } from "vite";
const server = await createServer({ server: { middlewareMode: true }, appType: "custom" });
try { await server.ssrLoadModule("/scripts/export-node-fixture.ts"); }
finally { await server.close(); }
