# Flextide – A Modern, Modular, High-Performance Workflow Automation Platform

Built with **Rust**, **WASM**, **JS/TS sandboxing**, and **Next.js**.

Flextide is a next-generation alternative to tools like n8n or Zapier, designed with a focus on **performance**, **security**, **extensibility**, and **developer experience**.  
Instead of a monolithic, JavaScript-heavy architecture, Flextide uses a **Rust-based backend**, a **sandboxed multi-runtime execution engine**, and a **beautiful visual workflow editor** built with **React Flow**.

Flextide aims to be the most reliable and extensible open-platform workflow engine available today.

---

## ✨ Key Features

### ✅ High-Performance Rust Backend  
- API built with **Axum**  
- Execution Worker built on **Tokio**  
- Fully asynchronous and horizontally scalable  
- Strong type safety and compile-time guarantees  

### ✅ Multi-Runtime Node Execution  
Run user-defined or marketplace nodes securely in isolated environments:

**Supported Node Types:**  
- **JavaScript / TypeScript** (QuickJS sandbox)  
- **Rust → WASM** (WASI sandbox via Wasmtime)  
- **Built-in Rust Nodes** (trusted, native performance)

### ✅ Visual Workflow Builder (Next.js + React Flow)  
- Drag-and-drop node editor  
- Left: data inputs  
- Right: data outputs  
- Top: exec-in pin  
- Bottom: exec-out pin + optional config  
- Typed connections and pin validation  
- Blueprint-like UX inspired by Unreal Engine 5

### ✅ Node Marketplace  
- Upload node packs  
- Sell or share custom nodes  
- Install packs for your organization  
- Versioning, metadata, documentation & icons  
- Secure sandbox validation on upload

### ✅ Fully Modular Architecture  
Separate Rust crates for:
- Core workflow engine  
- API  
- Worker  
- SDK for building WASM nodes  
- Node Registry  
- Plus a standalone Next.js Admin Panel

### ✅ Enterprise-Ready  
- Self-hosted or cloud deployment  
- MySQL & PostgreSQL support (via sqlx)  
- S3/Minio storage (optional, for later)  
- Strong security isolation  
- High reliability  
- Horizontal scaling through multiple workers

---

## 🏗️ Project Structure

/crates
/core # Shared workflow engine logic, ABI, validation
/api # REST API, auth, marketplace, TS compiler
/worker # Execution engine (JS, WASM, Rust nodes)
/sdk # Rust SDK for writing WASM nodes
/node_registry # Metadata + dynamic node loading

/bin
  api.rs # Starts API server
  worker.rs # Starts execution worker

/frontend # Next.js 16 app containing editor + dashboard
/migrations # SQLx migrations for MySQL & PostgreSQL

## 🚀 Getting Started

### 1. Install Rust and toolchains

```shell
rustup install stable
rustup target add wasm32-wasip1
```

### 2. Install SQLx CLI (for migrations)

For MySQL:
```shell
cargo install sqlx-cli --no-default-features --features mysql,native-tls
```

For PostgreSQL:
```shell
cargo install sqlx-cli --no-default-features --features postgres,native-tls
```

### 3. Install frontend dependencies

```shell
cd frontend
pnpm install
```

### 4. Run migrations

```shell
sqlx migrate run
```

### 5. Start services

API Backend:
```shell
cargo run --bin api
```

Worker:
```shell
cargo run --bin worker
```

Frontend:
```shell
pnpm run dev
```

## 🧩 Writing Custom Nodes

Flextide supports three types of nodes:

✅ JavaScript / TypeScript Nodes
  - User writes TS/JS
  - API compiles using esbuild
  - Worker executes inside QuickJS sandbox
  - Full JSON ABI support

✅ Rust → WASM Nodes
  - Built using wasm32-wasip1 target
  - Executed inside Wasmtime
  - Perfect for high-performance or secure logic

✅ Built-in Nodes
  - Written directly in Rust
  - No sandboxing
  - Fastest execution

Full node ABI documentation is available in /docs/node-abi.md.

## 🛒 Node Marketplace (WIP)

Flextide includes a full marketplace system where users can:
  - Upload node packs (*.zip)
  - Include multiple nodes per pack
  - Provide schema, code, icons, and documentation
  - Set pricing (one-time or subscription)
  - Publish, install, or update packs
  - Integrate with Stripe for payments

## 🔒 Security

  - JS nodes run in QuickJS with strict isolation
  - WASM nodes run in a WASI sandbox
  - No filesystem or network access unless explicitly granted
  - All user code is validated at upload
  - Node Packs checked for integrity + signature
  - Rate-limited API endpoints
  - TLS enforced in production

## 🗺️ Roadmap (High-Level)

### ✅ MVP Phase

  - Workflow builder
  - Node execution engine (JS + WASM)
  - Marketplace basics
  - Dashboard + auth
  - Node registry + pack install
  - Built-in nodes: HTTP, JSON tools, logic, etc.

### 🔜 Beta

  - AI Worker integration
  - Team roles & permissions
  - Subflow support
  - Execution debugger
  - Analytics dashboard

### 🔮 Future

  - Cloud hosting platform
  - Distributed graph execution
  - Auto-scaling worker clusters
  - Event-driven triggers (Kafka, NATS, SQS)
  - LDAP integration for authentication

## 🤝 Contributing

Contributions will be welcome once the core architecture stabilizes.
A detailed CONTRIBUTING.md will follow.

## 📜 License

To be defined (likely MIT or Apache-2.0).

## 💬 Feedback

If you have ideas or want to join the early builder group, open an issue or discussion on GitHub.