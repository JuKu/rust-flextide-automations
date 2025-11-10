# Flextide Architecture

## Overview
Flextide is a modular workflow automation platform built on Rust, WASM, and Next.js. The system is split into independent services and crates to ensure high performance, maintainability, and scalability.

## Components
- **API Service (Axum)**: Authentication, workflow CRUD, marketplace, JS/TS build pipeline.
- **Worker Service**: Executes workflow nodes via JS sandbox, WASM, or native Rust.
- **Core Crate**: Workflow data models, ABI definitions, graph validation.
- **SDK Crate**: For building Rust → WASM nodes.
- **Node Registry**: Handles installed node metadata and marketplace integration.
- **Next.js Admin UI**: Dashboard, editor, marketplace UI.

## Execution Flow
1. Trigger fires (Webhook/Cron/etc.)
2. Worker resolves next nodes in graph.
3. Executes node:
   - JS/TS via QuickJS
   - WASM via Wasmtime
   - Native Rust built-in
4. Logs results and continues graph traversal.
