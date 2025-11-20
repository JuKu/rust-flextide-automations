# Chroma Vector Database Integration

This module provides a Rust client for interacting with the Chroma vector database REST API. Chroma is an open-source vector database designed for AI applications, enabling efficient storage and retrieval of vector embeddings for semantic search, RAG (Retrieval-Augmented Generation), and similarity matching.

## Features

- **Collection Management**: Create, get, list, and delete collections
- **Document Operations**: Add, update, delete, and retrieve documents
- **Similarity Search**: Query collections for semantically similar documents
- **Metadata Filtering**: Filter documents by metadata key-value pairs
- **Flexible Deployment**: Support for local, client-server, and cloud deployments
- **Authentication**: Optional API key authentication for secure access

## Usage

### Basic Setup

```rust
use integrations::ChromaClient;
use integrations::chroma::*;

// Create a client for local Chroma server (default: http://localhost:8000)
let client = ChromaClient::new();

// Or with custom base URL
let client = ChromaClient::with_base_url("http://chroma.example.com:8000".to_string());

// Or with API key authentication (for Chroma Cloud or secured servers)
let client = ChromaClient::with_api_key(
    "https://api.trychroma.com".to_string(),
    "your-api-key".to_string()
);
```

### Collection Management

```rust
// Create a new collection
let collection = client.create_collection(CreateCollectionRequest {
    name: "my_documents".to_string(),
    metadata: Some({
        let mut meta = std::collections::HashMap::new();
        meta.insert("description".to_string(), json!("Document collection"));
        meta
    }),
    embedding_function: None, // Use default embedding function
}).await?;

// Get a collection
let collection = client.get_collection("my_documents").await?;

// List all collections
let collections = client.list_collections().await?;

// Delete a collection
client.delete_collection("my_documents").await?;
```

### Adding Documents

```rust
use std::collections::HashMap;

// Add documents with automatic embedding generation
client.add_documents("my_documents", AddDocumentsRequest {
    ids: vec!["doc1".to_string(), "doc2".to_string()],
    documents: Some(vec![
        "This is the first document".to_string(),
        "This is the second document".to_string(),
    ]),
    metadatas: Some(vec![
        {
            let mut meta = HashMap::new();
            meta.insert("source".to_string(), json!("web"));
            meta.insert("category".to_string(), json!("tech"));
            meta
        },
        {
            let mut meta = HashMap::new();
            meta.insert("source".to_string(), json!("book"));
            meta.insert("category".to_string(), json!("science"));
            meta
        },
    ]),
    embeddings: None, // Let Chroma generate embeddings automatically
}).await?;

// Or provide pre-computed embeddings
client.add_documents("my_documents", AddDocumentsRequest {
    ids: vec!["doc3".to_string()],
    documents: None,
    metadatas: None,
    embeddings: Some(vec![vec![0.1, 0.2, 0.3, /* ... */]]),
}).await?;
```

### Querying for Similar Documents

```rust
// Query by text (Chroma will generate embeddings automatically)
let results = client.query("my_documents", QueryRequest {
    query_texts: Some(vec!["What is machine learning?".to_string()]),
    query_embeddings: None,
    n_results: Some(5),
    where_clause: None,
    where_document: None,
    include: Some(vec!["documents".to_string(), "metadatas".to_string(), "distances".to_string()]),
}).await?;

// Process results
for (query_idx, query_results) in results.ids.iter().enumerate() {
    println!("Query {} found {} results:", query_idx, query_results.len());
    for (result_idx, doc_id) in query_results.iter().enumerate() {
        if let Some(doc) = results.documents[query_idx][result_idx].as_ref() {
            println!("  {}: {}", doc_id, doc);
        }
        if let Some(distance) = results.distances.get(query_idx)
            .and_then(|dists| dists.get(result_idx)) {
            println!("    Distance: {}", distance);
        }
    }
}

// Query with pre-computed embeddings
let query_embedding = vec![0.1, 0.2, 0.3, /* ... */];
let results = client.query("my_documents", QueryRequest {
    query_texts: None,
    query_embeddings: Some(vec![query_embedding]),
    n_results: Some(10),
    where_clause: None,
    where_document: None,
    include: Some(vec!["documents".to_string()]),
}).await?;
```

### Filtering with Metadata

```rust
use serde_json::json;

// Query with metadata filter
let results = client.query("my_documents", QueryRequest {
    query_texts: Some(vec!["AI research".to_string()]),
    query_embeddings: None,
    n_results: Some(5),
    where_clause: Some(MetadataFilter::Equals({
        let mut filter = HashMap::new();
        filter.insert("category".to_string(), json!("tech"));
        filter
    })),
    where_document: None,
    include: Some(vec!["documents".to_string(), "metadatas".to_string()]),
}).await?;
```

### Getting Documents

```rust
// Get documents by IDs
let docs = client.get_documents("my_documents", GetDocumentsRequest {
    ids: Some(vec!["doc1".to_string(), "doc2".to_string()]),
    where_clause: None,
    where_document: None,
    limit: None,
    offset: None,
    include: Some(vec!["documents".to_string(), "metadatas".to_string()]),
}).await?;

// Get all documents with metadata filter
let docs = client.get_documents("my_documents", GetDocumentsRequest {
    ids: None,
    where_clause: Some(MetadataFilter::Equals({
        let mut filter = HashMap::new();
        filter.insert("source".to_string(), json!("web"));
        filter
    })),
    where_document: None,
    limit: Some(100),
    offset: Some(0),
    include: Some(vec!["documents".to_string()]),
}).await?;
```

### Updating Documents

```rust
// Update document content and metadata
client.update_documents("my_documents", UpdateDocumentsRequest {
    ids: vec!["doc1".to_string()],
    documents: Some(vec!["Updated document content".to_string()]),
    metadatas: Some(vec![{
        let mut meta = HashMap::new();
        meta.insert("updated".to_string(), json!(true));
        meta
    }]),
    embeddings: None,
}).await?;
```

### Deleting Documents

```rust
// Delete by IDs
client.delete_documents("my_documents", DeleteDocumentsRequest {
    ids: Some(vec!["doc1".to_string(), "doc2".to_string()]),
    where_clause: None,
    where_document: None,
}).await?;

// Delete by metadata filter
client.delete_documents("my_documents", DeleteDocumentsRequest {
    ids: None,
    where_clause: Some(MetadataFilter::Equals({
        let mut filter = HashMap::new();
        filter.insert("category".to_string(), json!("archived"));
        filter
    })),
    where_document: None,
}).await?;
```

### Utility Operations

```rust
// Count documents in a collection
let count = client.count("my_documents").await?;
println!("Collection has {} documents", count);

// Peek at sample documents
let sample = client.peek("my_documents", Some(10)).await?;
println!("Sample documents: {:?}", sample.ids);
```

## Error Handling

The client uses the `ChromaError` enum for error handling:

```rust
use integrations::chroma::ChromaError;

match client.get_collection("nonexistent").await {
    Ok(collection) => println!("Found: {}", collection.name),
    Err(ChromaError::CollectionNotFound(name)) => {
        println!("Collection '{}' not found", name);
    }
    Err(ChromaError::InvalidApiKey) => {
        println!("Invalid API key");
    }
    Err(ChromaError::ApiError(msg)) => {
        println!("API error: {}", msg);
    }
    Err(e) => println!("Other error: {}", e),
}
```

## Deployment Modes

### Local Development
```rust
// Connect to local Chroma server (default: http://localhost:8000)
let client = ChromaClient::new();
```

### Remote Server
```rust
// Connect to remote Chroma server
let client = ChromaClient::with_base_url("http://chroma.example.com:8000".to_string());
```

### Chroma Cloud
```rust
// Connect to Chroma Cloud with API key
let client = ChromaClient::with_api_key(
    "https://api.trychroma.com".to_string(),
    "your-api-key".to_string()
);
```

## Best Practices

1. **Batch Operations**: When adding many documents, batch them in reasonable sizes (e.g., 100-1000 documents per request)
2. **Metadata Design**: Use consistent metadata schemas across documents for effective filtering
3. **Embedding Dimensions**: Ensure all embeddings in a collection have the same dimensions
4. **Error Handling**: Always handle errors appropriately, especially for network operations
5. **Connection Pooling**: The client uses a single HTTP client instance; reuse the client for multiple operations

## See Also

- [Chroma Research Documentation](../../../../docs/research/Chroma_API.md) - Comprehensive API research and documentation
- [Chroma Official Documentation](https://docs.trychroma.com/) - Official Chroma documentation
- [Chroma GitHub Repository](https://github.com/chroma-core/chroma) - Chroma source code and examples

