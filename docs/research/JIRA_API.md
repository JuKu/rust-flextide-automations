# JIRA API Research

## Overview

JIRA is a project management and issue tracking tool developed by Atlassian. The JIRA REST API enables developers to interact programmatically with JIRA, allowing operations such as creating, updating, and querying issues, managing projects, users, workflows, and more. The API follows RESTful principles and supports both JIRA Cloud and JIRA Server/Data Center instances.

## API Versions

JIRA provides multiple API versions:

- **REST API v2**: Legacy version, still widely used
- **REST API v3**: Current recommended version with improved features
- **Platform REST API**: For Atlassian Cloud instances

**Base URLs:**
- **JIRA Cloud:** `https://{your-domain}.atlassian.net/rest/api/{version}/`
- **JIRA Server/Data Center:** `https://{your-domain}/rest/api/{version}/`
- **Atlassian Cloud Platform:** `https://api.atlassian.com/ex/jira/{cloudId}/rest/api/{version}/`

## Authentication

JIRA supports multiple authentication methods depending on the instance type and use case.

### 1. Basic Authentication (API Token)

Basic authentication uses an email address and API token. This is the simplest method for scripts and automated tools.

#### Generating an API Token

1. Navigate to [Atlassian Account Settings](https://id.atlassian.com/manage-profile/security/api-tokens)
2. Under "Security," select "Create and manage API tokens"
3. Click "Create API token"
4. Provide a label and copy the generated token

**Important:** API tokens are only shown once. Store them securely.

#### Using Basic Authentication

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X GET \
   -H "Content-Type: application/json" \
   https://your-domain.atlassian.net/rest/api/2/issue/createmeta
```

**Python Example:**
```python
import requests
from requests.auth import HTTPBasicAuth

url = "https://your-domain.atlassian.net/rest/api/2/issue/createmeta"
auth = HTTPBasicAuth("your-email@example.com", "your-api-token")
headers = {"Content-Type": "application/json"}

response = requests.get(url, auth=auth, headers=headers)
print(response.json())
```

**JavaScript/Node.js Example:**
```javascript
const axios = require('axios');
const base64 = require('base-64');

const email = 'your-email@example.com';
const apiToken = 'your-api-token';
const credentials = base64.encode(`${email}:${apiToken}`);

axios.get('https://your-domain.atlassian.net/rest/api/2/issue/createmeta', {
  headers: {
    'Authorization': `Basic ${credentials}`,
    'Content-Type': 'application/json'
  }
})
.then(response => console.log(response.data))
.catch(error => console.error(error));
```

**Rust Example:**
```rust
use reqwest;
use base64;

let email = "your-email@example.com";
let api_token = "your-api-token";
let credentials = base64::encode(format!("{}:{}", email, api_token));

let client = reqwest::Client::new();
let response = client
    .get("https://your-domain.atlassian.net/rest/api/2/issue/createmeta")
    .header("Authorization", format!("Basic {}", credentials))
    .header("Content-Type", "application/json")
    .send()
    .await?;
```

### 2. OAuth 2.0 (3LO - Three-Legged OAuth)

OAuth 2.0 is recommended for applications that require user authorization. It's the preferred method for cloud applications.

#### Setting Up OAuth 2.0

1. **Register Application in Atlassian Developer Console:**
   - Go to [Atlassian Developer Console](https://developer.atlassian.com/console)
   - Create a new OAuth 2.0 integration
   - Configure scopes (e.g., `read:jira-work`, `write:jira-work`)
   - Set redirect URI
   - Obtain Client ID and Client Secret

2. **Authorization Flow:**

**Step 1: Direct User to Authorization URL**
```bash
https://auth.atlassian.com/authorize?audience=api.atlassian.com&client_id=YOUR_CLIENT_ID&scope=read:jira-work%20write:jira-work&redirect_uri=YOUR_REDIRECT_URI&state=YOUR_STATE&response_type=code&prompt=consent
```

**Step 2: Exchange Authorization Code for Access Token**
```bash
curl --request POST \
  --url https://auth.atlassian.com/oauth/token \
  --header 'Content-Type: application/json' \
  --data '{
    "grant_type": "authorization_code",
    "client_id": "YOUR_CLIENT_ID",
    "client_secret": "YOUR_CLIENT_SECRET",
    "code": "AUTHORIZATION_CODE",
    "redirect_uri": "YOUR_REDIRECT_URI"
  }'
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "refresh_token_here",
  "scope": "read:jira-work write:jira-work"
}
```

**Step 3: Use Access Token**
```bash
curl --request GET \
  --url 'https://api.atlassian.com/ex/jira/{cloudId}/rest/api/2/issue/{issueKey}' \
  --header 'Authorization: Bearer {access_token}' \
  --header 'Accept: application/json'
```

**Python OAuth 2.0 Example:**
```python
import requests

# Step 1: Get authorization URL (redirect user to this)
auth_url = (
    "https://auth.atlassian.com/authorize?"
    "audience=api.atlassian.com&"
    "client_id=YOUR_CLIENT_ID&"
    "scope=read:jira-work%20write:jira-work&"
    "redirect_uri=YOUR_REDIRECT_URI&"
    "state=YOUR_STATE&"
    "response_type=code&"
    "prompt=consent"
)

# Step 2: Exchange code for token
def get_access_token(authorization_code):
    token_url = "https://auth.atlassian.com/oauth/token"
    data = {
        "grant_type": "authorization_code",
        "client_id": "YOUR_CLIENT_ID",
        "client_secret": "YOUR_CLIENT_SECRET",
        "code": authorization_code,
        "redirect_uri": "YOUR_REDIRECT_URI"
    }
    response = requests.post(token_url, json=data)
    return response.json()

# Step 3: Use access token
def get_issue(access_token, cloud_id, issue_key):
    url = f"https://api.atlassian.com/ex/jira/{cloud_id}/rest/api/2/issue/{issue_key}"
    headers = {
        "Authorization": f"Bearer {access_token}",
        "Accept": "application/json"
    }
    response = requests.get(url, headers=headers)
    return response.json()
```

### 3. OAuth 1.0a

OAuth 1.0a is available for JIRA Server/Data Center instances. It requires setting up an Application Link.

#### Setting Up OAuth 1.0a

1. **Create Application Link in JIRA:**
   - Go to JIRA Administration > Applications > Application Links
   - Create a new application link
   - Configure for incoming authentication
   - Generate consumer key and public/private key pair

2. **OAuth Flow:**
   - Request token
   - Authorize request token
   - Exchange for access token
   - Sign requests with access token

**Note:** OAuth 1.0a is more complex and typically used for server instances. OAuth 2.0 is preferred for cloud.

### 4. Personal Access Tokens (PAT)

Personal Access Tokens are available for JIRA Data Center and provide a secure alternative to passwords.

#### Generating a PAT

1. Navigate to JIRA profile settings
2. Select "Personal Access Tokens"
3. Generate a new token
4. Copy and store securely

#### Using PAT

```bash
curl --request GET \
  --url 'https://your-domain.atlassian.net/rest/api/2/issue/createmeta' \
  --header 'Authorization: Bearer your-personal-access-token' \
  --header 'Accept: application/json'
```

### 5. Cookie-Based Authentication (Deprecated)

Cookie-based authentication is deprecated in JIRA Cloud. Use API tokens or OAuth instead.

## API Endpoints

### Issue Operations

#### Create Issue

**Endpoint:** `POST /rest/api/2/issue`

**Request Body:**
```json
{
  "fields": {
    "project": {
      "key": "PROJ"
    },
    "summary": "Issue summary",
    "description": "Detailed description of the issue",
    "issuetype": {
      "name": "Bug"
    },
    "priority": {
      "name": "High"
    },
    "assignee": {
      "accountId": "account-id-here"
    },
    "labels": ["label1", "label2"],
    "components": [
      {
        "name": "Component1"
      }
    ]
  }
}
```

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X POST \
   -H "Content-Type: application/json" \
   --data '{
     "fields": {
       "project": {
         "key": "PROJ"
       },
       "summary": "Issue created via REST API",
       "description": "Description of the issue",
       "issuetype": {
         "name": "Bug"
       }
     }
   }' \
   https://your-domain.atlassian.net/rest/api/2/issue
```

**Response:**
```json
{
  "id": "10000",
  "key": "PROJ-1",
  "self": "https://your-domain.atlassian.net/rest/api/2/issue/10000"
}
```

**Python Example:**
```python
import requests
from requests.auth import HTTPBasicAuth
import json

url = "https://your-domain.atlassian.net/rest/api/2/issue"
auth = HTTPBasicAuth("your-email@example.com", "your-api-token")
headers = {
    "Accept": "application/json",
    "Content-Type": "application/json"
}

payload = json.dumps({
    "fields": {
        "project": {
            "key": "PROJ"
        },
        "summary": "Issue created via REST API",
        "description": "Description of the issue",
        "issuetype": {
            "name": "Bug"
        }
    }
})

response = requests.post(url, data=payload, headers=headers, auth=auth)
print(response.status_code)
print(response.json())
```

#### Get Issue

**Endpoint:** `GET /rest/api/2/issue/{issueIdOrKey}`

**Query Parameters:**
- `fields`: Comma-separated list of fields to return
- `expand`: Additional information to include (e.g., `renderedFields`, `names`, `schema`, `transitions`, `operations`, `editmeta`, `changelog`)

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X GET \
   -H "Content-Type: application/json" \
   https://your-domain.atlassian.net/rest/api/2/issue/PROJ-1?fields=summary,description,status
```

**Python Example:**
```python
import requests
from requests.auth import HTTPBasicAuth

url = "https://your-domain.atlassian.net/rest/api/2/issue/PROJ-1"
auth = HTTPBasicAuth("your-email@example.com", "your-api-token")
headers = {"Accept": "application/json"}

response = requests.get(url, headers=headers, auth=auth)
issue = response.json()

print(f"Issue Key: {issue['key']}")
print(f"Summary: {issue['fields']['summary']}")
print(f"Status: {issue['fields']['status']['name']}")
```

**Response:**
```json
{
  "expand": "renderedFields,names,schema,operations,editmeta,changelog",
  "id": "10000",
  "self": "https://your-domain.atlassian.net/rest/api/2/issue/10000",
  "key": "PROJ-1",
  "fields": {
    "summary": "Issue created via REST API",
    "status": {
      "self": "https://your-domain.atlassian.net/rest/api/2/status/1",
      "description": "The issue is open and ready for the assignee to start work on it.",
      "iconUrl": "https://your-domain.atlassian.net/images/icons/statuses/open.png",
      "name": "Open",
      "id": "1"
    },
    "description": "Description of the issue"
  }
}
```

#### Update Issue

**Endpoint:** `PUT /rest/api/2/issue/{issueIdOrKey}`

**Request Body:**
```json
{
  "fields": {
    "summary": "Updated summary",
    "description": "Updated description",
    "assignee": {
      "accountId": "account-id-here"
    }
  }
}
```

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X PUT \
   -H "Content-Type: application/json" \
   --data '{
     "fields": {
       "summary": "Updated summary",
       "description": "Updated description"
     }
   }' \
   https://your-domain.atlassian.net/rest/api/2/issue/PROJ-1
```

**Python Example:**
```python
import requests
from requests.auth import HTTPBasicAuth
import json

url = "https://your-domain.atlassian.net/rest/api/2/issue/PROJ-1"
auth = HTTPBasicAuth("your-email@example.com", "your-api-token")
headers = {
    "Accept": "application/json",
    "Content-Type": "application/json"
}

payload = json.dumps({
    "fields": {
        "summary": "Updated summary",
        "description": "Updated description"
    }
})

response = requests.put(url, data=payload, headers=headers, auth=auth)
print(response.status_code)
```

#### Delete Issue

**Endpoint:** `DELETE /rest/api/2/issue/{issueIdOrKey}`

**Query Parameters:**
- `deleteSubtasks`: Delete subtasks (true/false, default: false)

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X DELETE \
   -H "Content-Type: application/json" \
   https://your-domain.atlassian.net/rest/api/2/issue/PROJ-1?deleteSubtasks=true
```

**Python Example:**
```python
import requests
from requests.auth import HTTPBasicAuth

url = "https://your-domain.atlassian.net/rest/api/2/issue/PROJ-1?deleteSubtasks=true"
auth = HTTPBasicAuth("your-email@example.com", "your-api-token")
headers = {"Accept": "application/json"}

response = requests.delete(url, headers=headers, auth=auth)
print(response.status_code)
```

### Search Operations

#### Search Issues (JQL)

**Endpoint:** `GET /rest/api/2/search` or `POST /rest/api/2/search`

**Query Parameters (GET):**
- `jql`: Jira Query Language query
- `startAt`: Index of first result (default: 0)
- `maxResults`: Maximum number of results (default: 50, max: 100)
- `fields`: Comma-separated list of fields to return
- `expand`: Additional information to include

**JQL Examples:**
- `project = PROJ`
- `project = PROJ AND status = "In Progress"`
- `assignee = currentUser() AND status != Done`
- `created >= -7d ORDER BY created DESC`
- `text ~ "error" AND priority = High`

**cURL Example (GET):**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X GET \
   -H "Content-Type: application/json" \
   "https://your-domain.atlassian.net/rest/api/2/search?jql=project=PROJ%20AND%20status=Open&maxResults=50"
```

**cURL Example (POST):**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X POST \
   -H "Content-Type: application/json" \
   --data '{
     "jql": "project = PROJ AND status = \"In Progress\"",
     "startAt": 0,
     "maxResults": 50,
     "fields": ["summary", "status", "assignee"]
   }' \
   https://your-domain.atlassian.net/rest/api/2/search
```

**Python Example:**
```python
import requests
from requests.auth import HTTPBasicAuth
import json

url = "https://your-domain.atlassian.net/rest/api/2/search"
auth = HTTPBasicAuth("your-email@example.com", "your-api-token")
headers = {
    "Accept": "application/json",
    "Content-Type": "application/json"
}

payload = json.dumps({
    "jql": "project = PROJ AND status = \"In Progress\"",
    "startAt": 0,
    "maxResults": 50,
    "fields": ["summary", "status", "assignee", "created"]
})

response = requests.post(url, data=payload, headers=headers, auth=auth)
results = response.json()

print(f"Total issues: {results['total']}")
for issue in results['issues']:
    print(f"{issue['key']}: {issue['fields']['summary']}")
```

**Response:**
```json
{
  "expand": "names,schema",
  "startAt": 0,
  "maxResults": 50,
  "total": 100,
  "issues": [
    {
      "expand": "operations,versionedRepresentations,editmeta,changelog,renderedFields",
      "id": "10000",
      "self": "https://your-domain.atlassian.net/rest/api/2/issue/10000",
      "key": "PROJ-1",
      "fields": {
        "summary": "Issue summary",
        "status": {
          "name": "In Progress"
        }
      }
    }
  ]
}
```

### Project Operations

#### Get All Projects

**Endpoint:** `GET /rest/api/2/project`

**Query Parameters:**
- `expand`: Additional information to include
- `recent`: Return recently accessed projects (number)

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X GET \
   -H "Content-Type: application/json" \
   https://your-domain.atlassian.net/rest/api/2/project
```

#### Get Project

**Endpoint:** `GET /rest/api/2/project/{projectIdOrKey}`

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X GET \
   -H "Content-Type: application/json" \
   https://your-domain.atlassian.net/rest/api/2/project/PROJ
```

**Python Example:**
```python
import requests
from requests.auth import HTTPBasicAuth

url = "https://your-domain.atlassian.net/rest/api/2/project/PROJ"
auth = HTTPBasicAuth("your-email@example.com", "your-api-token")
headers = {"Accept": "application/json"}

response = requests.get(url, headers=headers, auth=auth)
project = response.json()

print(f"Project Key: {project['key']}")
print(f"Project Name: {project['name']}")
print(f"Project Type: {project['projectTypeKey']}")
```

### User Operations

#### Get User

**Endpoint:** `GET /rest/api/2/user`

**Query Parameters:**
- `accountId`: User account ID (required for Cloud)
- `username`: Username (Server/Data Center)
- `key`: User key (Server/Data Center)
- `expand`: Additional information to include

**cURL Example (Cloud):**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X GET \
   -H "Content-Type: application/json" \
   "https://your-domain.atlassian.net/rest/api/2/user?accountId=account-id-here"
```

#### Search Users

**Endpoint:** `GET /rest/api/2/user/search`

**Query Parameters:**
- `query`: Search query (username, display name, or email)
- `startAt`: Index of first result
- `maxResults`: Maximum number of results
- `includeActive`: Include active users (default: true)
- `includeInactive`: Include inactive users (default: false)

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X GET \
   -H "Content-Type: application/json" \
   "https://your-domain.atlassian.net/rest/api/2/user/search?query=john"
```

### Comment Operations

#### Add Comment

**Endpoint:** `POST /rest/api/2/issue/{issueIdOrKey}/comment`

**Request Body:**
```json
{
  "body": "This is a comment added via REST API"
}
```

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X POST \
   -H "Content-Type: application/json" \
   --data '{
     "body": "This is a comment added via REST API"
   }' \
   https://your-domain.atlassian.net/rest/api/2/issue/PROJ-1/comment
```

#### Get Comments

**Endpoint:** `GET /rest/api/2/issue/{issueIdOrKey}/comment`

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X GET \
   -H "Content-Type: application/json" \
   https://your-domain.atlassian.net/rest/api/2/issue/PROJ-1/comment
```

### Attachment Operations

#### Add Attachment

**Endpoint:** `POST /rest/api/2/issue/{issueIdOrKey}/attachments`

**Content-Type:** `multipart/form-data`

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X POST \
   -H "X-Atlassian-Token: no-check" \
   -F "file=@/path/to/file.pdf" \
   https://your-domain.atlassian.net/rest/api/2/issue/PROJ-1/attachments
```

**Python Example:**
```python
import requests
from requests.auth import HTTPBasicAuth

url = "https://your-domain.atlassian.net/rest/api/2/issue/PROJ-1/attachments"
auth = HTTPBasicAuth("your-email@example.com", "your-api-token")
headers = {
    "X-Atlassian-Token": "no-check"
}

with open("file.pdf", "rb") as f:
    files = {"file": f}
    response = requests.post(url, headers=headers, auth=auth, files=files)
    print(response.json())
```

### Transition Operations

#### Get Transitions

**Endpoint:** `GET /rest/api/2/issue/{issueIdOrKey}/transitions`

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X GET \
   -H "Content-Type: application/json" \
   https://your-domain.atlassian.net/rest/api/2/issue/PROJ-1/transitions
```

#### Transition Issue

**Endpoint:** `POST /rest/api/2/issue/{issueIdOrKey}/transitions`

**Request Body:**
```json
{
  "transition": {
    "id": "21"
  },
  "fields": {
    "resolution": {
      "name": "Fixed"
    }
  }
}
```

**cURL Example:**
```bash
curl -D- \
   -u your-email@example.com:your-api-token \
   -X POST \
   -H "Content-Type: application/json" \
   --data '{
     "transition": {
       "id": "21"
     },
     "fields": {
       "resolution": {
         "name": "Fixed"
       }
     }
   }' \
   https://your-domain.atlassian.net/rest/api/2/issue/PROJ-1/transitions
```

## Jira Query Language (JQL)

JQL is a powerful query language for searching issues in JIRA.

### Basic JQL Syntax

- **Equality:** `project = PROJ`
- **Inequality:** `status != Done`
- **Comparison:** `priority >= High`, `created >= -7d`
- **AND/OR:** `project = PROJ AND status = Open`
- **NOT:** `NOT status = Done`
- **IN:** `status IN (Open, "In Progress")`
- **LIKE:** `summary ~ "error"`
- **Order By:** `ORDER BY created DESC`

### Common JQL Examples

```jql
# All issues in a project
project = PROJ

# Issues assigned to current user
assignee = currentUser()

# Issues created in last 7 days
created >= -7d

# High priority bugs
issuetype = Bug AND priority = High

# Issues with specific text
text ~ "authentication error"

# Issues in specific status
status IN ("In Progress", "Review")

# Issues updated today
updated >= startOfDay()

# Issues due in next week
duedate >= startOfDay() AND duedate <= endOfWeek()

# Complex query
project = PROJ AND (status = Open OR status = "In Progress") AND priority >= High ORDER BY created DESC
```

### JQL Functions

- `currentUser()`: Current logged-in user
- `membersOf("group-name")`: Members of a group
- `startOfDay()`, `endOfDay()`: Time boundaries
- `startOfWeek()`, `endOfWeek()`: Week boundaries
- `startOfMonth()`, `endOfMonth()`: Month boundaries
- `startOfYear()`, `endOfYear()`: Year boundaries

## Request/Response Format

### Request Headers

Common headers for JIRA API requests:

- `Authorization`: Authentication credentials (Basic, Bearer, or OAuth)
- `Content-Type`: `application/json` for JSON payloads
- `Accept`: `application/json` for JSON responses
- `X-Atlassian-Token`: `no-check` (required for attachment uploads)

### Response Format

**Success Response:**
- **Status Codes:** 200 (OK), 201 (Created), 204 (No Content)
- **Body:** JSON object with requested data

**Error Response:**
- **Status Codes:** 400 (Bad Request), 401 (Unauthorized), 403 (Forbidden), 404 (Not Found), 409 (Conflict), 500 (Internal Server Error)
- **Body:**
```json
{
  "errorMessages": ["Error message 1", "Error message 2"],
  "errors": {
    "field": "Additional error details"
  }
}
```

## Rate Limits

JIRA Cloud enforces rate limits on API requests:

- **Standard:** 300 requests per minute per user
- **Premium/Enterprise:** Higher limits available
- **Rate Limit Headers:**
  - `X-RateLimit-Limit`: Maximum requests allowed
  - `X-RateLimit-Remaining`: Remaining requests
  - `X-RateLimit-Reset`: Time when limit resets

**Handling Rate Limits:**
```python
import requests
import time

def make_request_with_retry(url, auth, headers, max_retries=3):
    for attempt in range(max_retries):
        response = requests.get(url, auth=auth, headers=headers)
        
        if response.status_code == 429:  # Rate limit exceeded
            retry_after = int(response.headers.get('Retry-After', 60))
            print(f"Rate limit exceeded. Waiting {retry_after} seconds...")
            time.sleep(retry_after)
            continue
        
        return response
    
    raise Exception("Max retries exceeded")
```

## Pagination

For endpoints that return multiple results, use pagination:

**Parameters:**
- `startAt`: Index of first result (0-based)
- `maxResults`: Maximum number of results per page (default: 50, max: 100)

**Example:**
```python
def get_all_issues(jql, auth, headers, base_url):
    all_issues = []
    start_at = 0
    max_results = 100
    
    while True:
        url = f"{base_url}/rest/api/2/search"
        params = {
            "jql": jql,
            "startAt": start_at,
            "maxResults": max_results
        }
        
        response = requests.get(url, params=params, auth=auth, headers=headers)
        results = response.json()
        
        all_issues.extend(results['issues'])
        
        if start_at + max_results >= results['total']:
            break
        
        start_at += max_results
    
    return all_issues
```

## Best Practices

### 1. Authentication

- Use API tokens instead of passwords for Basic Auth
- Prefer OAuth 2.0 for production applications
- Store credentials securely (environment variables, secret management)
- Implement token refresh for OAuth 2.0

### 2. Error Handling

```python
import requests
from requests.auth import HTTPBasicAuth

def handle_jira_response(response):
    if response.status_code == 200:
        return response.json()
    elif response.status_code == 401:
        raise Exception("Authentication failed. Check credentials.")
    elif response.status_code == 403:
        raise Exception("Permission denied. Check user permissions.")
    elif response.status_code == 404:
        raise Exception("Resource not found.")
    elif response.status_code == 429:
        raise Exception("Rate limit exceeded.")
    else:
        error_data = response.json() if response.text else {}
        error_messages = error_data.get('errorMessages', [])
        errors = error_data.get('errors', {})
        raise Exception(f"API Error: {error_messages} {errors}")
```

### 3. Field Selection

Only request fields you need to reduce response size:

```python
# Instead of getting all fields
response = requests.get(f"{url}/issue/PROJ-1", auth=auth, headers=headers)

# Request specific fields
response = requests.get(
    f"{url}/issue/PROJ-1?fields=summary,status,assignee",
    auth=auth,
    headers=headers
)
```

### 4. Batch Operations

Use batch endpoints when available to reduce API calls:

```python
# Instead of multiple individual requests
for issue_key in issue_keys:
    response = requests.get(f"{url}/issue/{issue_key}", auth=auth, headers=headers)

# Use search with JQL
jql = f"key IN ({','.join(issue_keys)})"
response = requests.post(f"{url}/search", json={"jql": jql}, auth=auth, headers=headers)
```

### 5. Caching

Cache frequently accessed data:

```python
from functools import lru_cache
import time

@lru_cache(maxsize=100)
def get_cached_project(project_key, cache_time=3600):
    # Implementation with caching
    pass
```

## Example Integration Scenarios

### Scenario 1: Create Issue with Full Configuration

```python
import requests
from requests.auth import HTTPBasicAuth
import json

def create_issue_with_details():
    url = "https://your-domain.atlassian.net/rest/api/2/issue"
    auth = HTTPBasicAuth("your-email@example.com", "your-api-token")
    headers = {
        "Accept": "application/json",
        "Content-Type": "application/json"
    }
    
    payload = json.dumps({
        "fields": {
            "project": {"key": "PROJ"},
            "summary": "New feature request",
            "description": {
                "type": "doc",
                "version": 1,
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": "Detailed description of the feature."
                            }
                        ]
                    }
                ]
            },
            "issuetype": {"name": "Story"},
            "priority": {"name": "High"},
            "labels": ["feature", "backend"],
            "components": [{"name": "API"}],
            "customfield_10001": "Custom field value"  # Custom field ID
        }
    })
    
    response = requests.post(url, data=payload, headers=headers, auth=auth)
    return response.json()
```

### Scenario 2: Search and Process Issues

```python
def search_and_process_issues(jql_query):
    url = "https://your-domain.atlassian.net/rest/api/2/search"
    auth = HTTPBasicAuth("your-email@example.com", "your-api-token")
    headers = {
        "Accept": "application/json",
        "Content-Type": "application/json"
    }
    
    payload = json.dumps({
        "jql": jql_query,
        "startAt": 0,
        "maxResults": 100,
        "fields": ["summary", "status", "assignee", "created"]
    })
    
    response = requests.post(url, data=payload, headers=headers, auth=auth)
    results = response.json()
    
    for issue in results['issues']:
        print(f"Processing {issue['key']}: {issue['fields']['summary']}")
        # Process each issue
        process_issue(issue)
    
    return results
```

### Scenario 3: Update Multiple Issues

```python
def update_multiple_issues(issue_keys, updates):
    auth = HTTPBasicAuth("your-email@example.com", "your-api-token")
    headers = {
        "Accept": "application/json",
        "Content-Type": "application/json"
    }
    
    for issue_key in issue_keys:
        url = f"https://your-domain.atlassian.net/rest/api/2/issue/{issue_key}"
        payload = json.dumps({"fields": updates})
        
        response = requests.put(url, data=payload, headers=headers, auth=auth)
        if response.status_code == 204:
            print(f"Updated {issue_key}")
        else:
            print(f"Failed to update {issue_key}: {response.status_code}")
```

### Scenario 4: Add Comment and Transition Issue

```python
def add_comment_and_close(issue_key, comment_text):
    auth = HTTPBasicAuth("your-email@example.com", "your-api-token")
    headers = {
        "Accept": "application/json",
        "Content-Type": "application/json"
    }
    base_url = "https://your-domain.atlassian.net/rest/api/2"
    
    # Add comment
    comment_url = f"{base_url}/issue/{issue_key}/comment"
    comment_payload = json.dumps({"body": comment_text})
    requests.post(comment_url, data=comment_payload, headers=headers, auth=auth)
    
    # Get available transitions
    transitions_url = f"{base_url}/issue/{issue_key}/transitions"
    transitions_response = requests.get(transitions_url, headers=headers, auth=auth)
    transitions = transitions_response.json()
    
    # Find "Close" or "Done" transition
    close_transition = next(
        (t for t in transitions['transitions'] if 'close' in t['name'].lower() or 'done' in t['name'].lower()),
        None
    )
    
    if close_transition:
        # Transition issue
        transition_url = f"{base_url}/issue/{issue_key}/transitions"
        transition_payload = json.dumps({
            "transition": {"id": close_transition['id']}
        })
        requests.post(transition_url, data=transition_payload, headers=headers, auth=auth)
        print(f"Issue {issue_key} closed")
```

## References

- **Official JIRA REST API Documentation:** https://developer.atlassian.com/cloud/jira/platform/rest/v3/
- **JIRA REST API v2 Documentation:** https://docs.atlassian.com/software/jira/docs/api/REST/8.22.0/
- **Basic Authentication Guide:** https://developer.atlassian.com/cloud/jira/platform/basic-auth-for-rest-apis/
- **OAuth 2.0 Guide:** https://developer.atlassian.com/cloud/jira/platform/oauth-2-3lo-apps/
- **JQL Documentation:** https://support.atlassian.com/jira-service-management-cloud/docs/use-advanced-search-with-jira-query-language-jql/
- **Atlassian Developer Console:** https://developer.atlassian.com/console/
- **API Token Management:** https://id.atlassian.com/manage-profile/security/api-tokens

## Notes

- JIRA Cloud uses account IDs instead of usernames for user references
- API v3 is recommended for new integrations, but v2 is still widely supported
- Custom fields require their field ID (e.g., `customfield_10001`)
- Some operations require specific permissions in JIRA
- Rate limits apply to prevent API abuse
- Use pagination for large result sets
- JQL is case-sensitive for field names but not for values
- OAuth 2.0 access tokens expire and need to be refreshed using refresh tokens

