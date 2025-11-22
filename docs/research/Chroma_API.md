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

**API Versions**:
- **API v1**: Legacy API (deprecated in newer versions)
- **API v2**: Current API with tenant/database structure (recommended)

## API v2 Architecture

API v2 introduces a multi-tenant architecture with the following hierarchy:
- **Tenants**: Top-level organizational units
- **Databases**: Collections of data within a tenant
- **Collections**: Groups of documents within a database

### API v2 Base Structure

All v2 endpoints follow the pattern: `/api/v2/tenants/{tenant}/databases/{database}/...`

**Default Tenant**: Most deployments use a default tenant (often `default_tenant`)

**Default Database**: Most deployments use a default database (often `default_database`)

### API v2 Authentication

Chroma API v2 supports multiple authentication methods:

#### Authentication Methods

1. **Token Authentication (Bearer Token)**
   - Token is sent in the `Authorization` header
   - Format: `Authorization: Bearer <token>`
   - Default token prefix: `"Bearer "`
   - Default transport header: `"Authorization"`

2. **Token Authentication (X-Chroma-Token)**
   - Token is sent in a custom header
   - Format: `X-Chroma-Token: <token>`
   - Used when `token_transport_header` is set to `"X-Chroma-Token"` or another custom header

3. **Custom Headers**
   - Additional headers can be specified for authentication
   - Useful for custom authentication schemes or proxy authentication

#### Authentication Configuration

When configuring Chroma credentials, the following fields control authentication:

- **`auth_method`**: Authentication method (default: `"token"`)
- **`token_transport_header`**: Header name for token transport (default: `"Authorization"`)
- **`token_prefix`**: Prefix for the token (default: `"Bearer "`)
- **`auth_token`**: The actual authentication token
- **`additional_headers`**: Array of key-value pairs for additional headers

#### Secured Mode

- **`secured_mode`**: Boolean indicating if connection uses HTTPS/TLS (default: `true`)
- When `true`, the connection should use HTTPS
- When `false`, HTTP connections are allowed (not recommended for production)

**Get User Identity**
- **Endpoint**: `GET /api/v2/auth/identity`
- **Response**: Returns current user's identity, tenant, and databases
- **Use Case**: Determine available tenants and databases for the authenticated user
- **Authentication**: Requires valid authentication token in headers

#### Core REST Endpoints (API v1 - Legacy)

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

## API v2 Endpoints

### Tenant and Database Management

Chroma API v2 uses a hierarchical structure: **Tenant → Database → Collection**

#### Tenants

Tenants are top-level organizational units that provide isolation between different organizations or projects. Each tenant can contain multiple databases.

**List Tenants**
- **Endpoint**: `GET /api/v2/tenants`
- **Response**: List of available tenants
- **Authentication**: Required

**Get Tenant**
- **Endpoint**: `GET /api/v2/tenants/{tenant_name}`
- **Response**: Tenant details
- **Status Codes**:
  - `200 OK`: Tenant exists
  - `404 Not Found`: Tenant does not exist
- **Use Case**: Check if a tenant exists before creating resources
- **Authentication**: Required

**Create Tenant**
- **Endpoint**: `POST /api/v2/tenants`
- **Request Body**: `{"name": "tenant_name"}`
- **Response**: Created tenant details
- **Authentication**: Required

**Default Tenant**: Most deployments use a default tenant (often `"default_tenant"`)

#### Databases

Databases are collections of data within a tenant. Each database can contain multiple collections. Databases provide logical separation of data within a tenant.

**List Databases**
- **Endpoint**: `GET /api/v2/tenants/{tenant}/databases`
- **Response**: List of databases in the tenant
- **Authentication**: Required

**Create Database**
- **Endpoint**: `POST /api/v2/tenants/{tenant}/databases`
- **Request Body**: `{"name": "database_name"}`
- **Response**: Created database details
- **Authentication**: Required

**Get Database**
- **Endpoint**: `GET /api/v2/tenants/{tenant}/databases/{database}`
- **Response**: Database details
- **Authentication**: Required

**Default Database**: Most deployments use a default database (often `"default_database"`)

#### Tenant/Database Hierarchy

The hierarchy ensures:
- **Isolation**: Data in different tenants is completely isolated
- **Organization**: Databases within a tenant can organize related collections
- **Scalability**: Supports multi-tenant deployments with proper access control

### Collections (API v2)

**List Collections**
- **Endpoint**: `GET /api/v2/tenants/{tenant}/databases/{database}/collections`
- **Response**: Array of collection objects

**Create Collection**
- **Endpoint**: `POST /api/v2/tenants/{tenant}/databases/{database}/collections`
- **Request Body**:
  ```json
  {
    "name": "collection_name",
    "metadata": {},
    "dimension": 1536,
    "get_or_create": false
  }
  ```
- **Response**: Collection object with id, name, metadata

**Get Collection**
- **Endpoint**: `GET /api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}`
- **Alternative**: `GET /api/v2/collections/{crn}` (using Chroma Resource Name)
- **Response**: Collection details

**Delete Collection**
- **Endpoint**: `DELETE /api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}`
- **Response**: Success confirmation

**Fork Collection**
- **Endpoint**: `POST /api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}/fork`
- **Request Body**: Target database/collection information
- **Response**: Forked collection details

### Document Operations (API v2)

**Add Documents**
- **Endpoint**: `POST /api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}/add`
- **Request Body**:
  ```json
  {
    "ids": ["id1", "id2"],
    "documents": ["document text 1", "document text 2"],
    "metadatas": [{"key": "value"}, {"key": "value"}],
    "embeddings": [[0.1, 0.2, ...], [0.3, 0.4, ...]]
  }
  ```
- **Response**: Success confirmation

**Upsert Documents** (Add or Update)
- **Endpoint**: `POST /api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}/upsert`
- **Request Body**: Same as add
- **Response**: Success confirmation
- **Note**: If document with ID exists, it will be updated; otherwise, it will be added

**Update Documents**
- **Endpoint**: `POST /api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}/update`
- **Request Body**: Same as add, with existing IDs
- **Response**: Success confirmation

**Delete Documents**
- **Endpoint**: `POST /api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}/delete`
- **Request Body**:
  ```json
  {
    "ids": ["id1", "id2"],
    "where": {},
    "where_document": {}
  }
  ```
- **Response**: Success confirmation

**Get Documents**
- **Endpoint**: `POST /api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}/get`
- **Request Body**:
  ```json
  {
    "ids": ["id1", "id2"],
    "where": {},
    "where_document": {},
    "limit": 10,
    "offset": 0,
    "include": ["documents", "metadatas", "embeddings"]
  }
  ```
- **Response**: Documents matching the criteria

**Query Collection**
- **Endpoint**: `POST /api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}/query`
- **Request Body**:
  ```json
  {
    "query_texts": ["search query"],
    "query_embeddings": [[0.1, 0.2, ...]],
    "n_results": 10,
    "where": {},
    "where_document": {},
    "include": ["documents", "metadatas", "distances", "embeddings"]
  }
  ```
- **Response**:
  ```json
  {
    "ids": [["id1", "id2"]],
    "documents": [["doc1", "doc2"]],
    "metadatas": [[{"key": "value"}]],
    "distances": [[0.1, 0.2]],
    "embeddings": [[[0.1, 0.2], [0.3, 0.4]]]
  }
  ```

**Search Collection**
- **Endpoint**: `POST /api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}/search`
- **Request Body**: Similar to query
- **Response**: Similar to query
- **Note**: Search may support additional features like full-text search in Chroma Cloud

**Count Documents**
- **Endpoint**: `POST /api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}/count`
- **Request Body**: Optional filters
- **Response**: `{"count": 42}`

### Utility Endpoints (API v2)

**Health Check**
- **Endpoint**: `GET /api/v2/healthcheck`
- **Response**: Health status

**Heartbeat**
- **Endpoint**: `GET /api/v2/heartbeat`
- **Response**: Server heartbeat

**Pre-flight Checks**
- **Endpoint**: `GET /api/v2/pre-flight-checks`
- **Response**: System readiness checks

**Reset** (Development only)
- **Endpoint**: `POST /api/v2/reset`
- **Response**: Success confirmation
- **Warning**: Resets all data - use only in development

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

## API v2 Reference

**Documentation Source**: Based on Chroma API v2 OpenAPI specification (http://152.53.103.98:8001/docs/)

**Key Endpoints Summary**:
- Authentication: `/api/v2/auth/identity`
- Tenants: `/api/v2/tenants`
- Databases: `/api/v2/tenants/{tenant}/databases`
- Collections: `/api/v2/tenants/{tenant}/databases/{database}/collections`
- Documents: `/api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}/add|upsert|update|delete|get|query|search|count`

**Important Notes**:
- API v2 is not backward compatible with v1
- All operations require tenant and database context
- Collections are identified by UUID, not name
- Use `/api/v2/auth/identity` to discover available tenants and databases

## Future Research Areas

1. **Exact Request/Response Schemas**: Detailed JSON schema for all v2 endpoints
2. **CRN Format Details**: Complete specification of Chroma Resource Name format
3. **Attached Functions**: Understanding of collection function attachment mechanism
4. **Batch Operations**: Support for bulk operations and batch processing in v2
5. **Replication**: High availability and data replication strategies
6. **Performance Benchmarks**: Query latency and throughput characteristics
7. **Advanced Filtering**: Complex query operators and filtering capabilities in v2
8. **Versioning**: Document versioning and history tracking

## References

- **Official Repository**: https://github.com/chroma-core/chroma
- **Documentation**: https://docs.trychroma.com/
- **Chroma Cloud**: https://www.trychroma.com/
- **License**: Apache License 2.0

## API v2 Key Changes from v1

### Breaking Changes

1. **Tenant/Database Structure**: All endpoints now require tenant and database parameters
   - v1: `/api/v1/collections/{name}`
   - v2: `/api/v2/tenants/{tenant}/databases/{database}/collections/{collection_id}`

2. **Collection Identification**: Collections are identified by ID (UUID) rather than name
   - Collections can also be accessed via CRN (Chroma Resource Name)

3. **New Upsert Operation**: API v2 introduces `upsert` which combines add/update logic

4. **Separate Search Endpoint**: Query and search are now separate endpoints with potentially different capabilities

5. **Authentication Endpoint**: New `/api/v2/auth/identity` endpoint to discover available tenants/databases

### Migration Path

For applications migrating from v1 to v2:

1. **Determine Tenant/Database**: Use `/api/v2/auth/identity` to discover available tenants and databases
2. **Update Endpoint URLs**: Replace all `/api/v1/` with `/api/v2/tenants/{tenant}/databases/{database}/`
3. **Update Collection References**: Use collection IDs instead of names
4. **Update Request Formats**: Some request/response formats may have changed
5. **Test Thoroughly**: API v2 is not backward compatible with v1

### Chroma Resource Names (CRN)

API v2 introduces Chroma Resource Names (CRN) as an alternative way to reference collections:
- Format: `crn://{tenant}/{database}/{collection_id}`
- Allows direct access: `GET /api/v2/collections/{crn}`
- Useful for cross-tenant/database operations

## Notes for Implementation

When implementing the Rust client for Chroma API v2:

1. **API Version**: Use API v2 (`/api/v2/`) instead of v1
2. **Tenant/Database Management**: Implement tenant and database discovery and management
3. **Base URL Configuration**: Support custom base URLs for different deployment modes
4. **Error Handling**: Map HTTP status codes to appropriate error types
5. **Request/Response Types**: Define Rust structs matching the JSON schema
6. **Authentication**: Support optional API key authentication via `X-Chroma-Token` header
7. **Async Operations**: Use async/await for all HTTP operations
8. **Collection Management**: Implement CRUD operations for collections using tenant/database structure
9. **Document Operations**: Support add, upsert, update, delete, get, query, search operations
10. **Embedding Support**: Allow both text-based queries (auto-embed) and pre-computed embeddings
11. **Metadata Filtering**: Implement metadata query language support
12. **Type Safety**: Leverage Rust's type system for request/response validation
13. **CRN Support**: Consider supporting Chroma Resource Names for collection access
14. **Default Values**: Handle default tenant/database values for simpler usage

