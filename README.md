# Rust Backend Microservices

A modern, scalable backend system built with Rust, featuring microservices architecture and best practices.

## 🚀 Features

- Microservices architecture
- RESTful API endpoints
- JWT Authentication
- Database integration with PostgreSQL
- Image processing capabilities
- Firebase Cloud Messaging (FCM) support
- Swagger/OpenAPI documentation
- Docker support
- Comprehensive logging and tracing

## 🏗️ Project Structure

The project is organized into several crates (Rust packages):

- `api/`: API endpoints and route handlers
- `app/`: Application business logic
- `core_app/`: Core application functionality
- `domain/`: Domain models and business rules
- `infra/`: Infrastructure components (database, external services)
- `utils/`: Shared utilities and helper functions

## 🛠️ Technology Stack

- **Framework**: Axum
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT
- **API Documentation**: OpenAPI/Swagger
- **Image Processing**: Image crate with TurboJPEG
- **Push Notifications**: Firebase Cloud Messaging
- **Configuration**: Config crate
- **Logging**: Tracing
- **Async Runtime**: Tokio

## 📋 Prerequisites

- Rust (latest stable version)
- PostgreSQL
- Docker (optional)
- Make (for using Makefile commands)

## 🔧 Setup and Installation

1. Clone the repository:

   ```bash
   git clone [repository-url]
   cd na-spa-be
   ```

2. Install dependencies:

   ```bash
   cargo build
   ```

3. Set up environment variables:

   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. Run database migrations:

   ```bash
   cargo run --bin migrate
   ```

5. Start the server:
   ```bash
   cargo run
   ```

## 🐳 Docker Support

The project includes Docker support for easy deployment. See `docker.md` for detailed instructions.

## 📚 API Documentation

Once the server is running, you can access the Swagger documentation at:

```
http://localhost:8000/swagger-ui/
```

## 🔐 Security

- JWT-based authentication
- Argon2 password hashing
- Secure configuration management
- Input validation and sanitization

## 📝 Development

### Code Style

The project uses rustfmt for code formatting. Run:

```bash
cargo fmt
```

### Pre-commit Hooks

The project includes pre-commit hooks for code quality. Install them with:

```bash
pre-commit install
```

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## 📄 License

[Add your license information here]

## 👥 Authors

[Add author information here]
