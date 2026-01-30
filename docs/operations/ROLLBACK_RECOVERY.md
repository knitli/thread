<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

[38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
     [38;5;238m│ [0m[1mSTDIN[0m
[38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
[38;5;238m   1[0m [38;5;238m│[0m [38;5;231m# Rollback and Recovery Procedures[0m
[38;5;238m   2[0m [38;5;238m│[0m 
[38;5;238m   3[0m [38;5;238m│[0m [38;5;231m**Version**: 1.0.0[0m
[38;5;238m   4[0m [38;5;238m│[0m [38;5;231m**Last Updated**: 2026-01-28[0m
[38;5;238m   5[0m [38;5;238m│[0m 
[38;5;238m   6[0m [38;5;238m│[0m [38;5;231m## Rollback Decision Criteria[0m
[38;5;238m   7[0m [38;5;238m│[0m 
[38;5;238m   8[0m [38;5;238m│[0m [38;5;231m**Immediate Rollback** (< 5 minutes):[0m
[38;5;238m   9[0m [38;5;238m│[0m [38;5;231m- Error rate > 1% sustained for 5 minutes[0m
[38;5;238m  10[0m [38;5;238m│[0m [38;5;231m- Latency p95 > 2× baseline for 10 minutes[0m
[38;5;238m  11[0m [38;5;238m│[0m [38;5;231m- Database corruption detected[0m
[38;5;238m  12[0m [38;5;238m│[0m [38;5;231m- Security vulnerability discovered[0m
[38;5;238m  13[0m [38;5;238m│[0m 
[38;5;238m  14[0m [38;5;238m│[0m [38;5;231m**Evaluate and Rollback** (5-15 minutes):[0m
[38;5;238m  15[0m [38;5;238m│[0m [38;5;231m- User-reported critical issues (> 10 reports/minute)[0m
[38;5;238m  16[0m [38;5;238m│[0m [38;5;231m- Cache hit rate < 80% for 15 minutes[0m
[38;5;238m  17[0m [38;5;238m│[0m [38;5;231m- Partial feature failure affecting > 10% of users[0m
[38;5;238m  18[0m [38;5;238m│[0m 
[38;5;238m  19[0m [38;5;238m│[0m [38;5;231m## Rollback Procedures[0m
[38;5;238m  20[0m [38;5;238m│[0m 
[38;5;238m  21[0m [38;5;238m│[0m [38;5;231m### Blue-Green Rollback (Instant)[0m
[38;5;238m  22[0m [38;5;238m│[0m 
[38;5;238m  23[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m  24[0m [38;5;238m│[0m [38;5;231m# Switch traffic back to Blue[0m
[38;5;238m  25[0m [38;5;238m│[0m [38;5;231mkubectl patch service thread-service \[0m
[38;5;238m  26[0m [38;5;238m│[0m [38;5;231m    --namespace=production \[0m
[38;5;238m  27[0m [38;5;238m│[0m [38;5;231m    -p '{"spec":{"selector":{"version":"blue"}}}'[0m
[38;5;238m  28[0m [38;5;238m│[0m 
[38;5;238m  29[0m [38;5;238m│[0m [38;5;231m# Verify rollback[0m
[38;5;238m  30[0m [38;5;238m│[0m [38;5;231mkubectl get service thread-service -o jsonpath='{.spec.selector.version}'[0m
[38;5;238m  31[0m [38;5;238m│[0m [38;5;231m# Should output: blue[0m
[38;5;238m  32[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m  33[0m [38;5;238m│[0m 
[38;5;238m  34[0m [38;5;238m│[0m [38;5;231m**Time to Rollback**: < 30 seconds[0m
[38;5;238m  35[0m [38;5;238m│[0m 
[38;5;238m  36[0m [38;5;238m│[0m [38;5;231m### Canary Rollback (Instant)[0m
[38;5;238m  37[0m [38;5;238m│[0m 
[38;5;238m  38[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m  39[0m [38;5;238m│[0m [38;5;231m# Set canary traffic to 0%[0m
[38;5;238m  40[0m [38;5;238m│[0m [38;5;231mkubectl patch virtualservice thread-canary \[0m
[38;5;238m  41[0m [38;5;238m│[0m [38;5;231m    --namespace=production \[0m
[38;5;238m  42[0m [38;5;238m│[0m [38;5;231m    --type merge \[0m
[38;5;238m  43[0m [38;5;238m│[0m [38;5;231m    -p '{"spec":{"http":[{"route":[{"destination":{"host":"thread-service","subset":"stable"},"weight":100}]}]}}'[0m
[38;5;238m  44[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m  45[0m [38;5;238m│[0m 
[38;5;238m  46[0m [38;5;238m│[0m [38;5;231m**Time to Rollback**: < 30 seconds[0m
[38;5;238m  47[0m [38;5;238m│[0m 
[38;5;238m  48[0m [38;5;238m│[0m [38;5;231m### Rolling Update Rollback (Minutes)[0m
[38;5;238m  49[0m [38;5;238m│[0m 
[38;5;238m  50[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m  51[0m [38;5;238m│[0m [38;5;231m# Kubernetes rollback[0m
[38;5;238m  52[0m [38;5;238m│[0m [38;5;231mkubectl rollout undo deployment/thread-worker --namespace=production[0m
[38;5;238m  53[0m [38;5;238m│[0m 
[38;5;238m  54[0m [38;5;238m│[0m [38;5;231m# Monitor rollback progress[0m
[38;5;238m  55[0m [38;5;238m│[0m [38;5;231mkubectl rollout status deployment/thread-worker --namespace=production[0m
[38;5;238m  56[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m  57[0m [38;5;238m│[0m 
[38;5;238m  58[0m [38;5;238m│[0m [38;5;231m**Time to Rollback**: 3-10 minutes (depends on instance count)[0m
[38;5;238m  59[0m [38;5;238m│[0m 
[38;5;238m  60[0m [38;5;238m│[0m [38;5;231m### Edge Rollback (Cloudflare Workers)[0m
[38;5;238m  61[0m [38;5;238m│[0m 
[38;5;238m  62[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m  63[0m [38;5;238m│[0m [38;5;231m# Rollback to previous deployment[0m
[38;5;238m  64[0m [38;5;238m│[0m [38;5;231mwrangler rollback --env production[0m
[38;5;238m  65[0m [38;5;238m│[0m 
[38;5;238m  66[0m [38;5;238m│[0m [38;5;231m# Or deploy specific version[0m
[38;5;238m  67[0m [38;5;238m│[0m [38;5;231mwrangler deploy --env production --version v1.0.0[0m
[38;5;238m  68[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m  69[0m [38;5;238m│[0m 
[38;5;238m  70[0m [38;5;238m│[0m [38;5;231m**Time to Rollback**: < 2 minutes (global propagation)[0m
[38;5;238m  71[0m [38;5;238m│[0m 
[38;5;238m  72[0m [38;5;238m│[0m [38;5;231m## Database Migration Rollback[0m
[38;5;238m  73[0m [38;5;238m│[0m 
[38;5;238m  74[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m  75[0m [38;5;238m│[0m [38;5;231m# Rollback last migration (Diesel)[0m
[38;5;238m  76[0m [38;5;238m│[0m [38;5;231mdiesel migration revert --database-url="$DATABASE_URL"[0m
[38;5;238m  77[0m [38;5;238m│[0m 
[38;5;238m  78[0m [38;5;238m│[0m [38;5;231m# Or manual SQL rollback[0m
[38;5;238m  79[0m [38;5;238m│[0m [38;5;231mpsql "$DATABASE_URL" -f migrations/down.sql[0m
[38;5;238m  80[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m  81[0m [38;5;238m│[0m 
[38;5;238m  82[0m [38;5;238m│[0m [38;5;231m**Important**: Only rollback migrations if backward-compatible. Otherwise, coordinate with code rollback.[0m
[38;5;238m  83[0m [38;5;238m│[0m 
[38;5;238m  84[0m [38;5;238m│[0m [38;5;231m## Disaster Recovery[0m
[38;5;238m  85[0m [38;5;238m│[0m 
[38;5;238m  86[0m [38;5;238m│[0m [38;5;231m### Recovery Time Objectives (RTO/RPO)[0m
[38;5;238m  87[0m [38;5;238m│[0m 
[38;5;238m  88[0m [38;5;238m│[0m [38;5;231m| Component | TO (Time to Recover) | RPO (Data Loss) |[0m
[38;5;238m  89[0m [38;5;238m│[0m [38;5;231m|-----------|----------------------|-----------------|[0m
[38;5;238m  90[0m [38;5;238m│[0m [38;5;231m| **CLI Workers** | 10 minutes | 0 (stateless) |[0m
[38;5;238m  91[0m [38;5;238m│[0m [38;5;231m| **Database** | 30 minutes | < 5 minutes |[0m
[38;5;238m  92[0m [38;5;238m│[0m [38;5;231m| **Edge Workers** | 5 minutes | 0 (stateless) |[0m
[38;5;238m  93[0m [38;5;238m│[0m [38;5;231m| **Cache** | 5 minutes | Acceptable (rebuild) |[0m
[38;5;238m  94[0m [38;5;238m│[0m 
[38;5;238m  95[0m [38;5;238m│[0m [38;5;231m### Database Recovery[0m
[38;5;238m  96[0m [38;5;238m│[0m 
[38;5;238m  97[0m [38;5;238m│[0m [38;5;231m**Automated Backup**:[0m
[38;5;238m  98[0m [38;5;238m│[0m [38;5;231m- Daily full backups (retained 30 days)[0m
[38;5;238m  99[0m [38;5;238m│[0m [38;5;231m- 5-minute incremental backups (point-in-time recovery)[0m
[38;5;238m 100[0m [38;5;238m│[0m 
[38;5;238m 101[0m [38;5;238m│[0m [38;5;231m**Recovery Procedure**:[0m
[38;5;238m 102[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m 103[0m [38;5;238m│[0m [38;5;231m# Restore from snapshot (AWS RDS)[0m
[38;5;238m 104[0m [38;5;238m│[0m [38;5;231maws rds restore-db-instance-to-point-in-time \[0m
[38;5;238m 105[0m [38;5;238m│[0m [38;5;231m    --source-db-instance-identifier thread-prod \[0m
[38;5;238m 106[0m [38;5;238m│[0m [38;5;231m    --target-db-instance-identifier thread-prod-restore \[0m
[38;5;238m 107[0m [38;5;238m│[0m [38;5;231m    --restore-time 2026-01-28T10:00:00Z[0m
[38;5;238m 108[0m [38;5;238m│[0m 
[38;5;238m 109[0m [38;5;238m│[0m [38;5;231m# Or restore from backup[0m
[38;5;238m 110[0m [38;5;238m│[0m [38;5;231maws rds restore-db-instance-from-db-snapshot \[0m
[38;5;238m 111[0m [38;5;238m│[0m [38;5;231m    --db-instance-identifier thread-prod-restore \[0m
[38;5;238m 112[0m [38;5;238m│[0m [38;5;231m    --db-snapshot-identifier thread-prod-snapshot-2026-01-28[0m
[38;5;238m 113[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 114[0m [38;5;238m│[0m 
[38;5;238m 115[0m [38;5;238m│[0m [38;5;231m**Time to Recover**: 20-30 minutes[0m
[38;5;238m 116[0m [38;5;238m│[0m 
[38;5;238m 117[0m [38;5;238m│[0m [38;5;231m### Complete System Recovery[0m
[38;5;238m 118[0m [38;5;238m│[0m 
[38;5;238m 119[0m [38;5;238m│[0m [38;5;231m**Scenario**: Complete infrastructure failure (region outage)[0m
[38;5;238m 120[0m [38;5;238m│[0m 
[38;5;238m 121[0m [38;5;238m│[0m [38;5;231m**Recovery Steps**:[0m
[38;5;238m 122[0m [38;5;238m│[0m [38;5;231m1. Activate DR region (if multi-region)[0m
[38;5;238m 123[0m [38;5;238m│[0m [38;5;231m2. Restore database from backup[0m
[38;5;238m 124[0m [38;5;238m│[0m [38;5;231m3. Deploy latest validated release[0m
[38;5;238m 125[0m [38;5;238m│[0m [38;5;231m4. Update DNS to DR region[0m
[38;5;238m 126[0m [38;5;238m│[0m [38;5;231m5. Validate functionality[0m
[38;5;238m 127[0m [38;5;238m│[0m 
[38;5;238m 128[0m [38;5;238m│[0m [38;5;231m**Recovery Time**: 1-2 hours (including validation)[0m
[38;5;238m 129[0m [38;5;238m│[0m 
[38;5;238m 130[0m [38;5;238m│[0m [38;5;231m## Post-Rollback Actions[0m
[38;5;238m 131[0m [38;5;238m│[0m 
[38;5;238m 132[0m [38;5;238m│[0m [38;5;231m1. **Investigate Root Cause**: Analyze logs, metrics, error reports[0m
[38;5;238m 133[0m [38;5;238m│[0m [38;5;231m2. **Document Incident**: Write incident report with timeline[0m
[38;5;238m 134[0m [38;5;238m│[0m [38;5;231m3. **Update Runbooks**: Add new failure mode to runbooks[0m
[38;5;238m 135[0m [38;5;238m│[0m [38;5;231m4. **Test Fix**: Validate fix in staging before re-deploying[0m
[38;5;238m 136[0m [38;5;238m│[0m [38;5;231m5. **Communicate**: Notify stakeholders of resolution[0m
[38;5;238m 137[0m [38;5;238m│[0m 
[38;5;238m 138[0m [38;5;238m│[0m [38;5;231m## Rollback Validation Checklist[0m
[38;5;238m 139[0m [38;5;238m│[0m 
[38;5;238m 140[0m [38;5;238m│[0m [38;5;231m- [ ] Service health checks passing[0m
[38;5;238m 141[0m [38;5;238m│[0m [38;5;231m- [ ] Error rate < 0.1%[0m
[38;5;238m 142[0m [38;5;238m│[0m [38;5;231m- [ ] Latency p95 < baseline[0m
[38;5;238m 143[0m [38;5;238m│[0m [38;5;231m- [ ] Cache hit rate > 90%[0m
[38;5;238m 144[0m [38;5;238m│[0m [38;5;231m- [ ] No user-reported issues[0m
[38;5;238m 145[0m [38;5;238m│[0m [38;5;231m- [ ] Database queries functioning[0m
[38;5;238m 146[0m [38;5;238m│[0m [38;5;231m- [ ] Monitoring dashboards green[0m
[38;5;238m 147[0m [38;5;238m│[0m 
[38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
