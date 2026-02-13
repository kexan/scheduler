# Deployment Guide

## Quick Start

### Option 1: Local Development
```bash
git clone https://github.com/kexan/scheduler
cd scheduler
cargo run --release server
```
Open http://localhost:3030

### Option 2: Production Server

#### Build and Deploy
```bash
# Clone repository
git clone https://github.com/kexan/scheduler
cd scheduler

# Build release binary
cargo build --release

# Create deployment directory
sudo mkdir -p /opt/scheduler
sudo cp target/release/migration-scheduler /opt/scheduler/
```

#### Create Systemd Service
```bash
# Create service file
sudo tee /etc/systemd/system/migration-scheduler.service > /dev/null <<EOF
[Unit]
Description=Migration Scheduler Service
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/scheduler
ExecStart=/opt/scheduler/migration-scheduler server
Restart=always
RestartSec=10
Environment=RUST_LOG=debug
Environment=PORT=3030
Environment=ADMIN_PASSWORD="super_pass"

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and start service
sudo systemctl daemon-reload
sudo systemctl enable migration-scheduler
sudo systemctl start migration-scheduler

# Check status
sudo systemctl status migration-scheduler
```
