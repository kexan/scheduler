# Deployment Guide

## Quick Start

### Local Development
```bash
git clone https://github.com/kexan/scheduler
cd scheduler
cargo run
```
Open http://localhost:3030

### Production Server

#### Build and Deploy
```bash
git clone https://github.com/kexan/scheduler
cd scheduler

cargo build --release

sudo mkdir -p /opt/scheduler
sudo cp target/release/migration-scheduler /opt/scheduler/
```

#### Create Systemd Service
```bash
sudo tee /etc/systemd/system/migration-scheduler.service > /dev/null <<EOF
[Unit]
Description=Migration Scheduler Service
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/scheduler
ExecStart=/opt/scheduler/migration-scheduler
Restart=always
RestartSec=10
Environment=RUST_LOG=info
Environment=PORT=3030
Environment=ADMIN_PASSWORD="super_pass"

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable migration-scheduler
sudo systemctl start migration-scheduler

sudo systemctl status migration-scheduler
```

#### Environment Variables

| Variable         | Default    | Description                  |
|------------------|------------|------------------------------|
| `PORT`           | `3030`     | Port to listen on            |
| `ADMIN_PASSWORD` | `admin123` | Password for the admin panel |
| `RUST_LOG`       | `info`     | Log level                    |

> The application binds to `127.0.0.1` and is intended to run behind a reverse proxy (nginx, caddy, etc.).
