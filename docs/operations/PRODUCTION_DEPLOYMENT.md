# Production Deployment Strategies

**Version**: 1.0.0
**Last Updated**: 2026-01-28
**Status**: Production Ready

---

## Overview

This document defines production deployment strategies for Thread across CLI and Edge environments. It covers deployment patterns, risk mitigation, validation procedures, and integration with CI/CD infrastructure.

### Purpose

- **Safe Deployments**: Minimize risk during production updates
- **Zero Downtime**: Maintain service availability during deployments
- **Quick Rollback**: Enable rapid recovery from deployment failures
- **Gradual Rollout**: Test changes with subset of traffic before full deployment

### Integration Points

- **Day 21 CI/CD**: Automated testing and validation pipelines
- **Day 22 Security**: Security scanning and vulnerability checks
- **Day 24 Capacity**: Resource planning and scaling automation
- **Day 24 Load Balancing**: Traffic routing and health checking

---

## Deployment Strategy Overview

### Strategy Comparison

| Strategy | Downtime | Risk | Rollback Speed | Resource Cost | Complexity |
|----------|----------|------|----------------|---------------|------------|
| **Recreate** | Yes (minutes) | High | Fast (seconds) | Low (1×) | Low |
| **Rolling** | No | Medium | Medium (minutes) | Low (1×) | Medium |
| **Blue-Green** | No | Low | Instant (seconds) | High (2×) | Medium |
| **Canary** | No | Very Low | Instant (seconds) | Medium (1.1-1.5×) | High |
| **A/B Testing** | No | Very Low | Instant (per cohort) | Medium (1.5×) | Very High |

### Strategy Selection Criteria

**Use Recreate When**:
- Development or staging environments
- Downtime acceptable (maintenance windows)
- Simplicity prioritized over availability
- Cost extremely sensitive

**Use Rolling When**:
- Zero downtime required
- Standard risk tolerance acceptable
- Resource cost must be minimized
- Kubernetes or similar orchestration available

**Use Blue-Green When**:
- Zero downtime critical
- Instant rollback required
- Can afford 2× resource cost
- Database migrations are backward-compatible

**Use Canary When**:
- High-risk deployments (major changes)
- Want gradual traffic increase (1% → 100%)
- Can monitor detailed metrics per version
- Production testing before full rollout

**Use A/B Testing When**:
- Testing feature variants
- Need statistical significance for decision
- Long-running experiments (days/weeks)
- Different user cohorts need different behavior

---

## Deployment Strategies

### Strategy 1: Recreate (Simple Replace)

**Description**: Stop all old instances, deploy new instances.

**Architecture**:
```
Step 1: Running v1.0 (100% traffic)
├─ Instance 1 (v1.0) ──┐
├─ Instance 2 (v1.0) ──┼─→ Load Balancer → Users
└─ Instance 3 (v1.0) ──┘

Step 2: Stop all v1.0 instances
(Downtime: 1-5 minutes)

Step 3: Running v1.1 (100% traffic)
├─ Instance 1 (v1.1) ──┐
├─ Instance 2 (v1.1) ──┼─→ Load Balancer → Users
└─ Instance 3 (v1.1) ──┘
```

**Characteristics**:
- **Downtime**: Yes (1-5 minutes typical)
- **Rollback**: Fast (redeploy v1.0, same downtime)
- **Resource Cost**: 1× (no extra resources)
- **Complexity**: Low (simplest strategy)

**Implementation** (Kubernetes):
```yaml
# Recreate deployment strategy
apiVersion: apps/v1
kind: Deployment
metadata:
  name: thread-worker
spec:
  replicas: 3
  strategy:
    type: Recreate  # Kills all pods before creating new ones
  template:
    spec:
      containers:
      - name: thread
        image: thread:v1.1
```

**Implementation** (Bash Script):
```bash
#!/bin/bash
# Recreate deployment script

echo "Stopping all v1.0 instances..."
systemctl stop thread-worker@{1,2,3}

echo "Deploying v1.1 instances..."
# Update binary
cp /tmp/thread-v1.1 /usr/local/bin/thread

echo "Starting v1.1 instances..."
systemctl start thread-worker@{1,2,3}

echo "Deployment complete"
```

**Use Cases**:
- Development environments
- Staging environments with scheduled maintenance
- Non-critical services with acceptable downtime
- Cost-optimized deployments

**Rollback Procedure**:
1. Stop all v1.1 instances
2. Redeploy v1.0 binary/image
3. Start v1.0 instances
4. Verify health checks pass

---

### Strategy 2: Rolling Deployment (Gradual Replace)

**Description**: Replace instances one-by-one or in batches, maintaining service availability.

**Architecture**:
```
Step 1: Running v1.0 (100% traffic)
├─ Instance 1 (v1.0) ──┐
├─ Instance 2 (v1.0) ──┼─→ Load Balancer → Users
└─ Instance 3 (v1.0) ──┘

Step 2: Rolling update starts
├─ Instance 1 (v1.1) ──┐  ← Updated
├─ Instance 2 (v1.0) ──┼─→ Load Balancer → Users
└─ Instance 3 (v1.0) ──┘

Step 3: Continue rolling
├─ Instance 1 (v1.1) ──┐
├─ Instance 2 (v1.1) ──┼─→ Load Balancer → Users  ← Updated
└─ Instance 3 (v1.0) ──┘

Step 4: Rolling complete (100% traffic)
├─ Instance 1 (v1.1) ──┐
├─ Instance 2 (v1.1) ──┼─→ Load Balancer → Users
└─ Instance 3 (v1.1) ──┘  ← Updated
```

**Characteristics**:
- **Downtime**: None
- **Rollback**: Medium speed (reverse rolling update)
- **Resource Cost**: 1× (no extra resources)
- **Complexity**: Medium (orchestration required)

**Implementation** (Kubernetes):
```yaml
# Rolling update deployment strategy
apiVersion: apps/v1
kind: Deployment
metadata:
  name: thread-worker
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1  # At most 1 pod down during update
      maxSurge: 1        # At most 1 extra pod during update
  template:
    spec:
      containers:
      - name: thread
        image: thread:v1.1
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
```

**Implementation** (HAProxy + Systemd):
```bash
#!/bin/bash
# Rolling deployment script

INSTANCES=(1 2 3)

for instance in "${INSTANCES[@]}"; do
    echo "Updating instance $instance..."

    # Disable instance in HAProxy
    echo "disable server thread_workers/worker$instance" | \
        socat stdio /var/run/haproxy.sock

    # Wait for connections to drain
    sleep 10

    # Stop old version
    systemctl stop thread-worker@$instance

    # Update binary
    cp /tmp/thread-v1.1 /usr/local/bin/thread

    # Start new version
    systemctl start thread-worker@$instance

    # Wait for health check
    until curl -f http://localhost:8080/health/ready; do
        sleep 2
    done

    # Re-enable instance in HAProxy
    echo "enable server thread_workers/worker$instance" | \
        socat stdio /var/run/haproxy.sock

    echo "Instance $instance updated successfully"
done

echo "Rolling deployment complete"
```

**Use Cases**:
- Standard production deployments
- Zero downtime requirement
- Limited resources (can't afford 2×)
- Kubernetes or orchestrated environments

**Rollback Procedure**:
1. Initiate reverse rolling update (v1.1 → v1.0)
2. Follow same process: update instances one-by-one
3. Time to rollback: Same as deployment time (minutes)

**Best Practices**:
- Set appropriate `maxUnavailable` (typically 1 or 25%)
- Configure health checks (readiness and liveness)
- Monitor error rates during rollout
- Pause rollout if error rate increases

---

### Strategy 3: Blue-Green Deployment (Full Swap)

**Description**: Run two identical environments (blue and green), switch traffic instantly.

**Architecture**:
```
Step 1: Blue environment active (100% traffic)
Blue Environment (v1.0)
├─ Instance 1 ──┐
├─ Instance 2 ──┼─→ Load Balancer → Users (100% to Blue)
└─ Instance 3 ──┘

Green Environment (idle)
├─ Instance 1 (stopped)
├─ Instance 2 (stopped)
└─ Instance 3 (stopped)

Step 2: Deploy to Green, test privately
Blue Environment (v1.0)
├─ Instance 1 ──┐
├─ Instance 2 ──┼─→ Load Balancer → Users (100% to Blue)
└─ Instance 3 ──┘

Green Environment (v1.1) ← Deploy new version
├─ Instance 1 ──┐
├─ Instance 2 ──┼─→ Internal testing only
└─ Instance 3 ──┘

Step 3: Switch traffic to Green (instant)
Blue Environment (v1.0) ← Kept running for rollback
├─ Instance 1 ──┐
├─ Instance 2 ──┼─→ (Standby)
└─ Instance 3 ──┘

Green Environment (v1.1)
├─ Instance 1 ──┐
├─ Instance 2 ──┼─→ Load Balancer → Users (100% to Green)
└─ Instance 3 ──┘

Step 4: Decommission Blue (after validation)
Green Environment (v1.1)
├─ Instance 1 ──┐
├─ Instance 2 ──┼─→ Load Balancer → Users (100% to Green)
└─ Instance 3 ──┘
```

**Characteristics**:
- **Downtime**: None
- **Rollback**: Instant (switch back to Blue)
- **Resource Cost**: 2× (double infrastructure during deployment)
- **Complexity**: Medium (need duplicate environment)

**Implementation** (Kubernetes with Services):
```yaml
# Blue deployment (current production)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: thread-worker-blue
  labels:
    app: thread
    version: blue
spec:
  replicas: 3
  selector:
    matchLabels:
      app: thread
      version: blue
  template:
    metadata:
      labels:
        app: thread
        version: blue
    spec:
      containers:
      - name: thread
        image: thread:v1.0

---
# Green deployment (new version)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: thread-worker-green
  labels:
    app: thread
    version: green
spec:
  replicas: 3
  selector:
    matchLabels:
      app: thread
      version: green
  template:
    metadata:
      labels:
        app: thread
        version: green
    spec:
      containers:
      - name: thread
        image: thread:v1.1

---
# Service (switch by updating selector)
apiVersion: v1
kind: Service
metadata:
  name: thread-service
spec:
  selector:
    app: thread
    version: blue  # Change to 'green' to switch traffic
  ports:
  - port: 80
    targetPort: 8080
```

**Traffic Switch Script**:
```bash
#!/bin/bash
# Blue-Green traffic switch

CURRENT_ENV="blue"
NEW_ENV="green"

echo "Current traffic: $CURRENT_ENV"
echo "Switching to: $NEW_ENV"

# Update service selector to point to green
kubectl patch service thread-service -p \
  "{\"spec\":{\"selector\":{\"version\":\"$NEW_ENV\"}}}"

echo "Traffic switched to $NEW_ENV"
echo "Monitor for 5-10 minutes, then run cleanup if successful"
```

**Rollback Script**:
```bash
#!/bin/bash
# Instant rollback to blue

kubectl patch service thread-service -p \
  "{\"spec\":{\"selector\":{\"version\":\"blue\"}}}"

echo "Rolled back to blue environment"
```

**Use Cases**:
- High-risk deployments requiring instant rollback
- Database migrations are backward-compatible
- Can afford 2× infrastructure cost
- Critical services with strict SLOs

**Rollback Procedure**:
1. Switch Service selector back to blue (instant)
2. Verify traffic routing to blue
3. Investigate green environment issues
4. Time to rollback: Seconds

**Best Practices**:
- Test green environment with internal traffic first
- Keep blue environment running for 24-48 hours post-deployment
- Validate database compatibility between versions
- Use smoke tests before switching traffic

---

### Strategy 4: Canary Deployment (Gradual Rollout)

**Description**: Deploy new version to small subset of instances, gradually increase traffic.

**Architecture**:
```
Step 1: Baseline (100% traffic to v1.0)
v1.0 (Stable)
├─ Instance 1 ──┐
├─ Instance 2 ──┼─→ Load Balancer → Users (100% to v1.0)
└─ Instance 3 ──┘

Step 2: Deploy canary (5% traffic to v1.1)
v1.0 (Stable)
├─ Instance 1 ──┐
└─ Instance 2 ──┼─→ Load Balancer → Users (95% to v1.0)
                │
v1.1 (Canary)   │
└─ Instance 3 ──┘                      (5% to v1.1)

Step 3: Increase canary (25% traffic)
v1.0 (Stable)
├─ Instance 1 ──┐
└─ Instance 2 ──┼─→ Load Balancer → Users (75% to v1.0)
                │
v1.1 (Canary)   │
└─ Instance 3 ──┘                      (25% to v1.1)

Step 4: Increase canary (50% traffic)
v1.0 (Stable)
└─ Instance 1 ──┬─→ Load Balancer → Users (50% to v1.0)
                │
v1.1 (Canary)   │
├─ Instance 2 ──┤                      (50% to v1.1)
└─ Instance 3 ──┘

Step 5: Full rollout (100% to v1.1)
v1.1 (Stable)
├─ Instance 1 ──┐
├─ Instance 2 ──┼─→ Load Balancer → Users (100% to v1.1)
└─ Instance 3 ──┘
```

**Characteristics**:
- **Downtime**: None
- **Rollback**: Instant (reduce canary traffic to 0%)
- **Resource Cost**: 1.1-1.5× (small overhead during rollout)
- **Complexity**: High (requires traffic shaping and metrics)

**Implementation** (Kubernetes with Istio):
```yaml
# Virtual Service for canary traffic routing
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: thread-canary
spec:
  hosts:
  - thread-service
  http:
  - match:
    - headers:
        user-agent:
          regex: ".*canary.*"  # Optional: specific users
    route:
    - destination:
        host: thread-service
        subset: v1.1
      weight: 100
  - route:
    - destination:
        host: thread-service
        subset: v1.0
      weight: 95  # 95% to stable
    - destination:
        host: thread-service
        subset: v1.1
      weight: 5   # 5% to canary

---
# Destination Rule for version subsets
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: thread-versions
spec:
  host: thread-service
  subsets:
  - name: v1.0
    labels:
      version: v1.0
  - name: v1.1
    labels:
      version: v1.1
```

**Canary Rollout Script** (Kubernetes + Flagger):
```yaml
# Flagger Canary resource
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: thread-canary
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: thread-worker
  service:
    port: 8080
  analysis:
    interval: 1m
    threshold: 5
    maxWeight: 50
    stepWeight: 10
    metrics:
    - name: request-success-rate
      thresholdRange:
        min: 99
      interval: 1m
    - name: request-duration
      thresholdRange:
        max: 500
      interval: 1m
    webhooks:
    - name: load-test
      url: http://flagger-loadtester/
      timeout: 5s
      metadata:
        type: cmd
        cmd: "hey -z 1m -q 10 -c 2 http://thread-service/"
```

**Manual Canary Rollout Script**:
```bash
#!/bin/bash
# Manual canary rollout with validation

CANARY_WEIGHTS=(5 10 25 50 75 100)

for weight in "${CANARY_WEIGHTS[@]}"; do
    echo "Setting canary traffic to ${weight}%..."

    # Update Istio VirtualService weight
    kubectl patch virtualservice thread-canary --type merge -p \
      "{\"spec\":{\"http\":[{\"route\":[
        {\"destination\":{\"host\":\"thread-service\",\"subset\":\"v1.0\"},\"weight\":$((100-weight))},
        {\"destination\":{\"host\":\"thread-service\",\"subset\":\"v1.1\"},\"weight\":${weight}}
      ]}]}}"

    # Wait for metrics to stabilize
    sleep 300  # 5 minutes

    # Check error rate
    error_rate=$(curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total{status=~\"5..\"}[5m])" | jq -r '.data.result[0].value[1]')

    if (( $(echo "$error_rate > 0.01" | bc -l) )); then
        echo "ERROR: Error rate too high ($error_rate), rolling back..."
        kubectl patch virtualservice thread-canary --type merge -p \
          "{\"spec\":{\"http\":[{\"route\":[
            {\"destination\":{\"host\":\"thread-service\",\"subset\":\"v1.0\"},\"weight\":100},
            {\"destination\":{\"host\":\"thread-service\",\"subset\":\"v1.1\"},\"weight\":0}
          ]}]}}"
        exit 1
    fi

    echo "Canary at ${weight}% healthy, continuing..."
done

echo "Canary rollout complete: 100% traffic to v1.1"
```

**Use Cases**:
- High-risk deployments (major feature changes)
- Want production testing before full rollout
- Need fine-grained traffic control
- Can monitor detailed metrics per version

**Rollback Procedure**:
1. Set canary traffic weight to 0% (instant)
2. Verify all traffic routing to stable version
3. Investigate canary issues offline
4. Time to rollback: Seconds

**Best Practices**:
- Start with small canary weight (1-5%)
- Increase gradually with validation at each step
- Monitor canary-specific metrics (error rate, latency)
- Automate rollback on threshold violations
- Use canary for internal users first (beta testers)

---

### Strategy 5: A/B Testing (Feature Variants)

**Description**: Run multiple versions simultaneously for long-term testing, route traffic by user cohort.

**Architecture**:
```
Users
├─ Cohort A (50%) → Version A (feature disabled)
│  └─ Behavior tracking, conversion metrics
│
└─ Cohort B (50%) → Version B (feature enabled)
   └─ Behavior tracking, conversion metrics

Statistical analysis determines winning variant
```

**Characteristics**:
- **Downtime**: None
- **Rollback**: Instant (route cohort to different version)
- **Resource Cost**: 1.5× (both versions running)
- **Complexity**: Very High (requires cohort management, analytics)

**Implementation** (Istio + Custom Headers):
```yaml
# A/B testing with user cohorts
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: thread-ab-test
spec:
  hosts:
  - thread-service
  http:
  - match:
    - headers:
        x-user-cohort:
          exact: "A"
    route:
    - destination:
        host: thread-service
        subset: variant-a
  - match:
    - headers:
        x-user-cohort:
          exact: "B"
    route:
    - destination:
        host: thread-service
        subset: variant-b
  - route:  # Default: random 50/50
    - destination:
        host: thread-service
        subset: variant-a
      weight: 50
    - destination:
        host: thread-service
        subset: variant-b
      weight: 50
```

**Use Cases**:
- Testing feature variants (UI changes, algorithm changes)
- Need statistical significance for product decisions
- Long-running experiments (days to weeks)
- Different user cohorts require different behavior

**Best Practices**:
- Define success metrics before experiment
- Calculate required sample size for statistical significance
- Run experiment for sufficient duration (typically 7-14 days)
- Ensure user experience consistency (sticky cohorts)
- Track both primary and secondary metrics

---

## CLI Deployment Implementation

### Single-Node CLI Deployment

**Recreate Strategy** (Simplest):
```bash
#!/bin/bash
# Single-node recreate deployment

# Stop old version
systemctl stop thread-worker

# Backup old binary
cp /usr/local/bin/thread /usr/local/bin/thread.backup

# Deploy new binary
cp /tmp/thread-new /usr/local/bin/thread
chmod +x /usr/local/bin/thread

# Start new version
systemctl start thread-worker

# Verify health
until curl -f http://localhost:8080/health/ready; do
    sleep 2
done

echo "Deployment complete"
```

**Rollback**:
```bash
#!/bin/bash
# Rollback to previous version

systemctl stop thread-worker
cp /usr/local/bin/thread.backup /usr/local/bin/thread
systemctl start thread-worker
```

### Multi-Node CLI Deployment

**Rolling Strategy** (Zero Downtime):
```bash
#!/bin/bash
# Multi-node rolling deployment

NODES=("node1.example.com" "node2.example.com" "node3.example.com")

for node in "${NODES[@]}"; do
    echo "Deploying to $node..."

    # Disable node in load balancer
    ssh lb.example.com "echo 'disable server thread_workers/$node' | socat stdio /var/run/haproxy.sock"

    # Wait for connections to drain
    sleep 10

    # Deploy new version
    ssh "$node" "systemctl stop thread-worker && \
                 cp /tmp/thread-new /usr/local/bin/thread && \
                 chmod +x /usr/local/bin/thread && \
                 systemctl start thread-worker"

    # Wait for health check
    until ssh "$node" "curl -f http://localhost:8080/health/ready"; do
        sleep 2
    done

    # Re-enable node in load balancer
    ssh lb.example.com "echo 'enable server thread_workers/$node' | socat stdio /var/run/haproxy.sock"

    echo "$node deployed successfully"
done

echo "Rolling deployment complete"
```

**Blue-Green Strategy**:
```bash
#!/bin/bash
# Blue-Green deployment for CLI cluster

BLUE_NODES=("blue1" "blue2" "blue3")
GREEN_NODES=("green1" "green2" "green3")

echo "Deploying to green environment..."

for node in "${GREEN_NODES[@]}"; do
    ssh "$node" "systemctl stop thread-worker && \
                 cp /tmp/thread-new /usr/local/bin/thread && \
                 chmod +x /usr/local/bin/thread && \
                 systemctl start thread-worker"
done

echo "Green environment deployed, testing..."

# Smoke test green environment
for node in "${GREEN_NODES[@]}"; do
    curl -f "http://$node:8080/health/ready" || {
        echo "Green environment unhealthy, aborting"
        exit 1
    }
done

echo "Green environment healthy, switching traffic..."

# Update HAProxy to point to green
ssh lb.example.com "cat > /etc/haproxy/haproxy.cfg <<EOF
backend thread_workers
    balance leastconn
    server green1 green1:8080 check
    server green2 green2:8080 check
    server green3 green3:8080 check
EOF
systemctl reload haproxy"

echo "Traffic switched to green environment"
echo "Monitor for issues, then decommission blue with:"
echo "for node in ${BLUE_NODES[@]}; do ssh \$node systemctl stop thread-worker; done"
```

---

## Edge Deployment Implementation

### Cloudflare Workers Deployment

**Default Strategy** (Instant Switch):
```bash
#!/bin/bash
# Cloudflare Workers deployment (instant switch)

# Build new WASM
cargo run -p xtask build-wasm --release

# Deploy to Cloudflare (atomic switch)
wrangler deploy

echo "Edge deployment complete (instant switch)"
```

**Gradual Rollout with Cloudflare Gradual Deployments**:
```bash
#!/bin/bash
# Gradual rollout on Cloudflare Workers

# Deploy with gradual rollout (10% increments over 10 minutes)
wrangler deployments gradual \
    --percentage 10 \
    --interval 60s

echo "Gradual rollout started: 10% every 60 seconds"
echo "Monitor at: https://dash.cloudflare.com/deployments"
```

**Canary with Cloudflare Workers**:
```toml
# wrangler.toml - Environment-based canary

name = "thread-worker"

[env.production]
route = "thread.example.com/*"
vars = { ENVIRONMENT = "production" }

[env.canary]
route = "thread.example.com/*"
vars = { ENVIRONMENT = "canary" }
# Route 5% of traffic to canary via Load Balancer
```

```bash
# Deploy canary
wrangler deploy --env canary

# Update Cloudflare Load Balancer to send 5% traffic to canary
# (via dashboard or API)

# Monitor canary metrics...

# If successful, deploy to production
wrangler deploy --env production
```

---

## Deployment Validation and Smoke Tests

### Pre-Deployment Validation

**Checklist**:
- [ ] All tests pass (unit, integration, E2E)
- [ ] Security scans complete (no critical vulnerabilities)
- [ ] Performance benchmarks meet SLOs
- [ ] Database migrations tested and backward-compatible
- [ ] Secrets and configuration validated
- [ ] Rollback plan documented

**Automated Pre-Deployment Script**:
```bash
#!/bin/bash
# Pre-deployment validation

set -e

echo "Running pre-deployment validation..."

# 1. Run tests
echo "Running tests..."
cargo nextest run --all-features --no-fail-fast

# 2. Security scan
echo "Running security scan..."
cargo audit

# 3. Performance benchmark
echo "Running performance benchmarks..."
cargo bench --bench fingerprint_benchmark -- --test

# 4. Database migration validation
echo "Validating database migrations..."
diesel migration run --database-url="$TEST_DATABASE_URL"
diesel migration redo --database-url="$TEST_DATABASE_URL"

# 5. Configuration validation
echo "Validating configuration..."
./scripts/validate-config.sh production

echo "Pre-deployment validation complete: PASSED"
```

### Post-Deployment Smoke Tests

**Critical Path Tests**:
```bash
#!/bin/bash
# Post-deployment smoke tests

BASE_URL="${1:-https://thread.example.com}"

echo "Running smoke tests against $BASE_URL..."

# 1. Health check
echo "Testing health endpoint..."
curl -f "$BASE_URL/health" || {
    echo "Health check failed"
    exit 1
}

# 2. Basic analysis
echo "Testing basic analysis..."
response=$(curl -s -X POST "$BASE_URL/api/analyze" \
    -H "Content-Type: application/json" \
    -d '{"code":"function test() { return 42; }"}')

if ! echo "$response" | jq -e '.fingerprint' > /dev/null; then
    echo "Analysis failed: $response"
    exit 1
fi

# 3. Cache hit
echo "Testing cache hit..."
response2=$(curl -s -X POST "$BASE_URL/api/analyze" \
    -H "Content-Type: application/json" \
    -d '{"code":"function test() { return 42; }"}')

cache_status=$(echo "$response2" | jq -r '.cache_status')
if [[ "$cache_status" != "hit" ]]; then
    echo "Cache miss on second request (expected hit)"
    exit 1
fi

# 4. Performance check
echo "Testing performance (latency)..."
latency=$(curl -o /dev/null -s -w '%{time_total}' "$BASE_URL/health")
if (( $(echo "$latency > 0.5" | bc -l) )); then
    echo "High latency: ${latency}s"
    exit 1
fi

echo "Smoke tests complete: PASSED"
```

### Continuous Validation During Rollout

**Metrics to Monitor**:
- Error rate (should remain < 1%)
- Latency p95 (should remain < 50 ms)
- Cache hit rate (should remain > 90%)
- Throughput (should not drop > 10%)

**Automated Rollout Validation**:
```bash
#!/bin/bash
# Continuous validation during canary rollout

PROMETHEUS_URL="${PROMETHEUS_URL:-http://localhost:9090}"

check_metrics() {
    # Query error rate
    error_rate=$(curl -s -G \
        --data-urlencode 'query=rate(http_requests_total{status=~"5.."}[5m])' \
        "$PROMETHEUS_URL/api/v1/query" | jq -r '.data.result[0].value[1]')

    # Query latency p95
    latency_p95=$(curl -s -G \
        --data-urlencode 'query=histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))' \
        "$PROMETHEUS_URL/api/v1/query" | jq -r '.data.result[0].value[1]')

    # Validate thresholds
    if (( $(echo "$error_rate > 0.01" | bc -l) )); then
        echo "ERROR: Error rate too high: $error_rate"
        return 1
    fi

    if (( $(echo "$latency_p95 > 0.05" | bc -l) )); then
        echo "ERROR: Latency p95 too high: ${latency_p95}s"
        return 1
    fi

    echo "Metrics healthy: error_rate=$error_rate, latency_p95=$latency_p95"
    return 0
}

# Monitor metrics every 30 seconds during rollout
while true; do
    if ! check_metrics; then
        echo "Metrics unhealthy, triggering rollback..."
        ./scripts/rollback.sh
        exit 1
    fi
    sleep 30
done
```

---

## Risk Mitigation

### Database Migration Safety

**Backward-Compatible Migrations**:
```sql
-- SAFE: Add nullable column (backward-compatible)
ALTER TABLE cache ADD COLUMN metadata JSONB;

-- SAFE: Add index (doesn't affect queries)
CREATE INDEX CONCURRENTLY idx_fingerprint ON cache(fingerprint);

-- UNSAFE: Drop column (breaks old code)
-- ALTER TABLE cache DROP COLUMN old_field;  -- DON'T DO THIS

-- SAFE: Deprecate column (keep for 2+ releases)
-- 1. Release v1.1: Stop writing to old_field, add new_field
-- 2. Release v1.2: Migrate data old_field → new_field
-- 3. Release v1.3: Drop old_field (after v1.1 fully rolled out)
```

**Migration Rollback**:
```bash
#!/bin/bash
# Database migration rollback script

echo "Rolling back database migration..."

# Diesel rollback (CLI)
diesel migration revert --database-url="$DATABASE_URL"

# Or manual SQL
psql "$DATABASE_URL" <<EOF
-- Undo migration manually
DROP INDEX idx_fingerprint;
ALTER TABLE cache DROP COLUMN metadata;
EOF

echo "Database migration rolled back"
```

### Feature Flags for Risk Mitigation

**Use Feature Flags** to decouple deployment from feature activation:

```rust
// Feature flag in code
if feature_enabled("new_analysis_algorithm") {
    new_analysis_implementation(code)
} else {
    old_analysis_implementation(code)
}
```

**Benefits**:
- Deploy code but keep feature disabled
- Enable feature for small % of users first (canary)
- Instant rollback (just disable feature, no redeployment)
- A/B test features without separate deployments

### Circuit Breaker Pattern

**Protect Against Cascading Failures**:

```rust
use failsafe::{Config, CircuitBreaker, Error};

let circuit_breaker = Config::new()
    .failure_rate_threshold(50.0)  // Open if 50% failures
    .wait_duration_in_open_state(Duration::from_secs(60))
    .build();

// Protected call
let result = circuit_breaker.call(|| {
    // Call to potentially failing service
    external_api_call()
});

match result {
    Ok(data) => handle_success(data),
    Err(Error::Rejected) => {
        // Circuit open, use fallback
        use_cached_data()
    }
    Err(e) => handle_error(e),
}
```

---

## Best Practices

### 1. Always Have a Rollback Plan

**Antipattern**: Deploy without documented rollback procedure

**Best Practice**: Document and test rollback before deployment

**Rollback Decision Criteria**:
- Error rate > 1% (immediate rollback)
- Latency p95 > 2× baseline (immediate rollback)
- User reports of critical issues (evaluate and rollback if severe)
- Database corruption detected (immediate rollback)

### 2. Deploy During Low-Traffic Hours

**Antipattern**: Deploy during peak traffic (highest risk)

**Best Practice**: Deploy during maintenance windows or low-traffic periods

**Optimal Deployment Windows**:
- Weekdays: 2 AM - 6 AM (local time)
- Avoid: Monday mornings, Friday afternoons
- Best: Tuesday-Thursday early morning

### 3. Monitor Closely During and After Deployment

**Antipattern**: Deploy and walk away

**Best Practice**: Active monitoring for 30-60 minutes post-deployment

**Monitoring Checklist**:
- [ ] Error rate dashboards (first 15 minutes)
- [ ] Latency graphs (first 30 minutes)
- [ ] Cache hit rate (first 30 minutes)
- [ ] User-facing metrics (session length, conversion)
- [ ] System resources (CPU, memory, disk)

### 4. Gradual Rollout for High-Risk Changes

**Antipattern**: Deploy major changes to 100% of users immediately

**Best Practice**: Use canary or blue-green for major changes

**Risk Assessment**:
- **Low Risk**: Bug fixes, minor improvements → Rolling deployment
- **Medium Risk**: New features, refactoring → Canary (5% → 100%)
- **High Risk**: Architecture changes, algorithm rewrites → Blue-Green or Canary with long validation

### 5. Automate Rollback Triggers

**Antipattern**: Manual decision for rollback (delays response)

**Best Practice**: Automated rollback on threshold violations

**Automated Rollback Triggers**:
```yaml
rollback_triggers:
  - error_rate > 1% for 5 minutes
  - latency_p95 > 100ms for 10 minutes
  - cache_hit_rate < 80% for 15 minutes
  - health_check_failures > 50%
```

---

## Appendix: Deployment Decision Tree

```
Start: Need to deploy new version
│
├─ Downtime acceptable?
│  ├─ Yes → Recreate (simplest, lowest cost)
│  └─ No → Continue
│
├─ Can afford 2× infrastructure?
│  ├─ Yes → Blue-Green (instant rollback)
│  └─ No → Continue
│
├─ High-risk deployment?
│  ├─ Yes → Canary (gradual rollout with validation)
│  └─ No → Rolling (standard zero-downtime)
│
└─ Testing feature variants?
   └─ Yes → A/B Testing (statistical decision)
```

---

**Document Version**: 1.0.0
**Last Updated**: 2026-01-28
**Next Review**: 2026-02-28
**Owner**: Thread Operations Team
