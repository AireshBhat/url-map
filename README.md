# URL Shortener Service

A high-performance URL shortening service built with Rust, featuring a layered architecture, async I/O, and PostgreSQL storage.

## Features

- **URL Shortening**: Create short, memorable URLs from long ones
- **Visit Tracking**: Track the number of visits to each shortened URL
- **Statistics**: View usage statistics for shortened URLs
- **Async I/O**: Built with async/await for high performance
- **PostgreSQL Storage**: Reliable, persistent storage with proper indexing
- **RESTful API**: Clean, well-documented HTTP endpoints

## Architecture

The service follows a clean, layered architecture:

```
┌─────────────┐
│  HTTP Layer │ → Request handling, routing, middleware
├─────────────┤
│  Handlers   │ → Input validation, response formatting
├─────────────┤
│  Services   │ → Business logic, URL generation
├─────────────┤
│  Storage    │ → Data persistence (PostgreSQL)
└─────────────┘
```

## API Endpoints

### Create Short URL
```http
POST /api/shorten
Content-Type: application/json

{
    "url": "https://example.com/very/long/url"
}
```

Response:
```json
{
    "short_url": "abc123",
    "original_url": "https://example.com/very/long/url",
    "created_at": "2024-03-20T00:00:00Z",
    "visits": 0
}
```

### Redirect to Original URL
```http
GET /{short_code}
```
Redirects to the original URL and increments visit counter.

### Get URL Statistics
```http
GET /api/stats/{short_code}
```

Response:
```json
{
    "short_url": "abc123",
    "original_url": "https://example.com/very/long/url",
    "created_at": "2024-03-20T00:00:00Z",
    "visits": 42
}
```

## Setup

### Prerequisites

- Rust (latest stable)
- PostgreSQL 15 or later
- SQLx CLI

### Database Setup

1. Install and start PostgreSQL:
```bash
brew install postgresql@15
brew services start postgresql@15
```

2. Follow the database setup guide:
```bash
# See docs/postgres_setup.md for detailed instructions
```

### Environment Configuration

Create `.env` file:
```env
DATABASE_URL=postgresql://your_username:your_password@localhost/url_shortener
POSTGRES_MAX_CONNECTIONS=5
POSTGRES_CONNECTION_TIMEOUT_SECS=30
PORT=8080
RUST_LOG=debug
```

### Build and Run

```bash
# Install SQLx CLI
cargo install sqlx-cli

# Run database migrations
sqlx migrate run

# Build and run the service
cargo run

# Run tests
cargo test
```

## Project Structure

```
src/
├── handlers/       # Request handlers
├── services/      # Business logic
├── storage/       # Data persistence
├── models/        # Data structures
├── errors/        # Error types
└── main.rs        # Application entry

migrations/        # Database migrations
docs/             # Documentation
tests/            # Integration tests
```

## Testing

### Unit Tests
```bash
cargo test --lib
```

### Integration Tests
```bash
cargo test --test '*'
```

### API Tests
See `tests/api/` for example API requests and responses.

## Error Handling

The service uses custom error types that map to appropriate HTTP status codes:

- 400 Bad Request: Invalid URL or input
- 404 Not Found: Short URL not found
- 500 Internal Server Error: Database errors

## Performance Considerations

- Connection pooling for database access
- Async I/O throughout the stack
- Proper database indexing
- Efficient URL generation algorithm

## Contributing

1. Fork the repository
2. Create your feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
