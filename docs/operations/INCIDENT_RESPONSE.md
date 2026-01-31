<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

[38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
     [38;5;238m│ [0m[1mSTDIN[0m
[38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
[38;5;238m   1[0m [38;5;238m│[0m [38;5;231m# Incident Response Runbooks[0m
[38;5;238m   2[0m [38;5;238m│[0m 
[38;5;238m   3[0m [38;5;238m│[0m [38;5;231m**Version**: 1.0.0[0m
[38;5;238m   4[0m [38;5;238m│[0m [38;5;231m**Last Updated**: 2026-01-28[0m
[38;5;238m   5[0m [38;5;238m│[0m [38;5;231m**Status**: Production Ready[0m
[38;5;238m   6[0m [38;5;238m│[0m 
[38;5;238m   7[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m   8[0m [38;5;238m│[0m 
[38;5;238m   9[0m [38;5;238m│[0m [38;5;231m## Incident Classification[0m
[38;5;238m  10[0m [38;5;238m│[0m 
[38;5;238m  11[0m [38;5;238m│[0m [38;5;231m### Severity Levels[0m
[38;5;238m  12[0m [38;5;238m│[0m 
[38;5;238m  13[0m [38;5;238m│[0m [38;5;231m| Severity | Impact | Response Time | Examples |[0m
[38;5;238m  14[0m [38;5;238m│[0m [38;5;231m|----------|--------|---------------|----------|[0m
[38;5;238m  15[0m [38;5;238m│[0m [38;5;231m| **SEV-1** | Complete outage | 15 minutes | Service down, data loss |[0m
[38;5;238m  16[0m [38;5;238m│[0m [38;5;231m| **SEV-2** | Major degradation | 30 minutes | High error rate, slow responses |[0m
[38;5;238m  17[0m [38;5;238m│[0m [38;5;231m| **SEV-3** | Partial degradation | 2 hours | Single feature broken |[0m
[38;5;238m  18[0m [38;5;238m│[0m [38;5;231m| **SEV-4** | Minor issue | 1 business day | Cosmetic bugs, low traffic impact |[0m
[38;5;238m  19[0m [38;5;238m│[0m 
[38;5;238m  20[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m  21[0m [38;5;238m│[0m 
[38;5;238m  22[0m [38;5;238m│[0m [38;5;231m## SEV-1: Service Down[0m
[38;5;238m  23[0m [38;5;238m│[0m 
[38;5;238m  24[0m [38;5;238m│[0m [38;5;231m**Symptoms**: Health check failing, 100% error rate, no successful requests[0m
[38;5;238m  25[0m [38;5;238m│[0m 
[38;5;238m  26[0m [38;5;238m│[0m [38;5;231m**Immediate Actions** (First 5 minutes):[0m
[38;5;238m  27[0m [38;5;238m│[0m [38;5;231m1. Page on-call engineer[0m
[38;5;238m  28[0m [38;5;238m│[0m [38;5;231m2. Create incident channel (#incident-YYYYMMDD-HH)[0m
[38;5;238m  29[0m [38;5;238m│[0m [38;5;231m3. Start incident timeline in shared doc[0m
[38;5;238m  30[0m [38;5;238m│[0m [38;5;231m4. Check deployment history: Recent deployment?[0m
[38;5;238m  31[0m [38;5;238m│[0m [38;5;231m5. Check infrastructure: All instances healthy?[0m
[38;5;238m  32[0m [38;5;238m│[0m 
[38;5;238m  33[0m [38;5;238m│[0m [38;5;231m**Investigation** (Minutes 5-15):[0m
[38;5;238m  34[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m  35[0m [38;5;238m│[0m [38;5;231m# Check service status[0m
[38;5;238m  36[0m [38;5;238m│[0m [38;5;231mkubectl get pods -n production | grep thread[0m
[38;5;238m  37[0m [38;5;238m│[0m 
[38;5;238m  38[0m [38;5;238m│[0m [38;5;231m# Check logs for errors[0m
[38;5;238m  39[0m [38;5;238m│[0m [38;5;231mkubectl logs -n production deployment/thread-worker --tail=100[0m
[38;5;238m  40[0m [38;5;238m│[0m 
[38;5;238m  41[0m [38;5;238m│[0m [38;5;231m# Check health endpoint[0m
[38;5;238m  42[0m [38;5;238m│[0m [38;5;231mcurl -v https://api.thread.io/health[0m
[38;5;238m  43[0m [38;5;238m│[0m 
[38;5;238m  44[0m [38;5;238m│[0m [38;5;231m# Check database connectivity[0m
[38;5;238m  45[0m [38;5;238m│[0m [38;5;231mpsql $DATABASE_URL -c "SELECT 1;"[0m
[38;5;238m  46[0m [38;5;238m│[0m 
[38;5;238m  47[0m [38;5;238m│[0m [38;5;231m# Check recent deployments[0m
[38;5;238m  48[0m [38;5;238m│[0m [38;5;231mkubectl rollout history deployment/thread-worker -n production[0m
[38;5;238m  49[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m  50[0m [38;5;238m│[0m 
[38;5;238m  51[0m [38;5;238m│[0m [38;5;231m**Resolution Paths**:[0m
[38;5;238m  52[0m [38;5;238m│[0m 
[38;5;238m  53[0m [38;5;238m│[0m [38;5;231m**Path A: Recent Deployment Issue**[0m
[38;5;238m  54[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m  55[0m [38;5;238m│[0m [38;5;231m# Rollback to previous version[0m
[38;5;238m  56[0m [38;5;238m│[0m [38;5;231mkubectl rollout undo deployment/thread-worker -n production[0m
[38;5;238m  57[0m [38;5;238m│[0m 
[38;5;238m  58[0m [38;5;238m│[0m [38;5;231m# Monitor rollback[0m
[38;5;238m  59[0m [38;5;238m│[0m [38;5;231mkubectl rollout status deployment/thread-worker -n production[0m
[38;5;238m  60[0m [38;5;238m│[0m 
[38;5;238m  61[0m [38;5;238m│[0m [38;5;231m# Verify service recovery[0m
[38;5;238m  62[0m [38;5;238m│[0m [38;5;231m./scripts/continuous-validation.sh production[0m
[38;5;238m  63[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m  64[0m [38;5;238m│[0m 
[38;5;238m  65[0m [38;5;238m│[0m [38;5;231m**Path B: Infrastructure Issue**[0m
[38;5;238m  66[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m  67[0m [38;5;238m│[0m [38;5;231m# Check node health[0m
[38;5;238m  68[0m [38;5;238m│[0m [38;5;231mkubectl get nodes[0m
[38;5;238m  69[0m [38;5;238m│[0m 
[38;5;238m  70[0m [38;5;238m│[0m [38;5;231m# Restart failed pods[0m
[38;5;238m  71[0m [38;5;238m│[0m [38;5;231mkubectl delete pod <pod-name> -n production[0m
[38;5;238m  72[0m [38;5;238m│[0m 
[38;5;238m  73[0m [38;5;238m│[0m [38;5;231m# Check resource constraints[0m
[38;5;238m  74[0m [38;5;238m│[0m [38;5;231mkubectl top nodes[0m
[38;5;238m  75[0m [38;5;238m│[0m [38;5;231mkubectl top pods -n production[0m
[38;5;238m  76[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m  77[0m [38;5;238m│[0m 
[38;5;238m  78[0m [38;5;238m│[0m [38;5;231m**Path C: Database Connectivity**[0m
[38;5;238m  79[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m  80[0m [38;5;238m│[0m [38;5;231m# Check database status[0m
[38;5;238m  81[0m [38;5;238m│[0m [38;5;231mpg_isready -h $DB_HOST[0m
[38;5;238m  82[0m [38;5;238m│[0m 
[38;5;238m  83[0m [38;5;238m│[0m [38;5;231m# Check connection pool[0m
[38;5;238m  84[0m [38;5;238m│[0m [38;5;231mSELECT count(*) FROM pg_stat_activity WHERE datname='thread';[0m
[38;5;238m  85[0m [38;5;238m│[0m 
[38;5;238m  86[0m [38;5;238m│[0m [38;5;231m# If pool exhausted, restart application[0m
[38;5;238m  87[0m [38;5;238m│[0m [38;5;231mkubectl rollout restart deployment/thread-worker -n production[0m
[38;5;238m  88[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m  89[0m [38;5;238m│[0m 
[38;5;238m  90[0m [38;5;238m│[0m [38;5;231m**Communication Template**:[0m
[38;5;238m  91[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m  92[0m [38;5;238m│[0m [38;5;231m🚨 INCIDENT: Service Outage[0m
[38;5;238m  93[0m [38;5;238m│[0m 
[38;5;238m  94[0m [38;5;238m│[0m [38;5;231mStatus: Investigating[0m
[38;5;238m  95[0m [38;5;238m│[0m [38;5;231mSeverity: SEV-1[0m
[38;5;238m  96[0m [38;5;238m│[0m [38;5;231mStart Time: [TIME][0m
[38;5;238m  97[0m [38;5;238m│[0m [38;5;231mImpact: All users unable to access service[0m
[38;5;238m  98[0m [38;5;238m│[0m 
[38;5;238m  99[0m [38;5;238m│[0m [38;5;231mTimeline:[0m
[38;5;238m 100[0m [38;5;238m│[0m [38;5;231m- [TIME] Alert triggered: Service health check failing[0m
[38;5;238m 101[0m [38;5;238m│[0m [38;5;231m- [TIME] On-call engineer paged[0m
[38;5;238m 102[0m [38;5;238m│[0m [38;5;231m- [TIME] Investigation started[0m
[38;5;238m 103[0m [38;5;238m│[0m [38;5;231m- [TIME] Root cause: [IDENTIFIED CAUSE][0m
[38;5;238m 104[0m [38;5;238m│[0m [38;5;231m- [TIME] Mitigation: [ACTION TAKEN][0m
[38;5;238m 105[0m [38;5;238m│[0m [38;5;231m- [TIME] Service restored[0m
[38;5;238m 106[0m [38;5;238m│[0m 
[38;5;238m 107[0m [38;5;238m│[0m [38;5;231mNext Update: Every 15 minutes[0m
[38;5;238m 108[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 109[0m [38;5;238m│[0m 
[38;5;238m 110[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m 111[0m [38;5;238m│[0m 
[38;5;238m 112[0m [38;5;238m│[0m [38;5;231m## SEV-2: High Error Rate[0m
[38;5;238m 113[0m [38;5;238m│[0m 
[38;5;238m 114[0m [38;5;238m│[0m [38;5;231m**Symptoms**: Error rate > 1%, P95 latency > 1 second, partial service degradation[0m
[38;5;238m 115[0m [38;5;238m│[0m 
[38;5;238m 116[0m [38;5;238m│[0m [38;5;231m**Immediate Actions**:[0m
[38;5;238m 117[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m 118[0m [38;5;238m│[0m [38;5;231m# Check error rate[0m
[38;5;238m 119[0m [38;5;238m│[0m [38;5;231mcurl -s "http://prometheus:9090/api/v1/query?query=sum(rate(http_requests_total{status=~\"5..\"}[5m]))/sum(rate(http_requests_total[5m]))"[0m
[38;5;238m 120[0m [38;5;238m│[0m 
[38;5;238m 121[0m [38;5;238m│[0m [38;5;231m# Check error logs[0m
[38;5;238m 122[0m [38;5;238m│[0m [38;5;231mkubectl logs -n production deployment/thread-worker --tail=500 | grep ERROR[0m
[38;5;238m 123[0m [38;5;238m│[0m 
[38;5;238m 124[0m [38;5;238m│[0m [38;5;231m# Identify error patterns[0m
[38;5;238m 125[0m [38;5;238m│[0m [38;5;231mkubectl logs -n production deployment/thread-worker --tail=1000 \[0m
[38;5;238m 126[0m [38;5;238m│[0m [38;5;231m  | grep ERROR | awk '{print $NF}' | sort | uniq -c | sort -rn[0m
[38;5;238m 127[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 128[0m [38;5;238m│[0m 
[38;5;238m 129[0m [38;5;238m│[0m [38;5;231m**Common Causes**:[0m
[38;5;238m 130[0m [38;5;238m│[0m 
[38;5;238m 131[0m [38;5;238m│[0m [38;5;231m**A: Database Slow Queries**[0m
[38;5;238m 132[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m 133[0m [38;5;238m│[0m [38;5;231m# Find slow queries[0m
[38;5;238m 134[0m [38;5;238m│[0m [38;5;231mpsql $DATABASE_URL << 'SQL'[0m
[38;5;238m 135[0m [38;5;238m│[0m [38;5;231mSELECT query, calls, mean_exec_time, max_exec_time[0m
[38;5;238m 136[0m [38;5;238m│[0m [38;5;231mFROM pg_stat_statements[0m
[38;5;238m 137[0m [38;5;238m│[0m [38;5;231mWHERE mean_exec_time > 1000  -- > 1 second[0m
[38;5;238m 138[0m [38;5;238m│[0m [38;5;231mORDER BY mean_exec_time DESC[0m
[38;5;238m 139[0m [38;5;238m│[0m [38;5;231mLIMIT 10;[0m
[38;5;238m 140[0m [38;5;238m│[0m [38;5;231mSQL[0m
[38;5;238m 141[0m [38;5;238m│[0m 
[38;5;238m 142[0m [38;5;238m│[0m [38;5;231m# Terminate long-running queries[0m
[38;5;238m 143[0m [38;5;238m│[0m [38;5;231mSELECT pg_terminate_backend(pid)[0m
[38;5;238m 144[0m [38;5;238m│[0m [38;5;231mFROM pg_stat_activity[0m
[38;5;238m 145[0m [38;5;238m│[0m [38;5;231mWHERE state = 'active' AND query_start < now() - interval '5 minutes';[0m
[38;5;238m 146[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 147[0m [38;5;238m│[0m 
[38;5;238m 148[0m [38;5;238m│[0m [38;5;231m**B: Memory Pressure**[0m
[38;5;238m 149[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m 150[0m [38;5;238m│[0m [38;5;231m# Check memory usage[0m
[38;5;238m 151[0m [38;5;238m│[0m [38;5;231mkubectl top pods -n production[0m
[38;5;238m 152[0m [38;5;238m│[0m 
[38;5;238m 153[0m [38;5;238m│[0m [38;5;231m# Restart high-memory pods[0m
[38;5;238m 154[0m [38;5;238m│[0m [38;5;231mkubectl delete pod <high-memory-pod> -n production[0m
[38;5;238m 155[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 156[0m [38;5;238m│[0m 
[38;5;238m 157[0m [38;5;238m│[0m [38;5;231m**C: External Service Timeout**[0m
[38;5;238m 158[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m 159[0m [38;5;238m│[0m [38;5;231m# Check circuit breaker status[0m
[38;5;238m 160[0m [38;5;238m│[0m [38;5;231mcurl -s https://api.thread.io/health/circuit-breakers[0m
[38;5;238m 161[0m [38;5;238m│[0m 
[38;5;238m 162[0m [38;5;238m│[0m [38;5;231m# Implement temporary failover/degraded mode[0m
[38;5;238m 163[0m [38;5;238m│[0m [38;5;231m# (Application-specific)[0m
[38;5;238m 164[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 165[0m [38;5;238m│[0m 
[38;5;238m 166[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m 167[0m [38;5;238m│[0m 
[38;5;238m 168[0m [38;5;238m│[0m [38;5;231m## SEV-3: Partial Feature Broken[0m
[38;5;238m 169[0m [38;5;238m│[0m 
[38;5;238m 170[0m [38;5;238m│[0m [38;5;231m**Symptoms**: Specific API endpoint failing, isolated functionality broken[0m
[38;5;238m 171[0m [38;5;238m│[0m 
[38;5;238m 172[0m [38;5;238m│[0m [38;5;231m**Investigation**:[0m
[38;5;238m 173[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m 174[0m [38;5;238m│[0m [38;5;231m# Identify failing endpoints[0m
[38;5;238m 175[0m [38;5;238m│[0m [38;5;231mkubectl logs -n production deployment/thread-worker \[0m
[38;5;238m 176[0m [38;5;238m│[0m [38;5;231m  | grep "status:500" | awk '{print $5}' | sort | uniq -c[0m
[38;5;238m 177[0m [38;5;238m│[0m 
[38;5;238m 178[0m [38;5;238m│[0m [38;5;231m# Test specific endpoint[0m
[38;5;238m 179[0m [38;5;238m│[0m [38;5;231mcurl -v https://api.thread.io/api/query \[0m
[38;5;238m 180[0m [38;5;238m│[0m [38;5;231m  -H "Content-Type: application/json" \[0m
[38;5;238m 181[0m [38;5;238m│[0m [38;5;231m  -d '{"pattern":"test"}'[0m
[38;5;238m 182[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 183[0m [38;5;238m│[0m 
[38;5;238m 184[0m [38;5;238m│[0m [38;5;231m**Resolution**:[0m
[38;5;238m 185[0m [38;5;238m│[0m [38;5;231m- Fix bug and deploy patch[0m
[38;5;238m 186[0m [38;5;238m│[0m [38;5;231m- OR disable feature flag if feature-flagged[0m
[38;5;238m 187[0m [38;5;238m│[0m [38;5;231m- OR apply workaround and schedule proper fix[0m
[38;5;238m 188[0m [38;5;238m│[0m 
[38;5;238m 189[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m 190[0m [38;5;238m│[0m 
[38;5;238m 191[0m [38;5;238m│[0m [38;5;231m## Database Issues[0m
[38;5;238m 192[0m [38;5;238m│[0m 
[38;5;238m 193[0m [38;5;238m│[0m [38;5;231m### Connection Pool Exhaustion[0m
[38;5;238m 194[0m [38;5;238m│[0m 
[38;5;238m 195[0m [38;5;238m│[0m [38;5;231m**Symptoms**: "connection pool exhausted" errors[0m
[38;5;238m 196[0m [38;5;238m│[0m 
[38;5;238m 197[0m [38;5;238m│[0m [38;5;231m**Quick Fix**:[0m
[38;5;238m 198[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m 199[0m [38;5;238m│[0m [38;5;231m# Restart application (resets pool)[0m
[38;5;238m 200[0m [38;5;238m│[0m [38;5;231mkubectl rollout restart deployment/thread-worker -n production[0m
[38;5;238m 201[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 202[0m [38;5;238m│[0m 
[38;5;238m 203[0m [38;5;238m│[0m [38;5;231m**Long-term Fix**:[0m
[38;5;238m 204[0m [38;5;238m│[0m [38;5;231m```toml[0m
[38;5;238m 205[0m [38;5;238m│[0m [38;5;231m# Increase pool size in config/production.toml[0m
[38;5;238m 206[0m [38;5;238m│[0m [38;5;231m[database][0m
[38;5;238m 207[0m [38;5;238m│[0m [38;5;231mmax_connections = 300  # Increased from 200[0m
[38;5;238m 208[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 209[0m [38;5;238m│[0m 
[38;5;238m 210[0m [38;5;238m│[0m [38;5;231m### Slow Queries[0m
[38;5;238m 211[0m [38;5;238m│[0m 
[38;5;238m 212[0m [38;5;238m│[0m [38;5;231m**Investigation**:[0m
[38;5;238m 213[0m [38;5;238m│[0m [38;5;231m```sql[0m
[38;5;238m 214[0m [38;5;238m│[0m [38;5;231m-- Active queries[0m
[38;5;238m 215[0m [38;5;238m│[0m [38;5;231mSELECT pid, age(clock_timestamp(), query_start), usename, query[0m
[38;5;238m 216[0m [38;5;238m│[0m [38;5;231mFROM pg_stat_activity[0m
[38;5;238m 217[0m [38;5;238m│[0m [38;5;231mWHERE state != 'idle' AND query NOT ILIKE '%pg_stat_activity%'[0m
[38;5;238m 218[0m [38;5;238m│[0m [38;5;231mORDER BY query_start;[0m
[38;5;238m 219[0m [38;5;238m│[0m 
[38;5;238m 220[0m [38;5;238m│[0m [38;5;231m-- Table bloat[0m
[38;5;238m 221[0m [38;5;238m│[0m [38;5;231mSELECT schemaname, tablename, pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename))[0m
[38;5;238m 222[0m [38;5;238m│[0m [38;5;231mFROM pg_tables[0m
[38;5;238m 223[0m [38;5;238m│[0m [38;5;231mORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC[0m
[38;5;238m 224[0m [38;5;238m│[0m [38;5;231mLIMIT 10;[0m
[38;5;238m 225[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 226[0m [38;5;238m│[0m 
[38;5;238m 227[0m [38;5;238m│[0m [38;5;231m**Resolution**:[0m
[38;5;238m 228[0m [38;5;238m│[0m [38;5;231m```sql[0m
[38;5;238m 229[0m [38;5;238m│[0m [38;5;231m-- Kill slow query[0m
[38;5;238m 230[0m [38;5;238m│[0m [38;5;231mSELECT pg_terminate_backend(<pid>);[0m
[38;5;238m 231[0m [38;5;238m│[0m 
[38;5;238m 232[0m [38;5;238m│[0m [38;5;231m-- VACUUM bloated table[0m
[38;5;238m 233[0m [38;5;238m│[0m [38;5;231mVACUUM ANALYZE table_name;[0m
[38;5;238m 234[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 235[0m [38;5;238m│[0m 
[38;5;238m 236[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m 237[0m [38;5;238m│[0m 
[38;5;238m 238[0m [38;5;238m│[0m [38;5;231m## Cache Issues[0m
[38;5;238m 239[0m [38;5;238m│[0m 
[38;5;238m 240[0m [38;5;238m│[0m [38;5;231m### Low Hit Rate[0m
[38;5;238m 241[0m [38;5;238m│[0m 
[38;5;238m 242[0m [38;5;238m│[0m [38;5;231m**Investigation**:[0m
[38;5;238m 243[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m 244[0m [38;5;238m│[0m [38;5;231m# Check hit rate[0m
[38;5;238m 245[0m [38;5;238m│[0m [38;5;231mredis-cli INFO stats | grep keyspace[0m
[38;5;238m 246[0m [38;5;238m│[0m 
[38;5;238m 247[0m [38;5;238m│[0m [38;5;231m# Check eviction rate[0m
[38;5;238m 248[0m [38;5;238m│[0m [38;5;231mredis-cli INFO stats | grep evicted[0m
[38;5;238m 249[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 250[0m [38;5;238m│[0m 
[38;5;238m 251[0m [38;5;238m│[0m [38;5;231m**Resolution**:[0m
[38;5;238m 252[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m 253[0m [38;5;238m│[0m [38;5;231m# Increase cache memory (if available)[0m
[38;5;238m 254[0m [38;5;238m│[0m [38;5;231m# OR reduce TTL for less important data[0m
[38;5;238m 255[0m [38;5;238m│[0m [38;5;231m# OR implement cache warming for critical paths[0m
[38;5;238m 256[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 257[0m [38;5;238m│[0m 
[38;5;238m 258[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m 259[0m [38;5;238m│[0m 
[38;5;238m 260[0m [38;5;238m│[0m [38;5;231m## Post-Incident Review[0m
[38;5;238m 261[0m [38;5;238m│[0m 
[38;5;238m 262[0m [38;5;238m│[0m [38;5;231m**Template** (`claudedocs/incident-YYYYMMDD.md`):[0m
[38;5;238m 263[0m [38;5;238m│[0m [38;5;231m```markdown[0m
[38;5;238m 264[0m [38;5;238m│[0m [38;5;231m# Incident Report: [TITLE][0m
[38;5;238m 265[0m [38;5;238m│[0m 
[38;5;238m 266[0m [38;5;238m│[0m [38;5;231m**Date**: YYYY-MM-DD[0m
[38;5;238m 267[0m [38;5;238m│[0m [38;5;231m**Severity**: SEV-X[0m
[38;5;238m 268[0m [38;5;238m│[0m [38;5;231m**Duration**: X hours X minutes[0m
[38;5;238m 269[0m [38;5;238m│[0m [38;5;231m**Impact**: [User impact description][0m
[38;5;238m 270[0m [38;5;238m│[0m 
[38;5;238m 271[0m [38;5;238m│[0m [38;5;231m## Timeline[0m
[38;5;238m 272[0m [38;5;238m│[0m 
[38;5;238m 273[0m [38;5;238m│[0m [38;5;231m- HH:MM - Alert triggered[0m
[38;5;238m 274[0m [38;5;238m│[0m [38;5;231m- HH:MM - Investigation started[0m
[38;5;238m 275[0m [38;5;238m│[0m [38;5;231m- HH:MM - Root cause identified[0m
[38;5;238m 276[0m [38;5;238m│[0m [38;5;231m- HH:MM - Mitigation deployed[0m
[38;5;238m 277[0m [38;5;238m│[0m [38;5;231m- HH:MM - Service restored[0m
[38;5;238m 278[0m [38;5;238m│[0m [38;5;231m- HH:MM - Incident closed[0m
[38;5;238m 279[0m [38;5;238m│[0m 
[38;5;238m 280[0m [38;5;238m│[0m [38;5;231m## Root Cause[0m
[38;5;238m 281[0m [38;5;238m│[0m 
[38;5;238m 282[0m [38;5;238m│[0m [38;5;231m[Detailed root cause analysis][0m
[38;5;238m 283[0m [38;5;238m│[0m 
[38;5;238m 284[0m [38;5;238m│[0m [38;5;231m## Resolution[0m
[38;5;238m 285[0m [38;5;238m│[0m 
[38;5;238m 286[0m [38;5;238m│[0m [38;5;231m[What was done to resolve the incident][0m
[38;5;238m 287[0m [38;5;238m│[0m 
[38;5;238m 288[0m [38;5;238m│[0m [38;5;231m## Action Items[0m
[38;5;238m 289[0m [38;5;238m│[0m 
[38;5;238m 290[0m [38;5;238m│[0m [38;5;231m- [ ] [Action 1 - Owner: Name - Due: Date][0m
[38;5;238m 291[0m [38;5;238m│[0m [38;5;231m- [ ] [Action 2 - Owner: Name - Due: Date][0m
[38;5;238m 292[0m [38;5;238m│[0m 
[38;5;238m 293[0m [38;5;238m│[0m [38;5;231m## Lessons Learned[0m
[38;5;238m 294[0m [38;5;238m│[0m 
[38;5;238m 295[0m [38;5;238m│[0m [38;5;231m**What Went Well**:[0m
[38;5;238m 296[0m [38;5;238m│[0m [38;5;231m- [Item 1][0m
[38;5;238m 297[0m [38;5;238m│[0m 
[38;5;238m 298[0m [38;5;238m│[0m [38;5;231m**What Could Be Improved**:[0m
[38;5;238m 299[0m [38;5;238m│[0m [38;5;231m- [Item 1][0m
[38;5;238m 300[0m [38;5;238m│[0m 
[38;5;238m 301[0m [38;5;238m│[0m [38;5;231m**Follow-up Actions**:[0m
[38;5;238m 302[0m [38;5;238m│[0m [38;5;231m- [Action 1][0m
[38;5;238m 303[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 304[0m [38;5;238m│[0m 
[38;5;238m 305[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m 306[0m [38;5;238m│[0m 
[38;5;238m 307[0m [38;5;238m│[0m [38;5;231m**Document Version**: 1.0.0[0m
[38;5;238m 308[0m [38;5;238m│[0m [38;5;231m**Last Updated**: 2026-01-28[0m
[38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
