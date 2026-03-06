# Sing-Box Config Bot (Rust)

A Telegram bot for managing sing-box VPN configurations

## Features

- User registration via Telegram
- Admin approval workflow for new users
- Automatic VPN UUID generation for accepted users
- sing-box server config generation with all accepted users
- Remote deployment utility via SSH
- SQLite database with Diesel ORM

## Project Structure

```
sing-box-config-bot/
├── src/
│   ├── adapters/           # Database repository implementations
│   │   ├── mod.rs
│   │   ├── user_repo.rs           # User repository
│   │   └── vless_identity_repo.rs # VLESS identity repository
│   ├── config/             # Configuration structures
│   │   └── deploy_config.rs       # Multi-server deploy config
│   ├── db/                 # Database schema and models
│   ├── domain/             # Business logic (extracted from utils)
│   │   ├── mod.rs
│   │   ├── config_generator.rs    # sing-box config generation
│   │   └── routing_config.rs      # Routing rules configuration
│   ├── ports/              # Interface definitions (traits)
│   ├── service/            # Application services
│   ├── singbox/            # sing-box data structures
│   ├── utils/              # Technical utilities (logging)
│   ├── main.rs             # Bot + HTTP server entry point
│   ├── lib.rs              # Library root - exports public API
│   ├── deploy.rs           # Multi-machine deployment utility
│   ├── gen_client_config.rs # Client config generator
│   └── gen_node_config.rs   # Server config generator
├── config/
│   ├── domains.json        # Routing rules configuration
│   └── ...
├── Cargo.toml
└── .env.example
```

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- SQLite3
- libssh2 (for deployment utility)

## Installation

1. Clone the repository and navigate to the Rust directory:
   ```bash
   cd rust
   ```

2. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

3. Edit `.env` and fill in your configuration values.

4. Build the project:
   ```bash
   cargo build --release
   ```

## Configuration

### Application Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `TG_BOT_TOKEN` | Telegram bot token | Yes |
| `TG_ADMIN_ID` | Telegram admin user ID | Yes |
| `CLIENT_CONFIG_ENDPOINT` | Base URL for config downloads (e.g., `https://example.com`) | Yes |
| `DB_LOCATION` | SQLite database path | No (default: `./db/vpn_signaling_server.db`) |
| `LOG_LEVEL` | Logging level | No (default: `info`) |
| `LOG_DISABLE_TIMESTAMP` | Disable timestamps in logs | No (default: `false`) |

### Routing Configuration

The `config/domains.json` file contains routing rules for the client configuration:

```json
{
  "dns_proxy_keywords": ["example.com"],
  "dns_direct_keywords": ["localhost"],
  "dns_direct_regex": [".*\\.ru$"],
  "direct_route_keywords": ["telegram.org"]
}
```

| Field | Description |
|-------|-------------|
| `dns_proxy_keywords` | Domains that should use proxy DNS |
| `dns_direct_keywords` | Domains that should use direct DNS |
| `dns_direct_regex` | Regex patterns for direct DNS routing |
| `direct_route_keywords` | Domains that bypass the proxy |

### Sing-Box Server Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `SING_BOX_PRIVATE_KEY` | Reality private key | Yes |
| `SING_BOX_SHORT_ID` | Reality short ID | Yes |
| `SING_BOX_SERVER_NAME` | Server name for Reality (e.g., google.com) | No (default: google.com) |
| `SING_BOX_SERVER_PORT` | Server port | No (default: 443) |

### Deployment Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `DEPLOY_HOST` | Remote server hostname | Yes |
| `DEPLOY_KEYFILE` | SSH private key path | Yes |
| `DEPLOY_USER` | SSH username | Yes |
| `DEPLOY_COMMAND` | Command to execute on remote (e.g., `systemctl restart sing-box`) | Yes |
| `DEPLOY_CWD` | Working directory on remote | Yes |

## Usage

### Running the Bot and Config Server

The bot and HTTP config server run in a single process:

```bash
cargo run --bin bot
```

The HTTP server serves client configurations at `/:uuid` endpoint on the port specified by `SING_BOX_SERVER_PORT` (default: 443).

Users access their config via: `http://<server>:<port>/<uuid>`

Or in release mode:

```bash
cargo run --release --bin bot
```

### Generating Client Config (Offline)

Generate a client config file locally:

```bash
cargo run --bin gen_client_config
```

The config is written to `config/sing-box.client.json`.

### Generating Server Config

This generates a sing-box server config with all accepted users:

```bash
cargo run --bin gen_node_config
```

The config is written to `config/sing-box.server.json`.

### Deploying to Remote Servers

Deploys the generated config to multiple remote servers in parallel and restarts sing-box:

```bash
cargo run --bin deploy
```

**Multi-Server Configuration:**

Set `DEPLOY_SERVERS` in your `.env` file with comma-separated server definitions:

```bash
# Format: name:user:host:port
DEPLOY_SERVERS="server1:root:192.168.1.10:22,server2:root:192.168.1.11:22,server3:admin:10.0.0.5:22"
```

The deploy utility will:
1. Connect to all servers in parallel via SSH
2. Execute the deploy command on each server
3. Display individual results and a summary

### Complete Workflow

1. **Start the bot**: Users can register via Telegram (HTTP server also starts)
2. **Admin approves users**: Admin clicks "Accept" in Telegram
3. **Generate server config**: Run `cargo run --bin gen_node_config` to create server config
4. **Deploy**: Run `cargo run --bin deploy` to upload config and restart sing-box on the server

## Database Schema

### user table

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Telegram user ID (primary key) |
| username | TEXT | Telegram username |
| status | INTEGER | 0=New, 1=Accepted, 2=Rejected |
| created_at | TEXT | ISO 8601 timestamp |

### vless_identity table

| Column | Type | Description |
|--------|------|-------------|
| uuid | TEXT | VLESS identity UUID (primary key) |
| user_id | INTEGER | Foreign key to user.id |

## Architecture

This project follows the **Ports and Adapters** (Hexagonal) architecture:

- **Ports**: Define interfaces for external interactions (database)
- **Adapters**: Implement the ports with concrete technologies (Diesel + SQLite)
- **Services**: Contain business logic, depend only on ports

## License

ISC
