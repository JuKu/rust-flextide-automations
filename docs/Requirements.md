# Flextide – Requirements Specification

## 1. High-Level Vision
Flextide is a modular, secure, and high-performance workflow automation platform inspired by platforms like n8n or Zapier, but redesigned from the ground up with modern technology. Built using a Rust backend, a sandboxed multi-runtime execution engine, and a Next.js admin UI, Flextide aims to be the most reliable, extensible, and developer-friendly automation system available.

## 2. Core Functional Requirements
- Directed graph workflow model (nodes + edges)
- Multiple triggers per workflow
- Deterministic execution and history logging
- Typed pins (exec, string, number, boolean, json, any)
- ABI: { "input": {}, "config": {} } → { "output": {} }

## 3. Architecture
/crates: core, api, worker, sdk, node_registry
/bin: api.rs, worker.rs
/frontend: Next.js 16 admin UI

## 4. Node Marketplace
- Upload, validate, publish packs
- Monetization via Stripe
- Install packs per organization

## 5. Admin UI
- Auth required
- Header with Flextide branding
- Dashboard split: AI Employees (left), Workflows (right)
- Workflow editor via React Flow
- Typed connections
- Nodes with pins on left/right/top/bottom

## 6. Database
- Primary: MySQL & PostgreSQL support (via sqlx)
- Storage: S3/Minio support (optional, for later)
- Tables: Workflows, nodes, edges, runs, tasks, queue, packs, versions, installations, purchases, users.

## 7. Worker
- Poll queue
- Execute via JS sandbox, WASM sandbox, or Rust built-in types
- Logging + retries

## 8. API
Auth, workflows, nodes, marketplace, packs, runs, tasks.

## 9. JS/TS Build Pipeline
Compile TS/JS via esbuild or swc.

## 10. WASM Nodes
Rust → wasm32-wasip1 with JSON I/O.

## 11. Security
Sandboxed runtimes, pack validation, TLS, rate limiting.

## 12. Performance
Horizontal workers, low memory, fast execution.

## 13. Deployment
Self-hosted or cloud mode.

## 14. Monetization
SaaS + marketplace revenue split.

## 15. Developer Experience
SDK, templates, CLI.

## 16. Testing
Rust unit + integration tests, frontend E2E.

## 17. Extensibility
New node types, triggers, executors, UI modules.

## 18. Non-Functional
Stability, clarity, modularity, maintainability.
