<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

[38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
     [38;5;238m│ [0m[1mSTDIN[0m
[38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
[38;5;238m   1[0m [38;5;238m│[0m [38;5;231m# Alerting and Notification Configuration[0m
[38;5;238m   2[0m [38;5;238m│[0m 
[38;5;238m   3[0m [38;5;238m│[0m [38;5;231m**Version**: 1.0.0[0m
[38;5;238m   4[0m [38;5;238m│[0m [38;5;231m**Last Updated**: 2026-01-28[0m
[38;5;238m   5[0m [38;5;238m│[0m [38;5;231m**Status**: Production Ready[0m
[38;5;238m   6[0m [38;5;238m│[0m 
[38;5;238m   7[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m   8[0m [38;5;238m│[0m 
[38;5;238m   9[0m [38;5;238m│[0m [38;5;231m## Overview[0m
[38;5;238m  10[0m [38;5;238m│[0m 
[38;5;238m  11[0m [38;5;238m│[0m [38;5;231mComprehensive alerting strategy for Thread production environments with intelligent routing, escalation, and fatigue prevention.[0m
[38;5;238m  12[0m [38;5;238m│[0m 
[38;5;238m  13[0m [38;5;238m│[0m [38;5;231m### Alerting Philosophy[0m
[38;5;238m  14[0m [38;5;238m│[0m 
[38;5;238m  15[0m [38;5;238m│[0m [38;5;231m- **Actionable Alerts Only**: Every alert requires a response action[0m
[38;5;238m  16[0m [38;5;238m│[0m [38;5;231m- **Appropriate Severity**: Critical = immediate action, Warning = investigate soon[0m
[38;5;238m  17[0m [38;5;238m│[0m [38;5;231m- **Clear Context**: Alerts include runbook links and relevant metrics[0m
[38;5;238m  18[0m [38;5;238m│[0m [38;5;231m- **Escalation Paths**: Clear escalation for unacknowledged critical alerts[0m
[38;5;238m  19[0m [38;5;238m│[0m 
[38;5;238m  20[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m  21[0m [38;5;238m│[0m 
[38;5;238m  22[0m [38;5;238m│[0m [38;5;231m## Alert Routing[0m
[38;5;238m  23[0m [38;5;238m│[0m 
[38;5;238m  24[0m [38;5;238m│[0m [38;5;231m### Severity-Based Routing[0m
[38;5;238m  25[0m [38;5;238m│[0m 
[38;5;238m  26[0m [38;5;238m│[0m [38;5;231m| Severity | Destination | Response Time | Escalation |[0m
[38;5;238m  27[0m [38;5;238m│[0m [38;5;231m|----------|-------------|---------------|------------|[0m
[38;5;238m  28[0m [38;5;238m│[0m [38;5;231m| **Critical** | PagerDuty + Slack #incidents | 15 minutes | Manager after 30 min |[0m
[38;5;238m  29[0m [38;5;238m│[0m [38;5;231m| **Warning** | Slack #alerts | 2 hours | None |[0m
[38;5;238m  30[0m [38;5;238m│[0m [38;5;231m| **Info** | Slack #monitoring | Next business day | None |[0m
[38;5;238m  31[0m [38;5;238m│[0m 
[38;5;238m  32[0m [38;5;238m│[0m [38;5;231m### Alertmanager Configuration[0m
[38;5;238m  33[0m [38;5;238m│[0m 
[38;5;238m  34[0m [38;5;238m│[0m [38;5;231m**Main Config** (`alertmanager.yml`):[0m
[38;5;238m  35[0m [38;5;238m│[0m [38;5;231m```yaml[0m
[38;5;238m  36[0m [38;5;238m│[0m [38;5;231mglobal:[0m
[38;5;238m  37[0m [38;5;238m│[0m [38;5;231m  resolve_timeout: 5m[0m
[38;5;238m  38[0m [38;5;238m│[0m [38;5;231m  slack_api_url: '${SLACK_WEBHOOK_URL}'[0m
[38;5;238m  39[0m [38;5;238m│[0m [38;5;231m  pagerduty_url: 'https://events.pagerduty.com/v2/enqueue'[0m
[38;5;238m  40[0m [38;5;238m│[0m 
[38;5;238m  41[0m [38;5;238m│[0m [38;5;231m# Routing tree[0m
[38;5;238m  42[0m [38;5;238m│[0m [38;5;231mroute:[0m
[38;5;238m  43[0m [38;5;238m│[0m [38;5;231m  receiver: 'default'[0m
[38;5;238m  44[0m [38;5;238m│[0m [38;5;231m  group_by: ['alertname', 'severity'][0m
[38;5;238m  45[0m [38;5;238m│[0m [38;5;231m  group_wait: 30s[0m
[38;5;238m  46[0m [38;5;238m│[0m [38;5;231m  group_interval: 5m[0m
[38;5;238m  47[0m [38;5;238m│[0m [38;5;231m  repeat_interval: 4h[0m
[38;5;238m  48[0m [38;5;238m│[0m 
[38;5;238m  49[0m [38;5;238m│[0m [38;5;231m  routes:[0m
[38;5;238m  50[0m [38;5;238m│[0m [38;5;231m    # Critical alerts → PagerDuty + Slack[0m
[38;5;238m  51[0m [38;5;238m│[0m [38;5;231m    - match:[0m
[38;5;238m  52[0m [38;5;238m│[0m [38;5;231m        severity: critical[0m
[38;5;238m  53[0m [38;5;238m│[0m [38;5;231m      receiver: pagerduty-critical[0m
[38;5;238m  54[0m [38;5;238m│[0m [38;5;231m      group_wait: 10s[0m
[38;5;238m  55[0m [38;5;238m│[0m [38;5;231m      repeat_interval: 15m[0m
[38;5;238m  56[0m [38;5;238m│[0m [38;5;231m      continue: true[0m
[38;5;238m  57[0m [38;5;238m│[0m 
[38;5;238m  58[0m [38;5;238m│[0m [38;5;231m    - match:[0m
[38;5;238m  59[0m [38;5;238m│[0m [38;5;231m        severity: critical[0m
[38;5;238m  60[0m [38;5;238m│[0m [38;5;231m      receiver: slack-incidents[0m
[38;5;238m  61[0m [38;5;238m│[0m 
[38;5;238m  62[0m [38;5;238m│[0m [38;5;231m    # Warning alerts → Slack only[0m
[38;5;238m  63[0m [38;5;238m│[0m [38;5;231m    - match:[0m
[38;5;238m  64[0m [38;5;238m│[0m [38;5;231m        severity: warning[0m
[38;5;238m  65[0m [38;5;238m│[0m [38;5;231m      receiver: slack-warnings[0m
[38;5;238m  66[0m [38;5;238m│[0m [38;5;231m      group_wait: 5m[0m
[38;5;238m  67[0m [38;5;238m│[0m [38;5;231m      repeat_interval: 12h[0m
[38;5;238m  68[0m [38;5;238m│[0m 
[38;5;238m  69[0m [38;5;238m│[0m [38;5;231m    # Info alerts → Slack monitoring channel[0m
[38;5;238m  70[0m [38;5;238m│[0m [38;5;231m    - match:[0m
[38;5;238m  71[0m [38;5;238m│[0m [38;5;231m        severity: info[0m
[38;5;238m  72[0m [38;5;238m│[0m [38;5;231m      receiver: slack-monitoring[0m
[38;5;238m  73[0m [38;5;238m│[0m [38;5;231m      repeat_interval: 24h[0m
[38;5;238m  74[0m [38;5;238m│[0m 
[38;5;238m  75[0m [38;5;238m│[0m [38;5;231mreceivers:[0m
[38;5;238m  76[0m [38;5;238m│[0m [38;5;231m  - name: 'default'[0m
[38;5;238m  77[0m [38;5;238m│[0m [38;5;231m    slack_configs:[0m
[38;5;238m  78[0m [38;5;238m│[0m [38;5;231m      - channel: '#alerts'[0m
[38;5;238m  79[0m [38;5;238m│[0m [38;5;231m        title: 'Thread Alert'[0m
[38;5;238m  80[0m [38;5;238m│[0m [38;5;231m        text: '{{ .CommonAnnotations.summary }}'[0m
[38;5;238m  81[0m [38;5;238m│[0m 
[38;5;238m  82[0m [38;5;238m│[0m [38;5;231m  - name: 'pagerduty-critical'[0m
[38;5;238m  83[0m [38;5;238m│[0m [38;5;231m    pagerduty_configs:[0m
[38;5;238m  84[0m [38;5;238m│[0m [38;5;231m      - service_key: '${PAGERDUTY_SERVICE_KEY}'[0m
[38;5;238m  85[0m [38;5;238m│[0m [38;5;231m        description: '{{ .CommonAnnotations.summary }}'[0m
[38;5;238m  86[0m [38;5;238m│[0m [38;5;231m        client: 'Thread Monitoring'[0m
[38;5;238m  87[0m [38;5;238m│[0m [38;5;231m        client_url: '{{ .CommonAnnotations.runbook_url }}'[0m
[38;5;238m  88[0m [38;5;238m│[0m [38;5;231m        details:[0m
[38;5;238m  89[0m [38;5;238m│[0m [38;5;231m          severity: '{{ .CommonLabels.severity }}'[0m
[38;5;238m  90[0m [38;5;238m│[0m [38;5;231m          environment: '{{ .CommonLabels.environment }}'[0m
[38;5;238m  91[0m [38;5;238m│[0m [38;5;231m          firing_alerts: '{{ .Alerts.Firing | len }}'[0m
[38;5;238m  92[0m [38;5;238m│[0m 
[38;5;238m  93[0m [38;5;238m│[0m [38;5;231m  - name: 'slack-incidents'[0m
[38;5;238m  94[0m [38;5;238m│[0m [38;5;231m    slack_configs:[0m
[38;5;238m  95[0m [38;5;238m│[0m [38;5;231m      - channel: '#incidents'[0m
[38;5;238m  96[0m [38;5;238m│[0m [38;5;231m        title: '🚨 CRITICAL: {{ .CommonAnnotations.summary }}'[0m
[38;5;238m  97[0m [38;5;238m│[0m [38;5;231m        text: |[0m
[38;5;238m  98[0m [38;5;238m│[0m [38;5;231m          *Environment*: {{ .CommonLabels.environment }}[0m
[38;5;238m  99[0m [38;5;238m│[0m [38;5;231m          *Alerts Firing*: {{ .Alerts.Firing | len }}[0m
[38;5;238m 100[0m [38;5;238m│[0m [38;5;231m          [0m
[38;5;238m 101[0m [38;5;238m│[0m [38;5;231m          {{ range .Alerts }}[0m
[38;5;238m 102[0m [38;5;238m│[0m [38;5;231m          *Alert*: {{ .Labels.alertname }}[0m
[38;5;238m 103[0m [38;5;238m│[0m [38;5;231m          *Description*: {{ .Annotations.description }}[0m
[38;5;238m 104[0m [38;5;238m│[0m [38;5;231m          *Runbook*: {{ .Annotations.runbook_url }}[0m
[38;5;238m 105[0m [38;5;238m│[0m [38;5;231m          {{ end }}[0m
[38;5;238m 106[0m [38;5;238m│[0m [38;5;231m        actions:[0m
[38;5;238m 107[0m [38;5;238m│[0m [38;5;231m          - type: button[0m
[38;5;238m 108[0m [38;5;238m│[0m [38;5;231m            text: 'Acknowledge'[0m
[38;5;238m 109[0m [38;5;238m│[0m [38;5;231m            url: '{{ .ExternalURL }}/#/alerts'[0m
[38;5;238m 110[0m [38;5;238m│[0m [38;5;231m          - type: button[0m
[38;5;238m 111[0m [38;5;238m│[0m [38;5;231m            text: 'Runbook'[0m
[38;5;238m 112[0m [38;5;238m│[0m [38;5;231m            url: '{{ .CommonAnnotations.runbook_url }}'[0m
[38;5;238m 113[0m [38;5;238m│[0m [38;5;231m        color: danger[0m
[38;5;238m 114[0m [38;5;238m│[0m 
[38;5;238m 115[0m [38;5;238m│[0m [38;5;231m  - name: 'slack-warnings'[0m
[38;5;238m 116[0m [38;5;238m│[0m [38;5;231m    slack_configs:[0m
[38;5;238m 117[0m [38;5;238m│[0m [38;5;231m      - channel: '#alerts'[0m
[38;5;238m 118[0m [38;5;238m│[0m [38;5;231m        title: '⚠️  WARNING: {{ .CommonAnnotations.summary }}'[0m
[38;5;238m 119[0m [38;5;238m│[0m [38;5;231m        text: |[0m
[38;5;238m 120[0m [38;5;238m│[0m [38;5;231m          {{ range .Alerts }}[0m
[38;5;238m 121[0m [38;5;238m│[0m [38;5;231m          *Alert*: {{ .Labels.alertname }}[0m
[38;5;238m 122[0m [38;5;238m│[0m [38;5;231m          *Description*: {{ .Annotations.description }}[0m
[38;5;238m 123[0m [38;5;238m│[0m [38;5;231m          {{ end }}[0m
[38;5;238m 124[0m [38;5;238m│[0m [38;5;231m        color: warning[0m
[38;5;238m 125[0m [38;5;238m│[0m 
[38;5;238m 126[0m [38;5;238m│[0m [38;5;231m  - name: 'slack-monitoring'[0m
[38;5;238m 127[0m [38;5;238m│[0m [38;5;231m    slack_configs:[0m
[38;5;238m 128[0m [38;5;238m│[0m [38;5;231m      - channel: '#monitoring'[0m
[38;5;238m 129[0m [38;5;238m│[0m [38;5;231m        title: 'ℹ️  Info: {{ .CommonAnnotations.summary }}'[0m
[38;5;238m 130[0m [38;5;238m│[0m [38;5;231m        text: '{{ .CommonAnnotations.description }}'[0m
[38;5;238m 131[0m [38;5;238m│[0m [38;5;231m        color: '#439FE0'[0m
[38;5;238m 132[0m [38;5;238m│[0m 
[38;5;238m 133[0m [38;5;238m│[0m [38;5;231m# Inhibition rules[0m
[38;5;238m 134[0m [38;5;238m│[0m [38;5;231minhibit_rules:[0m
[38;5;238m 135[0m [38;5;238m│[0m [38;5;231m  # If service is down, suppress latency/error alerts[0m
[38;5;238m 136[0m [38;5;238m│[0m [38;5;231m  - source_match:[0m
[38;5;238m 137[0m [38;5;238m│[0m [38;5;231m      alertname: 'ServiceDown'[0m
[38;5;238m 138[0m [38;5;238m│[0m [38;5;231m    target_match_re:[0m
[38;5;238m 139[0m [38;5;238m│[0m [38;5;231m      alertname: '.*Latency.*|.*ErrorRate.*'[0m
[38;5;238m 140[0m [38;5;238m│[0m [38;5;231m    equal: ['instance'][0m
[38;5;238m 141[0m [38;5;238m│[0m 
[38;5;238m 142[0m [38;5;238m│[0m [38;5;231m  # If database is down, suppress query alerts[0m
[38;5;238m 143[0m [38;5;238m│[0m [38;5;231m  - source_match:[0m
[38;5;238m 144[0m [38;5;238m│[0m [38;5;231m      alertname: 'DatabaseDown'[0m
[38;5;238m 145[0m [38;5;238m│[0m [38;5;231m    target_match_re:[0m
[38;5;238m 146[0m [38;5;238m│[0m [38;5;231m      alertname: '.*Query.*|.*Connection.*'[0m
[38;5;238m 147[0m [38;5;238m│[0m [38;5;231m    equal: ['environment'][0m
[38;5;238m 148[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 149[0m [38;5;238m│[0m 
[38;5;238m 150[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m 151[0m [38;5;238m│[0m 
[38;5;238m 152[0m [38;5;238m│[0m [38;5;231m## On-Call Rotation[0m
[38;5;238m 153[0m [38;5;238m│[0m 
[38;5;238m 154[0m [38;5;238m│[0m [38;5;231m### PagerDuty Schedule[0m
[38;5;238m 155[0m [38;5;238m│[0m 
[38;5;238m 156[0m [38;5;238m│[0m [38;5;231m**Primary On-Call**:[0m
[38;5;238m 157[0m [38;5;238m│[0m [38;5;231m- Weekly rotation (Monday 9am - Monday 9am)[0m
[38;5;238m 158[0m [38;5;238m│[0m [38;5;231m- 2 engineers per week (primary + backup)[0m
[38;5;238m 159[0m [38;5;238m│[0m [38;5;231m- Automatic escalation to backup after 15 minutes[0m
[38;5;238m 160[0m [38;5;238m│[0m 
[38;5;238m 161[0m [38;5;238m│[0m [38;5;231m**Schedule Configuration** (`pagerduty-schedule.json`):[0m
[38;5;238m 162[0m [38;5;238m│[0m [38;5;231m```json[0m
[38;5;238m 163[0m [38;5;238m│[0m [38;5;231m{[0m
[38;5;238m 164[0m [38;5;238m│[0m [38;5;231m  "schedule": {[0m
[38;5;238m 165[0m [38;5;238m│[0m [38;5;231m    "type": "schedule",[0m
[38;5;238m 166[0m [38;5;238m│[0m [38;5;231m    "name": "Thread Primary On-Call",[0m
[38;5;238m 167[0m [38;5;238m│[0m [38;5;231m    "time_zone": "America/New_York",[0m
[38;5;238m 168[0m [38;5;238m│[0m [38;5;231m    "schedule_layers": [[0m
[38;5;238m 169[0m [38;5;238m│[0m [38;5;231m      {[0m
[38;5;238m 170[0m [38;5;238m│[0m [38;5;231m        "name": "Weekly Rotation",[0m
[38;5;238m 171[0m [38;5;238m│[0m [38;5;231m        "start": "2024-01-01T09:00:00",[0m
[38;5;238m 172[0m [38;5;238m│[0m [38;5;231m        "rotation_virtual_start": "2024-01-01T09:00:00",[0m
[38;5;238m 173[0m [38;5;238m│[0m [38;5;231m        "rotation_turn_length_seconds": 604800,[0m
[38;5;238m 174[0m [38;5;238m│[0m [38;5;231m        "users": [[0m
[38;5;238m 175[0m [38;5;238m│[0m [38;5;231m          {"user": {"id": "USER1"}},[0m
[38;5;238m 176[0m [38;5;238m│[0m [38;5;231m          {"user": {"id": "USER2"}},[0m
[38;5;238m 177[0m [38;5;238m│[0m [38;5;231m          {"user": {"id": "USER3"}}[0m
[38;5;238m 178[0m [38;5;238m│[0m [38;5;231m        ],[0m
[38;5;238m 179[0m [38;5;238m│[0m [38;5;231m        "restrictions": [][0m
[38;5;238m 180[0m [38;5;238m│[0m [38;5;231m      }[0m
[38;5;238m 181[0m [38;5;238m│[0m [38;5;231m    ][0m
[38;5;238m 182[0m [38;5;238m│[0m [38;5;231m  }[0m
[38;5;238m 183[0m [38;5;238m│[0m [38;5;231m}[0m
[38;5;238m 184[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 185[0m [38;5;238m│[0m 
[38;5;238m 186[0m [38;5;238m│[0m [38;5;231m### Escalation Policy[0m
[38;5;238m 187[0m [38;5;238m│[0m 
[38;5;238m 188[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 189[0m [38;5;238m│[0m [38;5;231mAlert Triggered[0m
[38;5;238m 190[0m [38;5;238m│[0m [38;5;231m    ↓[0m
[38;5;238m 191[0m [38;5;238m│[0m [38;5;231mPrimary On-Call (15 min timeout)[0m
[38;5;238m 192[0m [38;5;238m│[0m [38;5;231m    ↓ (no acknowledgement)[0m
[38;5;238m 193[0m [38;5;238m│[0m [38;5;231mBackup On-Call (15 min timeout)[0m
[38;5;238m 194[0m [38;5;238m│[0m [38;5;231m    ↓ (no acknowledgement)[0m
[38;5;238m 195[0m [38;5;238m│[0m [38;5;231mEngineering Manager[0m
[38;5;238m 196[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 197[0m [38;5;238m│[0m 
[38;5;238m 198[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m 199[0m [38;5;238m│[0m 
[38;5;238m 200[0m [38;5;238m│[0m [38;5;231m## Alert Fatigue Prevention[0m
[38;5;238m 201[0m [38;5;238m│[0m 
[38;5;238m 202[0m [38;5;238m│[0m [38;5;231m### Alert Tuning[0m
[38;5;238m 203[0m [38;5;238m│[0m 
[38;5;238m 204[0m [38;5;238m│[0m [38;5;231m**Monthly Review Process**:[0m
[38;5;238m 205[0m [38;5;238m│[0m [38;5;231m1. Identify alerts with > 10 occurrences/week[0m
[38;5;238m 206[0m [38;5;238m│[0m [38;5;231m2. Analyze: Is alert actionable? Is threshold appropriate?[0m
[38;5;238m 207[0m [38;5;238m│[0m [38;5;231m3. Adjust threshold OR suppress alert OR fix underlying issue[0m
[38;5;238m 208[0m [38;5;238m│[0m 
[38;5;238m 209[0m [38;5;238m│[0m [38;5;231m**Common Adjustments**:[0m
[38;5;238m 210[0m [38;5;238m│[0m [38;5;231m```yaml[0m
[38;5;238m 211[0m [38;5;238m│[0m [38;5;231m# Before: Too sensitive (fires on normal spikes)[0m
[38;5;238m 212[0m [38;5;238m│[0m [38;5;231m- alert: HighCPU[0m
[38;5;238m 213[0m [38;5;238m│[0m [38;5;231m  expr: node_cpu_usage > 60[0m
[38;5;238m 214[0m [38;5;238m│[0m 
[38;5;238m 215[0m [38;5;238m│[0m [38;5;231m# After: Account for normal variance[0m
[38;5;238m 216[0m [38;5;238m│[0m [38;5;231m- alert: HighCPU[0m
[38;5;238m 217[0m [38;5;238m│[0m [38;5;231m  expr: node_cpu_usage > 80[0m
[38;5;238m 218[0m [38;5;238m│[0m [38;5;231m  for: 15m  # Sustained high CPU[0m
[38;5;238m 219[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 220[0m [38;5;238m│[0m 
[38;5;238m 221[0m [38;5;238m│[0m [38;5;231m### Alert Grouping[0m
[38;5;238m 222[0m [38;5;238m│[0m 
[38;5;238m 223[0m [38;5;238m│[0m [38;5;231m**Group Related Alerts**:[0m
[38;5;238m 224[0m [38;5;238m│[0m [38;5;231m```yaml[0m
[38;5;238m 225[0m [38;5;238m│[0m [38;5;231m# Group by service and severity[0m
[38;5;238m 226[0m [38;5;238m│[0m [38;5;231mroute:[0m
[38;5;238m 227[0m [38;5;238m│[0m [38;5;231m  group_by: ['service', 'severity'][0m
[38;5;238m 228[0m [38;5;238m│[0m [38;5;231m  group_wait: 30s[0m
[38;5;238m 229[0m [38;5;238m│[0m [38;5;231m  group_interval: 5m[0m
[38;5;238m 230[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 231[0m [38;5;238m│[0m 
[38;5;238m 232[0m [38;5;238m│[0m [38;5;231m### Silence Patterns[0m
[38;5;238m 233[0m [38;5;238m│[0m 
[38;5;238m 234[0m [38;5;238m│[0m [38;5;231m**Planned Maintenance**:[0m
[38;5;238m 235[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m 236[0m [38;5;238m│[0m [38;5;231m# Silence alerts during deployment window[0m
[38;5;238m 237[0m [38;5;238m│[0m [38;5;231mamtool silence add \[0m
[38;5;238m 238[0m [38;5;238m│[0m [38;5;231m  alertname=~".*" \[0m
[38;5;238m 239[0m [38;5;238m│[0m [38;5;231m  environment=production \[0m
[38;5;238m 240[0m [38;5;238m│[0m [38;5;231m  --start="2024-01-15T02:00:00Z" \[0m
[38;5;238m 241[0m [38;5;238m│[0m [38;5;231m  --end="2024-01-15T04:00:00Z" \[0m
[38;5;238m 242[0m [38;5;238m│[0m [38;5;231m  --author="ops-team" \[0m
[38;5;238m 243[0m [38;5;238m│[0m [38;5;231m  --comment="Planned deployment: v1.2.3"[0m
[38;5;238m 244[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 245[0m [38;5;238m│[0m 
[38;5;238m 246[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m 247[0m [38;5;238m│[0m 
[38;5;238m 248[0m [38;5;238m│[0m [38;5;231m## Alert Templates[0m
[38;5;238m 249[0m [38;5;238m│[0m 
[38;5;238m 250[0m [38;5;238m│[0m [38;5;231m### Critical Alert Template[0m
[38;5;238m 251[0m [38;5;238m│[0m 
[38;5;238m 252[0m [38;5;238m│[0m [38;5;231m```yaml[0m
[38;5;238m 253[0m [38;5;238m│[0m [38;5;231m- alert: [AlertName][0m
[38;5;238m 254[0m [38;5;238m│[0m [38;5;231m  expr: [PromQL expression][0m
[38;5;238m 255[0m [38;5;238m│[0m [38;5;231m  for: [Duration][0m
[38;5;238m 256[0m [38;5;238m│[0m [38;5;231m  labels:[0m
[38;5;238m 257[0m [38;5;238m│[0m [38;5;231m    severity: critical[0m
[38;5;238m 258[0m [38;5;238m│[0m [38;5;231m    team: thread[0m
[38;5;238m 259[0m [38;5;238m│[0m [38;5;231m    environment: production[0m
[38;5;238m 260[0m [38;5;238m│[0m [38;5;231m  annotations:[0m
[38;5;238m 261[0m [38;5;238m│[0m [38;5;231m    summary: "[Brief description]"[0m
[38;5;238m 262[0m [38;5;238m│[0m [38;5;231m    description: "[Detailed description with values: {{ $value }}]"[0m
[38;5;238m 263[0m [38;5;238m│[0m [38;5;231m    impact: "[User/business impact]"[0m
[38;5;238m 264[0m [38;5;238m│[0m [38;5;231m    runbook_url: "https://docs.thread.io/runbooks/[alert-name]"[0m
[38;5;238m 265[0m [38;5;238m│[0m [38;5;231m    dashboard_url: "https://grafana.thread.io/d/[dashboard-id]"[0m
[38;5;238m 266[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 267[0m [38;5;238m│[0m 
[38;5;238m 268[0m [38;5;238m│[0m [38;5;231m### Warning Alert Template[0m
[38;5;238m 269[0m [38;5;238m│[0m 
[38;5;238m 270[0m [38;5;238m│[0m [38;5;231m```yaml[0m
[38;5;238m 271[0m [38;5;238m│[0m [38;5;231m- alert: [AlertName][0m
[38;5;238m 272[0m [38;5;238m│[0m [38;5;231m  expr: [PromQL expression][0m
[38;5;238m 273[0m [38;5;238m│[0m [38;5;231m  for: [Duration][0m
[38;5;238m 274[0m [38;5;238m│[0m [38;5;231m  labels:[0m
[38;5;238m 275[0m [38;5;238m│[0m [38;5;231m    severity: warning[0m
[38;5;238m 276[0m [38;5;238m│[0m [38;5;231m    team: thread[0m
[38;5;238m 277[0m [38;5;238m│[0m [38;5;231m    environment: production[0m
[38;5;238m 278[0m [38;5;238m│[0m [38;5;231m  annotations:[0m
[38;5;238m 279[0m [38;5;238m│[0m [38;5;231m    summary: "[Brief description]"[0m
[38;5;238m 280[0m [38;5;238m│[0m [38;5;231m    description: "[What to investigate]"[0m
[38;5;238m 281[0m [38;5;238m│[0m [38;5;231m    runbook_url: "https://docs.thread.io/runbooks/[alert-name]"[0m
[38;5;238m 282[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 283[0m [38;5;238m│[0m 
[38;5;238m 284[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m 285[0m [38;5;238m│[0m 
[38;5;238m 286[0m [38;5;238m│[0m [38;5;231m## Alert Testing[0m
[38;5;238m 287[0m [38;5;238m│[0m 
[38;5;238m 288[0m [38;5;238m│[0m [38;5;231m### Test Alert Workflow[0m
[38;5;238m 289[0m [38;5;238m│[0m 
[38;5;238m 290[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m 291[0m [38;5;238m│[0m [38;5;231m# Send test alert to Alertmanager[0m
[38;5;238m 292[0m [38;5;238m│[0m [38;5;231mamtool alert add \[0m
[38;5;238m 293[0m [38;5;238m│[0m [38;5;231m  alertname=TestAlert \[0m
[38;5;238m 294[0m [38;5;238m│[0m [38;5;231m  severity=warning \[0m
[38;5;238m 295[0m [38;5;238m│[0m [38;5;231m  instance=test-instance \[0m
[38;5;238m 296[0m [38;5;238m│[0m [38;5;231m  summary="Test alert" \[0m
[38;5;238m 297[0m [38;5;238m│[0m [38;5;231m  --annotation=runbook_url="https://example.com" \[0m
[38;5;238m 298[0m [38;5;238m│[0m [38;5;231m  --end=1h[0m
[38;5;238m 299[0m [38;5;238m│[0m 
[38;5;238m 300[0m [38;5;238m│[0m [38;5;231m# Verify routing[0m
[38;5;238m 301[0m [38;5;238m│[0m [38;5;231mamtool alert query alertname=TestAlert[0m
[38;5;238m 302[0m [38;5;238m│[0m 
[38;5;238m 303[0m [38;5;238m│[0m [38;5;231m# Check Slack/PagerDuty received notification[0m
[38;5;238m 304[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m 305[0m [38;5;238m│[0m 
[38;5;238m 306[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m 307[0m [38;5;238m│[0m 
[38;5;238m 308[0m [38;5;238m│[0m [38;5;231m## Best Practices[0m
[38;5;238m 309[0m [38;5;238m│[0m 
[38;5;238m 310[0m [38;5;238m│[0m [38;5;231m1. **Every Alert Needs a Runbook**: Document response procedure[0m
[38;5;238m 311[0m [38;5;238m│[0m [38;5;231m2. **Tune Regularly**: Review alert frequency monthly[0m
[38;5;238m 312[0m [38;5;238m│[0m [38;5;231m3. **Test Escalation**: Quarterly escalation policy drills[0m
[38;5;238m 313[0m [38;5;238m│[0m [38;5;231m4. **Clear Ownership**: Every alert has responsible team[0m
[38;5;238m 314[0m [38;5;238m│[0m [38;5;231m5. **Avoid Alert Fatigue**: < 5 alerts/week per engineer[0m
[38;5;238m 315[0m [38;5;238m│[0m 
[38;5;238m 316[0m [38;5;238m│[0m [38;5;231m---[0m
[38;5;238m 317[0m [38;5;238m│[0m 
[38;5;238m 318[0m [38;5;238m│[0m [38;5;231m**Document Version**: 1.0.0[0m
[38;5;238m 319[0m [38;5;238m│[0m [38;5;231m**Last Updated**: 2026-01-28[0m
[38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
