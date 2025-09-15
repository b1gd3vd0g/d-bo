# D-Bo Backend

_Backend API for a multiplayer, real-time card game inspired by a familiar classic._

![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-blue)
![Rust](https://img.shields.io/badge/Rust-Edition%202024-orange)

---

## Project Status

This project is a **work in progress**.

- Currently **functional**:
  - Player account creation via REST API
  - Email verification and rejection flow (via SMTP)
  - MongoDB integration
- Currently **in progress**:
  - JWT-based authentication (HS256, 15m access tokens / 7d refresh tokens)
- Not yet implemented:
  - WebSocket functionality
  - Friend system
  - Game lobbies and real-time play
  - Stats tracking and messaging

For a detailed breakdown of planned steps and progress, see [ROADMAP.md](./ROADMAP.md).

---

## Features (Planned)

- Secure account creation with email verification
- Login using username or email + password
- JWT-based authentication with refresh tokens
- Add and manage friends
- Create and join multiplayer games, including custom rules
- Real-time gameplay over WebSockets
- Game stats tracking (wins, losses, draws, dropouts)
- Possible in-game messaging

---

## Tech Stack

- **Language**: Rust (Edition 2024)
- **Framework**: Axum
- **Database**: MongoDB
- **Auth**: Argon2 password hashing + JWT (HS256)
- **Email**: SMTP via Lettre
- **Async runtime**: Tokio

---

## Dependencies

| Crate        | Version | Purpose                                    |
| ------------ | ------- | ------------------------------------------ |
| argon2       | 0.5.3   | Secure password hashing                    |
| axum         | 0.8.4   | Web framework for REST API                 |
| axum_extra   | 0.10.1  | Cookie functionality                       |
| base64       | 0.22.1  | Base 64 encoding                           |
| bson         | 2.15.0  | BSON support with Chrono integration       |
| chrono       | 0.4.41  | Date/time handling with Serde              |
| dotenvy      | 0.15.7  | Environment variable loading               |
| futures      | 0.3.31  | Async traits for iterating mongodb cursors |
| jsonwebtoken | 9.3.1   | JWT creation and validation (HS256)        |
| lettre       | 0.11.18 | Email sending via SMTP                     |
| mongodb      | 3.2.5   | MongoDB driver                             |
| once_cell    | 1.21.3  | Lazy-loaded values                         |
| rand         | 0.9.2   | Shuffling cards                            |
| regex        | 1.11.1  | Regex for validation                       |
| serde        | 1.0.219 | Serialization and deserialization          |
| tokio        | 1.47.1  | Async runtime                              |
| tower-http   | 0.6.6   | Middleware (CORS support)                  |
| urlencoding  | 2.1.3   | URL encoding/decoding                      |
| uuid         | 1.18.0  | UUID generation (v4) with Serde support    |

---

## Setup

This project is **not production-ready**. Running locally requires manual configuration.

### Prerequisites

- Rust toolchain (Edition 2024)
- MongoDB instance
- SMTP email account

### Environment Variables

Configure a `.env` file with the following:

```
AUTHN_TOKEN_SECRET=your_jwt_secret

MONGO_USERNAME=your_username
MONGO_PASSWORD=your_password
MONGO_SERVER=your_server
MONGO_DBNAME=your_dbname

SMTP_HOST=your_smtp_host
SMTP_USERNAME=your_email
SMTP_PASSWORD=your_password
```

### Running

```bash
cargo run
```

## Testing

Unit testing is currently minimal. Input validation functions for usernames, passwords, and emails are covered.
Run tests with:

```bash
cargo test
```

## Future Deployment

When production-ready, the backend will be deployed at https://api.d-bo.bigdevdog.com

Planned:

- `docker-compose.yaml` to provide a full test environment (MongoDB, backend, frontend).
- CI/CD pipeline for builds and tests.

## Contact

Contact **Devin Peevy** at [devin.peevy@outlook.com](mailto:devin.peevy@outlook.com).
