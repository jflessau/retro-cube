# Message Server

A simple Rust web server that serves messages with basic authentication.

## Features

- **GET `/`** - Returns the current message as plain text (requires basic auth)
- **GET `/form`** - Shows a web form to update the message (requires basic auth)
- **POST `/form`** - Updates the message (validates credentials in form)

## Setup

1. Install Rust if you haven't already: https://rustup.rs

2. Copy `.env.example` to `.env` and set your credentials:
   ```bash
   cp .env.example .env
   ```

3. Edit `.env` and set your username and password:
   ```
   AUTH_USERNAME=your_username
   AUTH_PASSWORD=your_secure_password
   ```

## Running

```bash
cargo run
```

The server will start on `http://0.0.0.0:3000`

## Usage

### Get Message (API endpoint)

```bash
curl -u username:password http://localhost:3000/
```

This endpoint is designed for the Pi Zero to fetch the message.

### Update Message (Web Form)

1. Navigate to `http://localhost:3000/form` in your browser
2. Enter your username and password
3. Type your message (max 144 characters)
4. Click "Update Message"

The message will be saved to `message.txt` in the same directory as the server.

## Environment Variables

- `AUTH_USERNAME` - Username for basic authentication (default: `admin`)
- `AUTH_PASSWORD` - Password for basic authentication (default: `password`)

## Production Deployment

For production, you should:

1. Use a reverse proxy like nginx with HTTPS
2. Set strong credentials in environment variables
3. Consider using systemd or similar to run the server as a service

Example systemd service file (`/etc/systemd/system/message-server.service`):

```ini
[Unit]
Description=Message Server
After=network.target

[Service]
Type=simple
User=youruser
WorkingDirectory=/path/to/retro-cube/server
Environment="AUTH_USERNAME=your_username"
Environment="AUTH_PASSWORD=your_secure_password"
ExecStart=/path/to/retro-cube/server/target/release/server
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Build for release:
```bash
cargo build --release
```
