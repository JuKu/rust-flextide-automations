# Docs Module

The Docs module provides functionality for managing documentation and related resources within the Flextide platform.

## Features

- Document management
- Organization-scoped data isolation

## Usage

This module provides REST API endpoints under the `/modules/docs` path.

## API Endpoints

- `GET /modules/docs/health` - Health check endpoint
- `GET /modules/docs/documents` - List all documents (TODO: implement)

## Database Schema

The Docs module will use database tables prefixed with `module_docs_` to follow the module naming convention.

## Organization Scoping

All Docs data is scoped to organizations. All queries automatically filter by the current organization context.

