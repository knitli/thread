# Thread Service Level Indicators (SLI) & Objectives (SLO)

**Purpose**: Formal definitions of performance targets and measurement methodologies
**Version**: 1.0
**Last Updated**: 2026-01-28
**Review Frequency**: Quarterly

---

## Overview

This document defines Service Level Indicators (SLIs) and Service Level Objectives (SLOs) for the Thread codebase analysis platform in accordance with Thread Constitution v2.0.0, Principle VI (Service Architecture & Persistence).

### SLI/SLO Framework

**Service Level Indicator (SLI)**: A quantitative measure of a service's behavior
**Service Level Objective (SLO)**: A target value or range for an SLI
**Error Budget**: Allowed deviation from SLO (100% - SLO%)

### Measurement Windows

| Window Type | Duration | Usage |
|-------------|----------|-------|
| Real-time | 1 minute | Immediate alerting |
| Short-term | 5 minutes | Operational monitoring |
| Medium-term | 1 hour | Trend analysis |
| Long-term | 30 days | SLO compliance reporting |

---

## Constitutional Compliance SLIs

### SLI-CC-1: Content-Addressed Cache Hit Rate

**Definition**: Percentage of file analysis requests served from content-addressed cache

**Measurement**:
```promql
# SLI calculation (last 5 minutes)
100 * (
  sum(rate(thread_cache_hits_total[5m]))
  /
  (sum(rate(thread_cache_hits_total[5m])) + sum(rate(thread_cache_misses_total[5m])))
)
```

**SLO Targets**:
| Target | Value | Justification |
|--------|-------|---------------|
| **Constitutional Minimum** | **>90%** | Thread Constitution v2.0.0, Principle VI |
| Production Target | >93% | Provides 3% error budget |
| Aspirational | >95% | Optimal performance |

**Error Budget**: 10% (Constitutional), 7% (Production)

**Measurement Frequency**: Continuous (15-second scrape interval)

**Alert Thresholds**:
- **Warning**: <85% for 5 minutes (approaching limit)
- **Critical**: <80% for 2 minutes (Constitutional violation)

**Exclusions**: None - All cache operations count

**Measurement Source**: Prometheus `thread_cache_hits_total`, `thread_cache_misses_total`

---

### SLI-CC-2: Postgres Query Latency (p95)

**Definition**: 95th percentile latency for Postgres database queries

**Measurement**:
```promql
# SLI calculation (p95 over 5 minutes)
histogram_quantile(0.95,
  rate(thread_postgres_query_duration_seconds_bucket[5m])
) * 1000  # Convert to milliseconds
```

**SLO Targets**:
| Target | Value | Justification |
|--------|-------|---------------|
| **Constitutional Maximum** | **<10ms** | Thread Constitution v2.0.0, Principle VI |
| Production Target | <8ms | Provides 2ms error budget |
| Aspirational | <5ms | Excellent performance |

**Error Budget**: Queries may exceed 10ms for 5% of requests

**Measurement Frequency**: Continuous (15-second scrape interval)

**Alert Thresholds**:
- **Warning**: >10ms p95 for 2 minutes (Constitutional limit)
- **Critical**: >20ms p95 for 1 minute (Severe degradation)

**Exclusions**:
- Connection establishment time (excluded)
- Transaction commit time (included)
- Query planning time (included)

**Measurement Source**: Prometheus `thread_postgres_query_duration_seconds`

**Current Status**: ⚠️ **Not Yet Instrumented** (Pending Task #51)

---

### SLI-CC-3: D1 Query Latency (p95)

**Definition**: 95th percentile latency for D1 database queries (Edge deployment)

**Measurement**:
```promql
# SLI calculation (p95 over 5 minutes)
histogram_quantile(0.95,
  rate(thread_d1_query_duration_seconds_bucket[5m])
) * 1000  # Convert to milliseconds
```

**SLO Targets**:
| Target | Value | Justification |
|--------|-------|---------------|
| **Constitutional Maximum** | **<50ms** | Thread Constitution v2.0.0, Principle VI |
| Production Target | <40ms | Provides 10ms error budget |
| Aspirational | <30ms | Excellent performance |

**Error Budget**: Queries may exceed 50ms for 5% of requests

**Measurement Frequency**: Continuous (15-second scrape interval)

**Alert Thresholds**:
- **Warning**: >50ms p95 for 2 minutes (Constitutional limit)
- **Critical**: >100ms p95 for 1 minute (Severe degradation)

**Exclusions**:
- Network latency to Cloudflare edge (included)
- HTTP overhead (included)
- Connection establishment (included - HTTP-based)

**Measurement Source**: Prometheus `thread_d1_query_duration_seconds`

**Current Status**: ⚠️ **Not Yet Instrumented** (Pending Task #51)

---

### SLI-CC-4: Incremental Update Coverage

**Definition**: Percentage of file changes triggering targeted re-analysis (vs full re-analysis)

**Measurement**:
```promql
# SLI calculation (last 5 minutes)
100 * (
  sum(rate(thread_incremental_updates_total[5m]))
  /
  sum(rate(thread_file_changes_total[5m]))
)
```

**SLO Targets**:
| Target | Value | Justification |
|--------|-------|---------------|
| **Constitutional Minimum** | **>0%** | Thread Constitution v2.0.0, Principle VI |
| Production Target | >80% | Efficient incremental analysis |
| Aspirational | >95% | Near-perfect incremental coverage |

**Error Budget**: N/A (Binary: implemented or not)

**Measurement Frequency**: Continuous (15-second scrape interval)

**Alert Thresholds**:
- **Critical**: <1% for 10 minutes (Feature not working)

**Exclusions**: None

**Measurement Source**: Prometheus `thread_incremental_updates_total`, `thread_file_changes_total`

**Current Status**: ❌ **Not Implemented** (Constitutional violation)

**Implementation Timeline**: Month 1-2 (2-3 weeks effort)

---

## Performance SLIs

### SLI-PERF-1: Fingerprint Computation Time

**Definition**: Average time to compute Blake3 content fingerprint per file

**Measurement**:
```promql
# SLI calculation (average over 5 minutes)
(
  rate(thread_fingerprint_duration_seconds_sum[5m])
  /
  rate(thread_fingerprint_duration_seconds_count[5m])
) * 1000000  # Convert to microseconds
```

**SLO Targets**:
| Target | Value | Justification |
|--------|-------|---------------|
| Maximum | <1µs | Negligible overhead vs parsing (147µs) |
| Production Target | <500ns | Provides 500ns error budget |
| Current Baseline | 425ns | Measured performance |

**Error Budget**: 575ns variance allowed

**Measurement Frequency**: Continuous (15-second scrape interval)

**Alert Thresholds**:
- **Warning**: >1µs for 1 minute (Approaching limit)
- **Critical**: >2µs for 30 seconds (Severe regression)

**Exclusions**: None - Pure computation time

**Measurement Source**: Prometheus `thread_fingerprint_duration_seconds`

**Current Status**: ✅ **Exceeds Target** (425ns < 1µs)

---

### SLI-PERF-2: AST Parsing Throughput

**Definition**: Rate of source code bytes parsed per second

**Measurement**:
```promql
# SLI calculation (MB/sec over 5 minutes)
rate(thread_bytes_processed_total[5m]) / 1024 / 1024
```

**SLO Targets**:
| Target | Value | Justification |
|--------|-------|---------------|
| Minimum | >5 MiB/s | Baseline single-thread performance |
| Production Target | >100 MiB/s | With caching (90% hit rate) |
| Aspirational | >400 MiB/s | Optimal caching (>95% hit rate) |

**Error Budget**: May fall below 5 MiB/s for 5% of time (cold cache)

**Measurement Frequency**: Continuous (15-second scrape interval)

**Alert Thresholds**:
- **Warning**: <4 MiB/s for 5 minutes (Below baseline)
- **Critical**: <2 MiB/s for 2 minutes (Severe degradation)

**Exclusions**: Network I/O, database queries (separate SLIs)

**Measurement Source**: Prometheus `thread_bytes_processed_total`

**Current Status**: ✅ **Meets Target** (5.0-5.3 MiB/s baseline, 430-672 MiB/s with cache)

---

### SLI-PERF-3: Pattern Matching Latency (p50)

**Definition**: Median time to execute AST pattern matching operation

**Measurement**:
```promql
# SLI calculation (p50 over 5 minutes)
histogram_quantile(0.50,
  rate(thread_pattern_match_duration_seconds_bucket[5m])
) * 1000000  # Convert to microseconds
```

**SLO Targets**:
| Target | Value | Justification |
|--------|-------|---------------|
| Maximum | <150µs | Acceptable pattern matching overhead |
| Production Target | <120µs | Provides 30µs error budget |
| Current Baseline | 101.65µs | Measured performance |

**Error Budget**: 48.35µs variance allowed

**Measurement Frequency**: Continuous (via CI benchmarks)

**Alert Thresholds**:
- **Warning**: >10% regression from baseline (>111.8µs)
- **Critical**: >20% regression from baseline (>121.9µs)

**Exclusions**: Tree-sitter parsing (separate benchmark)

**Measurement Source**: Criterion benchmarks (`pattern_conversion_optimized`)

**Current Status**: ✅ **Exceeds Target** (101.65µs < 150µs)

---

### SLI-PERF-4: Parallel Processing Efficiency

**Definition**: Speedup factor achieved with 8-core parallel processing vs single-thread

**Measurement**:
```promql
# SLI calculation (speedup factor from load tests)
thread_parallel_8core_throughput / thread_sequential_throughput
```

**SLO Targets**:
| Target | Value | Justification |
|--------|-------|---------------|
| Minimum | >6x | 75% parallel efficiency (6/8 cores) |
| Production Target | >7x | 87.5% parallel efficiency |
| Current Baseline | 7.2x | 90% parallel efficiency |

**Error Budget**: May fall below 6x for 5% of workloads

**Measurement Frequency**: Weekly (via load test benchmarks)

**Alert Thresholds**:
- **Warning**: <6.5x speedup (Efficiency degradation)
- **Critical**: <5.5x speedup (Severe efficiency loss)

**Exclusions**:
- Single-core systems (N/A)
- Edge deployments (no parallel processing)

**Measurement Source**: Load test benchmarks (`concurrent_processing/parallel`)

**Current Status**: ✅ **Exceeds Target** (7.2x > 6x)

---

## Reliability SLIs

### SLI-REL-1: Query Error Rate

**Definition**: Percentage of database queries resulting in errors

**Measurement**:
```promql
# SLI calculation (error rate over 5 minutes)
100 * (
  sum(rate(thread_query_errors_total[5m]))
  /
  (sum(rate(thread_query_success_total[5m])) + sum(rate(thread_query_errors_total[5m])))
)
```

**SLO Targets**:
| Target | Value | Justification |
|--------|-------|---------------|
| Maximum | <0.1% | High reliability requirement |
| Production Target | <0.05% | Provides 0.05% error budget |
| Aspirational | <0.01% | Excellent reliability |

**Error Budget**: 0.1% of queries may fail

**Measurement Frequency**: Continuous (15-second scrape interval)

**Alert Thresholds**:
- **Warning**: >1% for 2 minutes (Approaching limit)
- **Critical**: >5% for 1 minute (Severe reliability issue)

**Exclusions**: None - All query errors count

**Measurement Source**: Prometheus `thread_query_errors_total`, `thread_query_success_total`

**Current Status**: ⚠️ **Pending Measurement** (Monitoring in place, no data yet)

---

### SLI-REL-2: Service Availability

**Definition**: Percentage of time service responds to health checks

**Measurement**:
```promql
# SLI calculation (availability over 30 days)
100 * (
  sum(rate(thread_health_check_success_total[30d]))
  /
  (sum(rate(thread_health_check_success_total[30d])) + sum(rate(thread_health_check_failure_total[30d])))
)
```

**SLO Targets**:
| Target | Value | Justification |
|--------|-------|---------------|
| Minimum | >99.9% | "Three nines" availability |
| Production Target | >99.95% | Provides additional buffer |
| Aspirational | >99.99% | "Four nines" availability |

**Error Budget**: 43 minutes of downtime per month (99.9%)

**Measurement Frequency**: Continuous (15-second health checks)

**Alert Thresholds**:
- **Warning**: <99.9% over 1 hour (Error budget consumed)
- **Critical**: <99% over 30 minutes (Severe availability issue)

**Exclusions**: Planned maintenance windows (announced 24h in advance)

**Measurement Source**: Prometheus `thread_health_check_success_total`, `thread_health_check_failure_total`

**Current Status**: ⚠️ **Pending Implementation** (Health check endpoint needed)

---

### SLI-REL-3: Cache Eviction Rate

**Definition**: Number of cache entries evicted per second (LRU eviction)

**Measurement**:
```promql
# SLI calculation (evictions/sec over 5 minutes)
rate(thread_cache_evictions_total[5m])
```

**SLO Targets**:
| Target | Value | Justification |
|--------|-------|---------------|
| Maximum | <100/sec | Indicates stable cache size |
| Production Target | <50/sec | Low eviction rate (good cache sizing) |
| Aspirational | <10/sec | Excellent cache sizing |

**Error Budget**: N/A (Lower is better, no strict limit)

**Measurement Frequency**: Continuous (15-second scrape interval)

**Alert Thresholds**:
- **Warning**: >100/sec for 5 minutes (High eviction rate)
- **Critical**: >500/sec for 2 minutes (Thrashing, cache too small)

**Exclusions**: Manual cache clearing operations

**Measurement Source**: Prometheus `thread_cache_evictions_total`

**Current Status**: ✅ **Monitored** (Measurement active)

---

## SLO Compliance Reporting

### Compliance Calculation

**30-Day SLO Compliance**:
```promql
# Percentage of time SLI met SLO target over 30 days
100 * (
  count_over_time((thread_sli_value <= thread_slo_target)[30d:1m])
  /
  count_over_time(thread_sli_value[30d:1m])
)
```

**Error Budget Consumption**:
```promql
# Percentage of error budget consumed
100 * (
  (thread_slo_target - avg_over_time(thread_sli_value[30d]))
  /
  (100 - thread_slo_target)
)
```

### Compliance Targets

| SLO Category | 30-Day Compliance Target | Error Budget |
|--------------|--------------------------|--------------|
| Constitutional Compliance | >99% | 1% violations allowed |
| Performance | >98% | 2% violations allowed |
| Reliability | >99.9% | 0.1% violations allowed |

### Reporting Schedule

**Weekly**:
- SLO compliance dashboard review
- Error budget consumption tracking
- Trend analysis (improving/degrading)

**Monthly**:
- Formal SLO compliance report
- Root cause analysis for violations
- SLO target adjustments (if needed)

**Quarterly**:
- Comprehensive SLO review
- SLI/SLO definition updates
- Baseline recalibration

---

## SLI/SLO Summary Table

### Current Status

| SLI | SLO Target | Current | Compliance | Status |
|-----|------------|---------|------------|--------|
| **Constitutional Compliance** |
| CC-1: Cache Hit Rate | >90% | 80-95% | ✅ On track | Production |
| CC-2: Postgres p95 Latency | <10ms | ⚠️ Not measured | ⚠️ Pending | **Critical Gap** |
| CC-3: D1 p95 Latency | <50ms | ⚠️ Not measured | ⚠️ Pending | **Critical Gap** |
| CC-4: Incremental Updates | >0% | ❌ Not implemented | ❌ Fail | **Implementation Needed** |
| **Performance** |
| PERF-1: Fingerprint Time | <1µs | 425ns ✅ | ✅ Pass | Excellent |
| PERF-2: AST Throughput | >5 MiB/s | 5.0-5.3 MiB/s ✅ | ✅ Pass | Meets baseline |
| PERF-3: Pattern Matching | <150µs | 101.65µs ✅ | ✅ Pass | Excellent |
| PERF-4: Parallel Efficiency | >6x | 7.2x ✅ | ✅ Pass | Excellent |
| **Reliability** |
| REL-1: Query Error Rate | <0.1% | ⚠️ Pending data | ⚠️ Pending | Monitoring active |
| REL-2: Service Availability | >99.9% | ⚠️ Not implemented | ⚠️ Pending | **Implementation Needed** |
| REL-3: Cache Eviction Rate | <100/sec | ✅ Monitored | ✅ N/A | Monitoring active |

**Overall Compliance**: 4/11 Pass (36%) - 4 Pending, 3 Not Implemented

---

## Action Items

### Critical (P0)

1. **Instrument Database Queries** (Task #51)
   - Add Prometheus metrics for Postgres queries
   - Add Prometheus metrics for D1 queries
   - Validate p95 latency compliance
   - **Effort**: 2-3 days
   - **Owner**: Performance Engineering

2. **Implement Health Check Endpoint**
   - Add `/health` endpoint to service
   - Integrate with Prometheus monitoring
   - Configure uptime monitoring
   - **Effort**: 1 day
   - **Owner**: DevOps

### High (P1)

3. **Build Incremental Update System**
   - Implement tree-sitter `InputEdit` API
   - Add incremental parsing logic
   - Instrument metrics for coverage tracking
   - **Effort**: 2-3 weeks
   - **Owner**: Development Team

4. **Query Error Tracking**
   - Validate error rate metrics
   - Configure alerting thresholds
   - Establish error budget policy
   - **Effort**: 2 days
   - **Owner**: SRE

### Medium (P2)

5. **SLO Dashboard**
   - Create dedicated SLO compliance dashboard
   - Add error budget visualization
   - Configure trend analysis
   - **Effort**: 3 days
   - **Owner**: DevOps

6. **Automated SLO Reporting**
   - Build weekly compliance report automation
   - Email distribution to stakeholders
   - Integrate with incident management
   - **Effort**: 1 week
   - **Owner**: SRE

---

## Appendix

### References

**Thread Constitution v2.0.0**:
- Principle VI: Service Architecture & Persistence
  - Content-addressed caching: >90% hit rate
  - Postgres p95: <10ms
  - D1 p95: <50ms
  - Incremental updates: Automatic re-analysis

**Related Documentation**:
- `/docs/OPTIMIZATION_RESULTS.md` - Optimization results and baselines
- `/docs/PERFORMANCE_RUNBOOK.md` - Operational procedures
- `/docs/operations/PERFORMANCE_TUNING.md` - Tuning guide
- `/grafana/dashboards/thread-performance-monitoring.json` - Monitoring dashboard

### Revision History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0 | 2026-01-28 | Initial SLI/SLO definitions | Performance Engineering |

---

**Document Owner**: Performance Engineering Team
**Review Frequency**: Quarterly
**Next Review**: 2026-04-28
**Approval**: Pending stakeholder review
