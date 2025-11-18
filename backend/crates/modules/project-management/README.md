# Project Management Module

The Project Management module provides functionality for managing projects, tasks, and related resources within the Flextide platform.

## Features

- Project management
- Task tracking
- Organization-scoped data isolation

## Usage

This module provides REST API endpoints under the `/modules/project-management` path.

## API Endpoints

- `GET /modules/project-management/health` - Health check endpoint
- `GET /modules/project-management/projects` - List all projects (TODO: implement)

## Database Schema

The Project Management module will use database tables prefixed with `module_project_management_` to follow the module naming convention.

## Organization Scoping

All Project Management data is scoped to organizations. All queries automatically filter by the current organization context.

