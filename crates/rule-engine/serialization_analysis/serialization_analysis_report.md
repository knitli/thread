<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# SERIALIZATION DEPENDENCY ANALYSIS REPORT

## Executive Summary

- **Total files analyzed**: 9
- **High-impact files**: 4
- **Total serialization dependencies**: 110

## Dependency Categories

- **Core Serialization**: 27 occurrences
- **Crate-Specific Serialization**: 74 occurrences
- **Serialization Operations**: 6 occurrences
- **Schema Generation**: 3 occurrences

## Detailed Dependency Breakdown

- **DeserializeEnvUsage**: 38 (Crate-Specific Serialization)
- **JsonSchemaUsage**: 3 (Schema Generation)
- **DeserializationCall**: 5 (Serialization Operations)
- **SerdeImport**: 15 (Core Serialization)
- **SerdeDerive**: 12 (Core Serialization)
- **SerializationCall**: 1 (Serialization Operations)
- **MaybeWrapper**: 36 (Crate-Specific Serialization)

## High-Impact Files (Difficult to Separate)

### src/check_var.rs

- Serialization density: 2.1%
- Dependencies: 7
- Core functions: 1

### src/fixer.rs

- Serialization density: 7.9%
- Dependencies: 28
- Core functions: 6

### src/rule_core.rs

- Serialization density: 4.8%
- Dependencies: 22
- Core functions: 27

### src/rule_config.rs

- Serialization density: 1.8%
- Dependencies: 14
- Core functions: 34

## SEPARATION STRATEGY

### 1. Feature Gate Candidates (Easy wins)

- `src/combined.rs`
- `src/rule_collection.rs`

### 2. Core Logic Files (Keep in core)

- `src/combined.rs`
- `src/lib.rs`
- `src/label.rs`
- `src/check_var.rs`
- `src/fixer.rs`
- `src/rule_core.rs`
- `src/rule_config.rs`

### 3. Serialization-Only Files (Move to separate module)

### 4. Need Abstraction Layer

- `src/label.rs`

### 5. Mixed Responsibility (Requires Refactoring)

- `src/lib.rs`
- `src/maybe.rs`
- `src/check_var.rs`
- `src/fixer.rs`
- `src/rule_core.rs`
- `src/rule_config.rs`

## RECOMMENDATIONS

1. **Immediate actions**: Feature-gate files with low serialization impact
2. **Short-term**: Create abstraction layer for files needing it
3. **Medium-term**: Refactor mixed responsibility files
4. **Long-term**: Consider trait-based abstraction for core serialization needs
