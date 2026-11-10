#!/bin/bash
set -euo pipefail
exec > >(tee -a /var/log/todo-bootstrap.log) 2>&1
systemctl enable --now docker
command -v docker
docker compose version
install -d -m 755 /opt/todo-runtime/server
printf '%s' '${config_base64}' | base64 --decode > /opt/todo-runtime/server/config.json
chmod 644 /opt/todo-runtime/server/config.json
cat > /opt/todo-runtime/compose.yaml <<'COMPOSE'
services:
  server:
    image: ${server_image}
    restart: unless-stopped
    volumes:
      - /opt/todo-runtime/server/config.json:/app/config.json:ro
    ports:
      - "127.0.0.1:8080:8080"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 15s
      timeout: 10s
      retries: 10
      start_period: 60s
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"
  client:
    image: ${client_image}
    restart: unless-stopped
    ports:
      - "80:80"
    depends_on:
      server:
        condition: service_healthy
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"
COMPOSE
cat > /etc/systemd/system/todo-rust-stack.service <<'UNIT'
[Unit]
Description=Todo Rust Docker Compose stack
Requires=docker.service
After=docker.service network-online.target
Wants=network-online.target
[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/todo-runtime
ExecStart=/usr/bin/docker compose -p todo-rust up -d --remove-orphans --wait --wait-timeout 480
ExecStop=/usr/bin/docker compose -p todo-rust stop
TimeoutStartSec=600
Restart=on-failure
RestartSec=30
[Install]
WantedBy=multi-user.target
UNIT
systemctl daemon-reload
systemctl enable --now todo-rust-stack.service
