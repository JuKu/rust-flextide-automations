# Chroma Vector Database API Research

## Overview

Chroma is an open-source vector database designed to facilitate the development of AI applications by providing efficient storage, retrieval, and management of vector embeddings. It enables developers to build applications that require semantic search, similarity matching, and retrieval-augmented generation (RAG) capabilities.

**Key Characteristics:**
- **Open Source**: Licensed under Apache License 2.0
- **Developer-Friendly**: Simple API that allows rapid prototyping and seamless production deployment
- **Flexible Deployment**: Supports in-memory, client-server, and managed cloud deployments
- **AI Integration**: Native support for embedding models from OpenAI, Cohere, Google, Hugging Face, and others

## Core Concepts

### Vector Embeddings
Vector embeddings are high-dimensional numerical representations of data (text, images, etc.) that capture semantic meaning. These embeddings enable similarity search, where semantically similar items are located near each other in the vector space.

### Collections
Collections are the primary organizational unit in Chroma. They group related documents and their embeddings together, allowing for efficient querying and management. Each collection can have its own embedding function and metadata schema.

### Documents
Documents are the data items stored in Chroma. They consist of:
- **Content**: The actual text or data
- **Embeddings**: Vector representations (can be auto-generated or provided)
- **Metadata**: Key-value pairs for filtering and organization
- **IDs**: Unique identifiers for each document

## Deployment Modes

### 1. In-Memory Mode
- **Use Case**: Rapid prototyping, development, and testing
- **Characteristics**: Data is stored in memory and lost on restart
- **Setup**: Simple client initialization without server
- **Limitations**: Not suitable for production or persistent storage

### 2. Client-Server Mode
- **Use Case**: Production deployments, scalable applications
- **Characteristics**: Persistent storage, multiple clients can connect
- **Setup**: Requires running a Chroma server (typically on port 8000)
- **API**: RESTful HTTP API for remote access
- **Authentication**: Optional API key support for secure access

### 3. Chroma Cloud (Managed Service)
- **Use Case**: Serverless, scalable production deployments
- **Characteristics**: Managed infrastructure, automatic scaling
- **Setup**: Requires Chroma Cloud account and API key
- **Features**: Full-text search, vector search, and metadata filtering

## API Architecture

### Python Client API (Primary)
Chroma's primary API is designed for Python, providing a simple, intuitive interface:

**Core Operations:**
1. **Client Initialization**: Create a client (in-memory or remote)
2. **Collection Management**: Create, get, list, delete collections
3. **Document Operations**: Add, update, delete documents
4. **Querying**: Perform similarity searches and filtering

**Example Usage:**
```python
import chromadb

# In-memory client
client = chromadb.Client()

# Or remote client
client = chromadb.HttpClient(host="localhost", port=8000)

# Create collection
collection = client.create_collection("my_collection")

# Add documents
collection.add(
    documents=["Document 1", "Document 2"],
    metadatas=[{"source": "web"}, {"source": "book"}],
    ids=["doc1", "doc2"]
)

# Query
results = collection.query(
    query_texts=["search query"],
    n_results=2
)
```

### REST API (HTTP Client-Server Mode)

When running Chroma in client-server mode, it exposes a RESTful HTTP API. The API follows REST conventions with JSON request/response formats.

**Base URL**: `http://localhost:8000` (default) or custom server address

**Authentication**: 
- Optional API key authentication via `X-Chroma-Token` header
- Format: `X-Chroma-Token: <api_key>`

**Content-Type**: `application/json` for all requests

#### Core REST Endpoints

##### Collections

**Create Collection**
- **Endpoint**: `POST /api/v1/collections`
- **Request Body**:
  ```json
  {
    "name": "collection_name",
    "metadata": {},
    "embedding_function": "optional_embedding_function_name"
  }
  ```
- **Response**: Collection object with name, id, metadata

**Get Collection**
- **Endpoint**: `GET /api/v1/collections/{collection_name}`
- **Response**: Collection details

**List Collections**
- **Endpoint**: `GET /api/v1/collections`
- **Response**: Array of collection objects

**Delete Collection**
- **Endpoint**: `DELETE /api/v1/collections/{collection_name}`
- **Response**: Success confirmation

##### Documents

**Add Documents**
- **Endpoint**: `POST /api/v1/collections/{collection_name}/add`
- **Request Body**:
  ```json
  {
    "ids": ["id1", "id2"],
    "documents": ["document text 1", "document text 2"],
    "metadatas": [{"key": "value"}, {"key": "value"}],
    "embeddings": [[0.1, 0.2, ...], [0.3, 0.4, ...]]  // Optional
  }
  ```
- **Response**: Success confirmation

**Update Documents**
- **Endpoint**: `POST /api/v1/collections/{collection_name}/update`
- **Request Body**: Similar to add, with existing IDs
- **Response**: Success confirmation

**Delete Documents**
- **Endpoint**: `POST /api/v1/collections/{collection_name}/delete`
- **Request Body**:
  ```json
  {
    "ids": ["id1", "id2"],
    "where": {},  // Optional metadata filter
    "where_document": {}  // Optional document content filter
  }
  ```
- **Response**: Success confirmation

**Query Collection**
- **Endpoint**: `POST /api/v1/collections/{collection_name}/query`
- **Request Body**:
  ```json
  {
    "query_texts": ["search query"],  // Optional if query_embeddings provided
    "query_embeddings": [[0.1, 0.2, ...]],  // Optional if query_texts provided
    "n_results": 10,
    "where": {},  // Optional metadata filter
    "where_document": {},  // Optional document content filter
    "include": ["documents", "metadatas", "distances", "embeddings"]
  }
  ```
- **Response**:
  ```json
  {
    "ids": [["id1", "id2"]],
    "documents": [["doc1", "doc2"]],
    "metadatas": [[{"key": "value"}, {"key": "value"}]],
    "distances": [[0.1, 0.2]],
    "embeddings": [[[0.1, 0.2], [0.3, 0.4]]]
  }
  ```

**Get Documents**
- **Endpoint**: `POST /api/v1/collections/{collection_name}/get`
- **Request Body**:
  ```json
  {
    "ids": ["id1", "id2"],  // Optional
    "where": {},  // Optional metadata filter
    "where_document": {},  // Optional document content filter
    "limit": 10,  // Optional
    "offset": 0,  // Optional
    "include": ["documents", "metadatas", "embeddings"]
  }
  ```
- **Response**: Documents matching the criteria

##### Count Documents
- **Endpoint**: `POST /api/v1/collections/{collection_name}/count`
- **Request Body**: Optional filters
- **Response**: `{"count": 42}`

##### Peek Collection
- **Endpoint**: `GET /api/v1/collections/{collection_name}/peek?limit=10`
- **Response**: Sample documents from the collection

## Embedding Functions

Chroma supports multiple embedding providers:

### Built-in Embedding Functions
- **OpenAI**: `chromadb.utils.embedding_functions.OpenAIEmbeddingFunction`
- **Cohere**: `chromadb.utils.embedding_functions.CohereEmbeddingFunction`
- **Hugging Face**: `chromadb.utils.embedding_functions.HuggingFaceEmbeddingFunction`
- **Sentence Transformers**: `chromadb.utils.embedding_functions.SentenceTransformerEmbeddingFunction`

### Custom Embeddings
- Users can provide pre-computed embeddings when adding documents
- Custom embedding functions can be implemented

## Search Capabilities

### Vector Similarity Search
- **Method**: Cosine similarity, Euclidean distance, or inner product
- **Algorithm**: HNSW (Hierarchical Navigable Small World) graphs for efficient approximate nearest neighbor search
- **Use Case**: Semantic similarity matching

### Metadata Filtering
- Filter documents by metadata key-value pairs
- Supports complex queries with operators (equals, not equals, greater than, etc.)
- Example: `{"source": "web", "category": {"$ne": "archived"}}`

### Full-Text Search
- Available in Chroma Cloud
- Enables keyword-based search alongside vector search
- Combines semantic and keyword matching

### Hybrid Search
- Combines vector similarity with metadata filtering
- Can combine with full-text search (Chroma Cloud)

## Error Handling

**Common HTTP Status Codes:**
- `200 OK`: Successful request
- `400 Bad Request`: Invalid request parameters
- `404 Not Found`: Collection or resource not found
- `409 Conflict`: Collection already exists
- `500 Internal Server Error`: Server error

**Error Response Format:**
```json
{
  "error": "Error message description"
}
```

## Performance Considerations

### Indexing
- Chroma automatically indexes embeddings when documents are added
- HNSW algorithm provides fast approximate nearest neighbor search
- Index updates are incremental

### Scalability
- Client-server mode supports multiple concurrent clients
- Chroma Cloud provides automatic scaling
- Horizontal scaling possible with multiple server instances

### Storage
- Embeddings are stored efficiently
- Metadata is indexed for fast filtering
- Supports persistent storage in client-server mode

## Use Cases

### 1. Retrieval-Augmented Generation (RAG)
- Store document embeddings
- Retrieve relevant context for LLM prompts
- Enhance AI responses with accurate information

### 2. Semantic Search
- Search documents by meaning rather than keywords
- Find similar content across large document collections
- Recommendation systems

### 3. Question Answering Systems
- Store knowledge base as embeddings
- Query for relevant information
- Generate answers from retrieved context

### 4. Content Recommendation
- Find similar items based on embeddings
- Personalize recommendations
- Content discovery

## Integration with AI Frameworks

### LangChain
Chroma integrates seamlessly with LangChain's vector store interface:
```python
from langchain.vectorstores import Chroma
from langchain.embeddings import OpenAIEmbeddings

vectorstore = Chroma.from_documents(
    documents,
    OpenAIEmbeddings(),
    collection_name="my_collection"
)
```

### LlamaIndex
Chroma can be used as a storage backend for LlamaIndex:
```python
from llama_index import VectorStoreIndex, StorageContext
from llama_index.vector_stores import ChromaVectorStore

vector_store = ChromaVectorStore(chroma_collection=collection)
storage_context = StorageContext.from_defaults(vector_store=vector_store)
index = VectorStoreIndex.from_documents(documents, storage_context=storage_context)
```

## Installation and Setup

### Python Client
```bash
pip install chromadb
```

### Docker Deployment (Client-Server Mode)
```bash
docker pull chromadb/chroma
docker run -p 8000:8000 chromadb/chroma
```

### Environment Variables
- `CHROMA_SERVER_HOST`: Server host (default: localhost)
- `CHROMA_SERVER_PORT`: Server port (default: 8000)
- `CHROMA_SERVER_AUTHN_PROVIDER`: Authentication provider
- `CHROMA_SERVER_AUTHN_CREDENTIALS`: Authentication credentials

## Security Considerations

### Authentication
- API key authentication supported in client-server mode
- Token-based authentication for Chroma Cloud
- HTTPS recommended for production deployments

### Data Privacy
- Local deployment ensures data stays on-premises
- Chroma Cloud provides enterprise-grade security
- Encryption at rest and in transit (Chroma Cloud)

## Limitations and Considerations

1. **Embedding Dimensions**: Must be consistent within a collection
2. **Metadata Size**: Large metadata can impact performance
3. **Collection Limits**: Practical limits depend on deployment mode
4. **Update Operations**: Updates require re-indexing affected vectors
5. **Concurrent Writes**: May require coordination in multi-client scenarios

## Future Research Areas

1. **Exact API Endpoint Specifications**: Verify exact REST API endpoints from official documentation
2. **Authentication Flow**: Detailed authentication mechanism for API keys
3. **Batch Operations**: Support for bulk operations and batch processing
4. **Replication**: High availability and data replication strategies
5. **Performance Benchmarks**: Query latency and throughput characteristics
6. **Advanced Filtering**: Complex query operators and filtering capabilities
7. **Versioning**: Document versioning and history tracking

## References

- **Official Repository**: https://github.com/chroma-core/chroma
- **Documentation**: https://docs.trychroma.com/
- **Chroma Cloud**: https://www.trychroma.com/
- **License**: Apache License 2.0

## Notes for Implementation

When implementing the Rust client for Chroma:

1. **Base URL Configuration**: Support custom base URLs for different deployment modes
2. **Error Handling**: Map HTTP status codes to appropriate error types
3. **Request/Response Types**: Define Rust structs matching the JSON schema
4. **Authentication**: Support optional API key authentication
5. **Async Operations**: Use async/await for all HTTP operations
6. **Collection Management**: Implement CRUD operations for collections
7. **Document Operations**: Support add, update, delete, get, query operations
8. **Embedding Support**: Allow both text-based queries (auto-embed) and pre-computed embeddings
9. **Metadata Filtering**: Implement metadata query language support
10. **Type Safety**: Leverage Rust's type system for request/response validation

