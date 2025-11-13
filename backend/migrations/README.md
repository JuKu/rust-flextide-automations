# Database Migrations

This directory contains SQLx database migrations for the Flextide platform.

## Quick Start

1. **Set up your database connection** in `backend/.env`:
   ```bash
   # PostgreSQL
   DATABASE_URL=postgres://username:password@localhost:5432/flextide_db
   
   # MySQL
   DATABASE_URL=mysql://username:password@localhost:3306/flextide_db
   ```

2. **Create a new migration** (from `backend/` directory):
   ```bash
   cd backend
   sqlx migrate add <migration_name>
   ```

   Or from project root:
   ```bash
   sqlx migrate add <migration_name> --source backend/migrations
   ```

3. **Apply migrations** (from `backend/` directory):
   ```bash
   cd backend
   sqlx migrate run
   ```

   Or from project root:
   ```bash
   sqlx migrate run --source backend/migrations
   ```

4. **Check migration status**:
   ```bash
   sqlx migrate info --source backend/migrations
   ```

## Migration Files

Migration files follow the naming pattern: `{timestamp}_{description}.sql`

Example: `20250115120000_create_users_table.sql`

## Best Practices

- Always test migrations on both MySQL and PostgreSQL
- Never modify existing migrations - create new ones to fix issues
- Use descriptive migration names
- Keep migrations small and focused
- Document breaking changes in migration comments

## Database Support

This project supports both:
- **PostgreSQL** (recommended for production)
- **MySQL** (also supported)

When writing migrations, ensure compatibility with both databases or provide database-specific variants.

## References

See `.cursor/rules/database_migrations.mdc` for detailed migration rules and best practices.

