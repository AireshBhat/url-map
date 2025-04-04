# PostgreSQL Setup Guide for URL Shortener

This guide covers the complete setup of PostgreSQL for the URL shortener project.

## Prerequisites

### 1. Install PostgreSQL (macOS)

Using Homebrew:
```bash
brew install postgresql@15
```

### 2. Start PostgreSQL Service

```bash
brew services start postgresql@15
```

To verify PostgreSQL is running:
```bash
brew services list | grep postgresql
```

Expected output should show postgresql@15 as "started".

## Database Setup

### 1. Create Database and User

```bash
# Connect to PostgreSQL as the default user
psql postgres

# Create the database
CREATE DATABASE url_shortener;

# Create a user (if not exists) and grant privileges
CREATE USER your_username WITH PASSWORD 'your_password';
ALTER USER your_username WITH SUPERUSER;
GRANT ALL PRIVILEGES ON DATABASE url_shortener TO your_username;

# Exit psql
\q
```

### 2. Configure Environment Variables

Create or update `.env` file in your project root:

```env
DATABASE_URL=postgresql://your_username:your_password@localhost/url_shortener
POSTGRES_MAX_CONNECTIONS=5
POSTGRES_CONNECTION_TIMEOUT_SECS=30
```

## Database Migration

### 1. Install SQLx CLI

```bash
cargo install sqlx-cli
```

### 2. Run Migrations

```bash
# Create database if it doesn't exist
sqlx database create

# Run all migrations
sqlx migrate run
```

### 3. Verify Setup

```bash
# Connect to the url_shortener database
psql url_shortener

# List all tables
\dt

# Describe the shortened_urls table
\d shortened_urls

# Exit psql
\q
```

Expected output should show the `shortened_urls` table with the following schema:
```sql
                                    Table "public.shortened_urls"
    Column    |           Type           | Collation | Nullable |          Default           
--------------+--------------------------+-----------+----------+----------------------------
 id           | bigint                   |           | not null | nextval('shortened_urls_id_seq'::regclass)
 original_url | text                     |           | not null | 
 short_url    | character varying(10)    |           | not null | 
 created_at   | timestamp with time zone |           | not null | CURRENT_TIMESTAMP
 visits       | bigint                   |           | not null | 0
```

## Troubleshooting

### 1. Permission Issues

If you encounter permission errors:

```bash
# Connect to PostgreSQL as superuser
psql postgres

# Grant necessary permissions
ALTER USER your_username WITH SUPERUSER;
GRANT ALL PRIVILEGES ON DATABASE url_shortener TO your_username;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO your_username;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO your_username;
```

### 2. Connection Issues

If you can't connect to PostgreSQL:

1. Check if PostgreSQL is running:
```bash
brew services list | grep postgresql
```

2. Verify connection settings:
```bash
psql -U your_username -d url_shortener -h localhost
```

3. Check PostgreSQL logs:
```bash
tail -f /usr/local/var/log/postgresql@15.log
```

### 3. Migration Issues

If migrations fail:

1. Drop and recreate the database:
```bash
sqlx database drop
sqlx database create
sqlx migrate run
```

2. Check migration files in `./migrations` directory:
```bash
ls -l migrations/
```

## Testing the Setup

1. Test database connection:
```bash
psql url_shortener -c "SELECT version();"
```

2. Test table creation:
```bash
psql url_shortener -c "\d shortened_urls"
```

3. Test basic operations:
```sql
-- Insert test URL
INSERT INTO shortened_urls (original_url, short_url) 
VALUES ('https://example.com', 'test123');

-- Query the inserted URL
SELECT * FROM shortened_urls WHERE short_url = 'test123';

-- Clean up test data
DELETE FROM shortened_urls WHERE short_url = 'test123';
```

## Maintenance

### 1. Backup Database

```bash
pg_dump url_shortener > backup.sql
```

### 2. Monitor Connections

```sql
SELECT * FROM pg_stat_activity WHERE datname = 'url_shortener';
```

### 3. Reset Statistics

```sql
SELECT pg_stat_reset();
```

## Additional Resources

- [PostgreSQL Documentation](https://www.postgresql.org/docs/15/index.html)
- [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)
- [Rust PostgreSQL Driver](https://docs.rs/postgres/latest/postgres/) 