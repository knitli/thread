#!/bin/bash
# SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Thread CLI Deployment Script
# Automated deployment of Thread CLI to production servers

set -euo pipefail

# Configuration
VERSION="${VERSION:-latest}"
TARGET_ARCH="${TARGET_ARCH:-x86_64-unknown-linux-gnu}"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
SERVICE_USER="${SERVICE_USER:-thread}"
SYSTEMD_SERVICE="${SYSTEMD_SERVICE:-thread}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check if running as root
    if [ "$EUID" -ne 0 ]; then
        log_error "This script must be run as root"
        exit 1
    fi

    # Check required commands
    for cmd in curl tar systemctl; do
        if ! command -v "$cmd" &> /dev/null; then
            log_error "Required command not found: $cmd"
            exit 1
        fi
    done

    log_info "Prerequisites check passed"
}

get_latest_version() {
    if [ "$VERSION" = "latest" ]; then
        log_info "Fetching latest version..."
        VERSION=$(curl -s https://api.github.com/repos/knitli/thread/releases/latest | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')
        log_info "Latest version: $VERSION"
    fi
}

download_binary() {
    log_info "Downloading Thread CLI $VERSION for $TARGET_ARCH..."

    local download_url="https://github.com/knitli/thread/releases/download/v${VERSION}/thread-${VERSION}-${TARGET_ARCH}.tar.gz"
    local temp_dir=$(mktemp -d)
    local archive_path="${temp_dir}/thread.tar.gz"

    if ! curl -L -o "$archive_path" "$download_url"; then
        log_error "Failed to download binary"
        rm -rf "$temp_dir"
        exit 1
    fi

    log_info "Extracting archive..."
    tar -xzf "$archive_path" -C "$temp_dir"

    echo "$temp_dir"
}

install_binary() {
    local temp_dir=$1
    local binary_path="${temp_dir}/thread"

    log_info "Installing binary to $INSTALL_DIR..."

    # Backup existing binary if present
    if [ -f "${INSTALL_DIR}/thread" ]; then
        log_warn "Backing up existing binary..."
        cp "${INSTALL_DIR}/thread" "${INSTALL_DIR}/thread.backup.$(date +%Y%m%d%H%M%S)"
    fi

    # Install new binary
    cp "$binary_path" "${INSTALL_DIR}/thread"
    chmod +x "${INSTALL_DIR}/thread"

    # Verify installation
    if "${INSTALL_DIR}/thread" --version; then
        log_info "Binary installed successfully"
    else
        log_error "Binary installation verification failed"
        exit 1
    fi
}

create_service_user() {
    if ! id "$SERVICE_USER" &>/dev/null; then
        log_info "Creating service user: $SERVICE_USER"
        useradd --system --no-create-home --shell /bin/false "$SERVICE_USER"
    else
        log_info "Service user already exists: $SERVICE_USER"
    fi
}

setup_systemd_service() {
    log_info "Setting up systemd service..."

    cat > "/etc/systemd/system/${SYSTEMD_SERVICE}.service" <<EOF
[Unit]
Description=Thread Code Analysis Service
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=${SERVICE_USER}
Group=${SERVICE_USER}
WorkingDirectory=/var/lib/thread
ExecStart=${INSTALL_DIR}/thread serve
Restart=on-failure
RestartSec=10

# Environment
Environment="RUST_LOG=info"
Environment="DATABASE_URL=postgresql://thread:thread@localhost:5432/thread"

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/thread

# Resource limits
LimitNOFILE=65536
MemoryMax=2G

[Install]
WantedBy=multi-user.target
EOF

    # Create working directory
    mkdir -p /var/lib/thread
    chown "$SERVICE_USER:$SERVICE_USER" /var/lib/thread

    # Reload systemd
    systemctl daemon-reload

    log_info "Systemd service configured"
}

configure_database() {
    log_info "Checking database configuration..."

    # Check if PostgreSQL is running
    if ! systemctl is-active --quiet postgresql; then
        log_warn "PostgreSQL is not running. Skipping database setup."
        log_warn "Please configure DATABASE_URL in /etc/systemd/system/${SYSTEMD_SERVICE}.service"
        return
    fi

    log_info "Database configuration should be done manually"
    log_info "Run: sudo -u postgres psql"
    log_info "Then: CREATE DATABASE thread; CREATE USER thread WITH PASSWORD 'your_password';"
}

enable_service() {
    log_info "Enabling and starting service..."

    systemctl enable "${SYSTEMD_SERVICE}.service"
    systemctl start "${SYSTEMD_SERVICE}.service"

    # Wait for service to start
    sleep 2

    if systemctl is-active --quiet "${SYSTEMD_SERVICE}.service"; then
        log_info "Service started successfully"
    else
        log_error "Service failed to start. Check logs with: journalctl -u ${SYSTEMD_SERVICE}.service"
        exit 1
    fi
}

health_check() {
    log_info "Performing health check..."

    # Check if binary responds
    if "${INSTALL_DIR}/thread" --version > /dev/null 2>&1; then
        log_info "Health check passed"
    else
        log_error "Health check failed"
        exit 1
    fi
}

cleanup() {
    local temp_dir=$1
    log_info "Cleaning up temporary files..."
    rm -rf "$temp_dir"
}

show_summary() {
    cat <<EOF

${GREEN}========================================
Thread CLI Deployment Complete
========================================${NC}

Version: $VERSION
Install Location: ${INSTALL_DIR}/thread
Service: ${SYSTEMD_SERVICE}.service

Next Steps:
1. Configure database connection in /etc/systemd/system/${SYSTEMD_SERVICE}.service
2. Check service status: systemctl status ${SYSTEMD_SERVICE}.service
3. View logs: journalctl -fu ${SYSTEMD_SERVICE}.service
4. Test binary: ${INSTALL_DIR}/thread --version

EOF
}

main() {
    log_info "Starting Thread CLI deployment..."

    check_prerequisites
    get_latest_version

    local temp_dir=$(download_binary)

    install_binary "$temp_dir"
    create_service_user
    setup_systemd_service
    configure_database
    enable_service
    health_check
    cleanup "$temp_dir"

    show_summary

    log_info "Deployment completed successfully!"
}

# Run main function
main "$@"
