<!--
SPDX-FileCopyrightText: 2026 Knitli Inc.

SPDX-License-Identifier: MIT OR Apache-2.0
-->

[38;5;238m─────┬──────────────────────────────────────────────────────────────────────────[0m
     [38;5;238m│ [0m[1mSTDIN[0m
[38;5;238m─────┼──────────────────────────────────────────────────────────────────────────[0m
[38;5;238m   1[0m [38;5;238m│[0m [38;5;231m# Secrets Management Guide[0m
[38;5;238m   2[0m [38;5;238m│[0m 
[38;5;238m   3[0m [38;5;238m│[0m [38;5;231m**Version**: 1.0.0[0m
[38;5;238m   4[0m [38;5;238m│[0m [38;5;231m**Last Updated**: 2026-01-28[0m
[38;5;238m   5[0m [38;5;238m│[0m 
[38;5;238m   6[0m [38;5;238m│[0m [38;5;231m## Tools and Services[0m
[38;5;238m   7[0m [38;5;238m│[0m 
[38;5;238m   8[0m [38;5;238m│[0m [38;5;231m**AWS Secrets Manager** (CLI/Kubernetes): Centralized secrets with rotation[0m
[38;5;238m   9[0m [38;5;238m│[0m [38;5;231m**GitHub Secrets** (Edge): Encrypted CI/CD secrets[0m
[38;5;238m  10[0m [38;5;238m│[0m [38;5;231m**HashCorp Vault** (Enterprise): Advanced secrets management[0m
[38;5;238m  11[0m [38;5;238m│[0m 
[38;5;238m  12[0m [38;5;238m│[0m [38;5;231m## Best Practices[0m
[38;5;238m  13[0m [38;5;238m│[0m 
[38;5;238m  14[0m [38;5;238m│[0m [38;5;231m1. **Never Commit Secrets**: Use `.gitignore` for `.env` files[0m
[38;5;238m  15[0m [38;5;238m│[0m [38;5;231m2. **Rotate Regularly**: Database passwords every 90 days, API keys every 180 days[0m
[38;5;238m  16[0m [38;5;238m│[0m [38;5;231m3. **Least Privilege**: Grant minimal necessary access[0m
[38;5;238m  17[0m [38;5;238m│[0m [38;5;231m4. **Audit Access**: Log all secret retrievals[0m
[38;5;238m  18[0m [38;5;238m│[0m 
[38;5;238m  19[0m [38;5;238m│[0m [38;5;231m## CLI Secrets (AWS Secrets Manager)[0m
[38;5;238m  20[0m [38;5;238m│[0m 
[38;5;238m  21[0m [38;5;238m│[0m [38;5;231m```bash[0m
[38;5;238m  22[0m [38;5;238m│[0m [38;5;231m# Store secret[0m
[38;5;238m  23[0m [38;5;238m│[0m [38;5;231maws secretsmanager create-secret \[0m
[38;5;238m  24[0m [38;5;238m│[0m [38;5;231m    --name thread/production/database \[0m
[38;5;238m  25[0m [38;5;238m│[0m [38;5;231m    --secret-string '{"url":"postgresql://..."}'[0m
[38;5;238m  26[0m [38;5;238m│[0m 
[38;5;238m  27[0m [38;5;238m│[0m [38;5;231m# Retrieve secret[0m
[38;5;238m  28[0m [38;5;238m│[0m [38;5;231maws secretsmanager get-secret-value \[0m
[38;5;238m  29[0m [38;5;238m│[0m [38;5;231m    --secret-id thread/production/database \[0m
[38;5;238m  30[0m [38;5;238m│[0m [38;5;231m    --query SecretString --output text[0m
[38;5;238m  31[0m [38;5;238m│[0m [38;5;231m```[0m
[38;5;238m  32[0m [38;5;238m│[0m 
[38;5;238m  33[0m [38;5;238m│[0m [38;5;231m## Edge Secrets (GitHub Secrets)[0m
[38;5;238m  34[0m [38;5;238m│[0m 
[38;5;238m  35[0m [38;5;238m│[0m [38;5;231mNavigate to repository Settings → Secrets → Actions:[0m
[38;5;238m  36[0m [38;5;238m│[0m [38;5;231m- `CLOUDFLARE_API_TOKEN`: Cloudflare Workers deployment[0m
[38;5;238m  37[0m [38;5;238m│[0m [38;5;231m- `DATABASE_URL`: Production database connection[0m
[38;5;238m  38[0m [38;5;238m│[0m [38;5;231m- `SECRET_KEY`: Application secret key[0m
[38;5;238m  39[0m [38;5;238m│[0m 
[38;5;238m  40[0m [38;5;238m│[0m [38;5;231m## Production Checklist[0m
[38;5;238m  41[0m [38;5;238m│[0m 
[38;5;238m  42[0m [38;5;238m│[0m [38;5;231m- [ ] All secrets in AWS Secrets Manager (not environment variables)[0m
[38;5;238m  43[0m [38;5;238m│[0m [38;5;231m- [ ] IAM roles restrict secret access (not IAM users)[0m
[38;5;238m  44[0m [38;5;238m│[0m [38;5;231m- [ ] Rotation schedule configured (90-day database, 180-day API keys)[0m
[38;5;238m  45[0m [38;5;238m│[0m [38;5;231m- [ ] Audit logging enabled[0m
[38;5;238m  46[0m [38;5;238m│[0m [38;5;231m- [ ] Emergency access procedure documented[0m
[38;5;238m─────┴──────────────────────────────────────────────────────────────────────────[0m
