//! Chroma Vector Database Integration
//! 
//! Provides a client for interacting with the Chroma vector database REST API,
//! including collection management, document operations, and similarity search.

mod client;
mod error;
mod types;

pub use client::ChromaClient;
pub use error::ChromaError;
pub use types::*;

