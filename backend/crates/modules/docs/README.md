# Docs Module

The Docs module provides functionality for managing documentation and related resources within the Flextide platform.

## Features

- Document management
- Organization-scoped data isolation
- Area-based organization of documentation
- Page versioning
- Permission-based access control

## Usage

This module provides REST API endpoints under the `/modules/docs` path and can be used as a library module.

## API Endpoints

### Health Check
- `GET /modules/docs/health` - Health check endpoint

### Areas
- `POST /modules/docs/areas` - Create a new documentation area
- `GET /modules/docs/areas/{uuid}` - Get an area by UUID
- `PUT /modules/docs/areas/{uuid}` - Update an area
- `DELETE /modules/docs/areas/{uuid}` - Delete an area

### Documents
- `GET /modules/docs/documents` - List all documents (TODO: implement)

## Library Functions

### Area Functions

#### `create_area`
Creates a new documentation area in the database.

**Parameters:**
- `pool: &DatabasePool` - Database connection pool
- `organization_uuid: &str` - UUID of the organization
- `user_uuid: &str` - UUID of the user creating the area
- `request: CreateDocsAreaRequest` - Area creation request

**Returns:** `Result<String, DocsAreaDatabaseError>` - UUID of the newly created area

**Errors:**
- User does not belong to the organization
- User does not have permission to create areas
- Short name is empty
- Database operation fails

#### `load_area_by_uuid`
Loads an area from the database by UUID.

**Parameters:**
- `pool: &DatabasePool` - Database connection pool
- `area_uuid: &str` - UUID of the area

**Returns:** `Result<DocsArea, DocsAreaDatabaseError>` - The area data

**Errors:**
- Area not found
- Database operation fails

#### `update_area`
Updates an area in the database.

**Parameters:**
- `pool: &DatabasePool` - Database connection pool
- `area_uuid: &str` - UUID of the area to update
- `organization_uuid: &str` - UUID of the organization (for verification)
- `user_uuid: &str` - UUID of the user updating the area
- `request: UpdateDocsAreaRequest` - Update request with fields to update

**Returns:** `Result<(), DocsAreaDatabaseError>`

**Errors:**
- User does not belong to the organization
- Area does not belong to the organization
- User does not have permission to edit the area
- Database operation fails

#### `delete_area`
Deletes an area from the database.

**Parameters:**
- `pool: &DatabasePool` - Database connection pool
- `area_uuid: &str` - UUID of the area to delete
- `organization_uuid: &str` - UUID of the organization (for verification)
- `user_uuid: &str` - UUID of the user deleting the area

**Returns:** `Result<(), DocsAreaDatabaseError>`

**Errors:**
- User does not belong to the organization
- Area does not belong to the organization
- User does not have permission to delete the area
- Area is not deletable
- Database operation fails

#### `load_area_member_permissions`
Loads area member permissions for a user in an area.

**Parameters:**
- `pool: &DatabasePool` - Database connection pool
- `area_uuid: &str` - UUID of the area
- `user_uuid: &str` - UUID of the user

**Returns:** `Result<Option<AreaMemberPermissions>, DocsAreaDatabaseError>` - Permissions if user is a member, None otherwise

**Errors:**
- Database operation fails

### Page Functions

#### `create_page`
Creates a new page in the database.

**Parameters:**
- `pool: &DatabasePool` - Database connection pool
- `organization_uuid: &str` - UUID of the organization the page belongs to
- `user_uuid: &str` - UUID of the user creating the page
- `request: CreateDocsPageRequest` - Page creation request

**Returns:** `Result<String, DocsPageDatabaseError>` - UUID of the newly created page

**Errors:**
- User does not belong to the organization
- User does not have permission to create pages
- Area does not belong to the organization
- Title is empty
- Database operation fails

#### `delete_page`
Deletes a page from the database.

**Parameters:**
- `pool: &DatabasePool` - Database connection pool
- `page_uuid: &str` - UUID of the page to delete
- `organization_uuid: &str` - UUID of the organization
- `user_uuid: &str` - UUID of the user deleting the page

**Returns:** `Result<(), DocsPageDatabaseError>`

**Errors:**
- User does not belong to the organization
- User does not have permission to delete pages
- Page does not belong to the organization
- Page not found
- Database operation fails

#### `load_page_with_version`
Loads a page with its current version by page UUID.

**Parameters:**
- `pool: &DatabasePool` - Database connection pool
- `page_uuid: &str` - UUID of the page

**Returns:** `Result<DocsPageWithVersion, DocsPageDatabaseError>` - Page data with current version (if exists)

**Errors:**
- Page not found
- Database operation fails

**Note:** If `current_version_uuid` is set, loads that version. Otherwise, loads the latest version by `version_number`.

#### `get_page_user_permissions`
Gets permissions for a specific user for a specific page.

**Parameters:**
- `pool: &DatabasePool` - Database connection pool
- `page_uuid: &str` - UUID of the page
- `user_uuid: &str` - UUID of the user

**Returns:** `Result<Option<AreaMemberPermissions>, DocsPageDatabaseError>` - Permissions if user has permissions in the page's area, None if not a member

**Errors:**
- Page not found
- Area not found
- Database operation fails

## Data Structures

### Area Types

#### `DocsArea`
Represents a documentation area.

**Fields:**
- `uuid: String` - Unique identifier
- `organization_uuid: String` - Organization this area belongs to
- `short_name: String` - Short name/title of the area
- `description: Option<String>` - Optional description
- `icon_name: Option<String>` - Optional icon identifier
- `public: bool` - Whether the area is public
- `visible: bool` - Whether the area is visible
- `deletable: bool` - Whether the area can be deleted
- `creator_uuid: String` - UUID of the user who created the area
- `created_at: DateTime<Utc>` - Creation timestamp

#### `CreateDocsAreaRequest`
Request structure for creating a new area.

**Fields:**
- `short_name: String` - Short name/title (required)
- `description: Option<String>` - Optional description
- `icon_name: Option<String>` - Optional icon identifier
- `public: Option<bool>` - Whether the area is public (default: false)
- `visible: Option<bool>` - Whether the area is visible (default: true)
- `deletable: Option<bool>` - Whether the area can be deleted (default: true)

#### `UpdateDocsAreaRequest`
Request structure for updating an area.

**Fields:** (all optional)
- `short_name: Option<String>` - Short name/title
- `description: Option<String>` - Description
- `icon_name: Option<String>` - Icon identifier
- `public: Option<bool>` - Whether the area is public
- `visible: Option<bool>` - Whether the area is visible
- `deletable: Option<bool>` - Whether the area can be deleted

#### `AreaMemberPermissions`
Represents permissions for a user in an area.

**Fields:**
- `role: String` - User's role (owner, admin, member, guest)
- `can_view: bool` - Can view pages
- `can_add_pages: bool` - Can add pages
- `can_edit_pages: bool` - Can edit any pages
- `can_edit_own_pages: bool` - Can edit own pages
- `can_archive_pages: bool` - Can archive any pages
- `can_archive_own_pages: bool` - Can archive own pages
- `can_delete_pages: bool` - Can delete any pages
- `can_delete_own_pages: bool` - Can delete own pages
- `can_export_pages: bool` - Can export pages
- `admin: bool` - Is admin in the area

### Page Types

#### `DocsPage`
Represents a documentation page.

**Fields:**
- `uuid: String` - Unique identifier
- `organization_uuid: String` - Organization this page belongs to
- `area_uuid: String` - Area this page belongs to
- `title: String` - Page title
- `short_summary: Option<String>` - Optional short summary
- `parent_page_uuid: Option<String>` - UUID of parent page (for hierarchical structure)
- `current_version_uuid: Option<String>` - UUID of the current version
- `page_type: String` - Type of page (markdown_page, json_document, database, sheet)
- `last_updated: DateTime<Utc>` - Last update timestamp
- `created_at: DateTime<Utc>` - Creation timestamp

#### `DocsPageVersion`
Represents a page version.

**Fields:**
- `uuid: String` - Unique identifier
- `page_uuid: String` - UUID of the page this version belongs to
- `version_number: i32` - Version number
- `content: String` - Version content
- `last_updated: Option<DateTime<Utc>>` - Last update timestamp
- `created_at: DateTime<Utc>` - Creation timestamp

#### `DocsPageWithVersion`
Combines page data with its current version.

**Fields:**
- All fields from `DocsPage`
- `version: Option<DocsPageVersion>` - Current version (if exists)

#### `CreateDocsPageRequest`
Request structure for creating a new page.

**Fields:**
- `area_uuid: String` - UUID of the area (required)
- `title: String` - Page title (required)
- `short_summary: Option<String>` - Optional short summary
- `parent_page_uuid: Option<String>` - UUID of parent page
- `page_type: Option<String>` - Page type (default: "markdown_page")

### Error Types

#### `DocsAreaDatabaseError`
Error type for area database operations.

**Variants:**
- `Database(DatabaseError)` - Database error
- `Sql(sqlx::Error)` - SQL execution error
- `UserNotInOrganization` - User does not belong to organization
- `PermissionDenied` - User does not have permission
- `AreaNotFound` - Area not found
- `AreaNotInOrganization` - Area does not belong to organization
- `EmptyShortName` - Short name cannot be empty

#### `DocsPageDatabaseError`
Error type for page database operations.

**Variants:**
- `Database(DatabaseError)` - Database error
- `Sql(sqlx::Error)` - SQL execution error
- `UserNotInOrganization` - User does not belong to organization
- `PermissionDenied` - User does not have permission
- `PageNotFound` - Page not found
- `PageNotInOrganization` - Page does not belong to organization
- `AreaNotFound` - Area not found
- `AreaNotInOrganization` - Area does not belong to organization
- `EmptyTitle` - Title cannot be empty

## Database Schema

The Docs module uses database tables prefixed with `module_docs_` to follow the module naming convention.

### Tables
- `module_docs_areas` - Documentation areas
- `module_docs_area_members` - Area member permissions
- `module_docs_pages` - Documentation pages
- `module_docs_page_versions` - Page versions

## Organization Scoping

All Docs data is scoped to organizations. All queries automatically filter by the current organization context.

## Permissions

The module supports both organization-level and area-level permissions:

### Organization-Level Permissions
- `module_docs_can_create_areas` - Can create areas
- `module_docs_can_edit_all_areas` - Can edit all areas
- `module_docs_can_edit_own_areas` - Can edit own areas
- `module_docs_can_archive_areas` - Can archive areas
- `module_docs_can_archive_own_areas` - Can archive own areas
- `module_docs_can_delete_areas` - Can delete areas
- `module_docs_can_delete_own_areas` - Can delete own areas

### Area-Level Permissions
Area members can have granular permissions for pages within an area, including view, add, edit, archive, delete, and export capabilities.

