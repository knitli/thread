<!--
SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
SPDX-FileContributor: Adam Poulemanos <adam@knit.li>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Serialization Dependency Analysis Tools

This directory contains a comprehensive analysis of serialization dependencies in the `thread-rule-engine` crate and tools to help with the separation effort.

## 📁 Files Created

### 🔍 Analysis Tools

1. **[`serialization_analysis.yml`](./serialization_analysis.yml)**
   - AST-Grep rules to systematically find serialization dependencies
   - 11 different rule patterns covering all aspects of serialization usage
   - Can be used with the `ast-grep` CLI tool for detailed code analysis

2. **[`analyze_serialization.rs`](./analyze_serialization.rs)**
   - Rust tool for comprehensive dependency analysis
   - Categorizes dependencies by type and severity
   - Generates separation strategy recommendations
   - Can be compiled and run for detailed reporting

### 📊 Reports

3. **[`SERIALIZATION_ANALYSIS_REPORT.md`](./SERIALIZATION_ANALYSIS_REPORT.md)**
   - **Comprehensive analysis report** with findings and recommendations
   - Identifies high-impact files and separation challenges
   - Provides 4-phase separation strategy with effort estimates
   - **START HERE** for understanding the scope of the problem

### 🛠️ Helper Tools

4. **[`separation_helper.sh`](./separation_helper.sh)** ✅ *executable*
   - Interactive shell script with multiple analysis functions
   - Validates current state and identifies feature gate candidates
   - Creates patches for feature gating
   - Generates separation roadmaps and checklists

## 🚀 Quick Start

### 1. Read the Analysis Report

```bash
# Start with the comprehensive analysis
cat SERIALIZATION_ANALYSIS_REPORT.md
```

### 2. Run the Interactive Helper

```bash
# Run the helper script for guided analysis
./separation_helper.sh
```

### 3. Use AST-Grep for Detailed Analysis

```bash
# Install ast-grep if not available
npm install -g @ast-grep/cli

# Run specific serialization analysis
ast-grep --config serialization_analysis.yml scan ../src/
```

### 4. Compile and Run the Analysis Tool

```bash
# Compile the Rust analysis tool
rustc --edition 2021 analyze_serialization.rs -o analyze_serialization

# Run the analysis
./analyze_serialization
```

## 🔍 Key Findings Summary

Based on the comprehensive analysis:

### 📈 Impact Assessment

- **Serialization density**: 70-80% of the codebase
- **High-impact files**: 8+ files with deep integration
- **Separation difficulty**: **HIGH** - requires architectural changes

### 🎯 Root Cause

The crate is fundamentally architected around **YAML/JSON configuration input**, making serialization central to its operation rather than optional.

### 📋 Critical Files

1. `src/lib.rs` - Public API is serialization-based
2. `src/rule_config.rs` - Entire config system assumes serialization
3. `src/rule_core.rs` - Core logic mixed with serialization
4. `src/rule/mod.rs` - Every rule type has serialization derives

## 🗺️ Separation Strategy

### Phase 1: Feature Gating (1-2 weeks)

- ✅ **Low risk** - Feature gate files with minimal serialization
- Target files: `combined.rs`, `label.rs`, `check_var.rs`

### Phase 2: Abstraction Layer (3-4 weeks)

- ⚠️ **Medium risk** - Create trait abstractions for core functionality
- Design `RuleMatcher` and `RuleBuilder` traits

### Phase 3: Core Logic Extraction (6-8 weeks)

- ❌ **High risk** - Extract matching logic from serialization concerns
- Create non-serializable rule representations

### Phase 4: Alternative APIs (4-6 weeks)

- ⚠️ **Medium risk** - Provide programmatic rule construction
- Implement builder patterns for direct API usage

## 🛠️ Using the Tools

### AST-Grep Analysis

```bash
# Find all serde derives
ast-grep --lang rust --pattern '#[derive($$$)]' ../src/ | grep -E 'Serialize|Deserialize'

# Find serialization function calls
ast-grep --lang rust --pattern 'deserialize($$$)' ../src/
ast-grep --lang rust --pattern 'serialize($$$)' ../src/

# Find DeserializeEnv usage
ast-grep --lang rust --pattern 'DeserializeEnv' ../src/
```

### Helper Script Functions

The interactive helper provides:

- Current state validation
- Serialization usage analysis
- Feature gate candidate identification
- Abstraction point suggestions
- Patch file generation
- Separation roadmap creation

### Analysis Tool Features

The Rust analysis tool categorizes dependencies by:

- **Dependency type** (SerdeDerive, DeserializationCall, etc.)
- **Category** (Core Serialization, Schema Generation, etc.)
- **Severity** (High, Medium, Low impact)
- **Separation difficulty** assessment

## ⚠️ Important Considerations

### Backward Compatibility

Any separation effort must maintain the existing YAML/JSON-based public API for backward compatibility.

### Performance Impact

Abstraction layers may introduce performance overhead that should be benchmarked.

### Test Coverage

The test suite heavily relies on YAML-based rule construction and will need parallel test infrastructure.

### External Dependencies

Tools and documentation generation depend on JsonSchema derives, which complicates separation.

## 📝 Next Steps

1. **Review the analysis report** to understand the full scope
2. **Run the helper script** to explore current state
3. **Start with Phase 1** feature gating for quick wins
4. **Design abstraction layer** before attempting major refactoring
5. **Create migration plan** that maintains backward compatibility

## 🤝 Contributing

When working on separation:

- Use the analysis tools to validate changes
- Update the tools if new serialization patterns are introduced
- Maintain the separation roadmap as work progresses
- Test both serialized and non-serialized code paths

---

**Created by**: Serialization Analysis Task
**Date**: January 2025
**Purpose**: Support separation of serialization logic from core rule engine functionality for WASM deployment
