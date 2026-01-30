#!/bin/bash
# SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Thread Edge Deployment Script
# Automated deployment to Cloudflare Workers

set -euo pipefail

# Configuration
ENVIRONMENT="${ENVIRONMENT:-production}"
WASM_BUILD="${WASM_BUILD:-release}"
WRANGLER_VERSION="${WRANGLER_VERSION:-3}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check for required tools
    for tool in cargo rustc npm; do
        if ! command -v "$tool" &> /dev/null; then
            log_error "Required tool not found: $tool"
            exit 1
        fi
    done

    # Check for wasm32 target
    if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
        log_info "Installing wasm32-unknown-unknown target..."
        rustup target add wasm32-unknown-unknown
    fi

    # Check for wrangler
    if ! command -v wrangler &> /dev/null; then
        log_info "Installing wrangler..."
        npm install -g wrangler@${WRANGLER_VERSION}
    fi

    log_info "Prerequisites check passed"
}

check_environment_variables() {
    log_info "Checking environment variables..."

    local missing_vars=()

    if [ -z "${CLOUDFLARE_API_TOKEN:-}" ]; then
        missing_vars+=("CLOUDFLARE_API_TOKEN")
    fi

    if [ -z "${CLOUDFLARE_ACCOUNT_ID:-}" ]; then
        missing_vars+=("CLOUDFLARE_ACCOUNT_ID")
    fi

    if [ ${#missing_vars[@]} -gt 0 ]; then
        log_error "Missing required environment variables: ${missing_vars[*]}"
        log_error "Set them with: export CLOUDFLARE_API_TOKEN=your_token"
        exit 1
    fi

    log_info "Environment variables verified"
}

build_wasm() {
    log_step "Building WASM for Edge deployment..."

    if [ "$WASM_BUILD" = "release" ]; then
        log_info "Building optimized release WASM..."
        cargo run -p xtask build-wasm --release
    else
        log_info "Building development WASM..."
        cargo run -p xtask build-wasm
    fi

    # Verify WASM files exist
    if [ ! -f "thread_wasm_bg.wasm" ]; then
        log_error "WASM build failed - thread_wasm_bg.wasm not found"
        exit 1
    fi

    log_info "WASM build completed successfully"
}

run_tests() {
    log_step "Running pre-deployment tests..."

    # Run WASM-specific tests
    log_info "Testing WASM module..."
    cargo test -p thread-wasm --target wasm32-unknown-unknown

    log_info "Tests passed"
}

configure_wrangler() {
    log_step "Configuring Cloudflare Workers..."

    # Verify wrangler.toml exists
    if [ ! -f "wrangler.toml" ]; then
        log_error "wrangler.toml not found in current directory"
        exit 1
    fi

    # Validate wrangler configuration
    log_info "Validating wrangler configuration..."
    if ! wrangler deploy --dry-run --env "$ENVIRONMENT"; then
        log_error "Wrangler configuration validation failed"
        exit 1
    fi

    log_info "Wrangler configuration validated"
}

deploy_to_edge() {
    log_step "Deploying to Cloudflare Edge ($ENVIRONMENT)..."

    # Deploy with wrangler
    if wrangler deploy --env "$ENVIRONMENT"; then
        log_info "Deployment successful"
    else
        log_error "Deployment failed"
        exit 1
    fi
}

run_smoke_tests() {
    log_step "Running smoke tests..."

    # Get deployment URL
    local deployment_url
    if [ "$ENVIRONMENT" = "production" ]; then
        deployment_url="https://thread.knit.li"
    else
        deployment_url="https://thread-${ENVIRONMENT}.knit.li"
    fi

    log_info "Testing endpoint: $deployment_url"

    # Health check
    if curl -f -s "${deployment_url}/health" > /dev/null; then
        log_info "Health check passed"
    else
        log_warn "Health check failed - endpoint may still be propagating"
    fi
}

show_deployment_info() {
    log_step "Deployment Information"

    # Get worker info
    wrangler deployments list --env "$ENVIRONMENT" | head -10

    cat <<EOF

${GREEN}========================================
Thread Edge Deployment Complete
========================================${NC}

Environment: $ENVIRONMENT
Build Type: $WASM_BUILD

Deployment URL:
EOF

    if [ "$ENVIRONMENT" = "production" ]; then
        echo "  https://thread.knit.li"
    else
        echo "  https://thread-${ENVIRONMENT}.knit.li"
    fi

    cat <<EOF

Next Steps:
1. Test the deployment: curl https://thread.knit.li/health
2. View logs: wrangler tail --env $ENVIRONMENT
3. Check metrics: Visit Cloudflare Workers dashboard
4. Rollback if needed: wrangler rollback --env $ENVIRONMENT

EOF
}

rollback() {
    log_warn "Rolling back deployment..."
    wrangler rollback --env "$ENVIRONMENT"
    log_info "Rollback completed"
}

main() {
    log_info "Starting Thread Edge deployment to $ENVIRONMENT..."

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --rollback)
                rollback
                exit 0
                ;;
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --dev)
                WASM_BUILD=dev
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    check_prerequisites
    check_environment_variables
    build_wasm

    if [ "${SKIP_TESTS:-false}" != "true" ]; then
        run_tests
    fi

    configure_wrangler
    deploy_to_edge
    run_smoke_tests
    show_deployment_info

    log_info "Edge deployment completed successfully!"
}

# Run main function
main "$@"
