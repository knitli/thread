#!/bin/bash

# Serialization Separation Helper Script
# This script provides practical tools for separating serialization logic
# from core functionality in the thread-rule-engine crate

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
CRATE_DIR="$SCRIPT_DIR"
SRC_DIR="$CRATE_DIR/src"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Thread Rule Engine Serialization Separation Helper ===${NC}"
echo ""

# Function to print section headers
print_section() {
    echo -e "${GREEN}=== $1 ===${NC}"
}

# Function to print warnings
print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function to print errors
print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Check if ast-grep is available
check_ast_grep() {
    if command -v ast-grep &> /dev/null; then
        print_success "ast-grep is available"
        return 0
    else
        print_error "ast-grep is not installed. Please install it first:"
        echo "  npm install -g @ast-grep/cli"
        echo "  or visit: https://ast-grep.github.io/guide/quick-start.html"
        return 1
    fi
}

# Function to analyze current serialization usage
analyze_current_usage() {
    print_section "Current Serialization Usage Analysis"

    echo "Analyzing Serde derive usage..."
    if command -v ast-grep &> /dev/null; then
        cd "$SRC_DIR"
        echo ""
        echo "Files with Serde derives:"
        ast-grep --lang rust -p '#[derive($$$)]' --json | jq -r '.[] | select(.text | test("Serialize|Deserialize")) | .file' | sort | uniq

        echo ""
        echo "Serde import statements:"
        ast-grep --lang rust -p 'use serde' --json | jq -r '.[] | "\(.file):\(.range.start.line): \(.text)"'

        echo ""
        echo "DeserializeEnv usage:"
        ast-grep --lang rust -p 'DeserializeEnv' --json | jq -r '.[] | "\(.file):\(.range.start.line): \(.text)"'
    else
        print_warning "ast-grep not available, performing basic grep analysis..."
        echo ""
        echo "Files with Serde derives:"
        grep -r "derive.*Serialize\|derive.*Deserialize" . --include="*.rs" | cut -d: -f1 | sort | uniq

        echo ""
        echo "Serde imports:"
        grep -r "use serde" . --include="*.rs"
    fi
}

# Function to identify feature gate candidates
identify_feature_gate_candidates() {
    print_section "Feature Gate Candidates (Phase 1)"

    echo "These files have minimal serialization and can be feature-gated easily:"
    echo ""

    # Files with low serialization density
    local candidates=(
        "src/combined.rs"
        "src/label.rs"
        "src/check_var.rs"
        "src/maybe.rs"
    )

    for file in "${candidates[@]}"; do
        if [[ -f "$CRATE_DIR/$file" ]]; then
            local serde_count total_lines density
            serde_count="$(grep -c "serde\|Serialize\|Deserialize" "$CRATE_DIR/$file" 2>/dev/null)"
            echo "$serde_count"
            total_lines="$(wc -l < "$CRATE_DIR/$file" 2>/dev/null)"
            density="$((serde_count * 100 / total_lines))"

            if [[ $density -lt 30 ]]; then
                echo -e "  ✅ ${GREEN}$file${NC} - Serialization density: ${density}%"
            else
                echo -e "  ⚠️  ${YELLOW}$file${NC} - Serialization density: ${density}% (review needed)"
            fi
        else
            echo -e "  ❓ $file - File not found"
        fi
    done
}

# Function to suggest abstraction points
suggest_abstractions() {
    print_section "Abstraction Layer Suggestions (Phase 2)"

    cat << 'EOF'
Consider creating these trait abstractions:

1. Core Rule Matching:
   ```rust
   pub trait RuleMatcher {
       fn match_node(&self, node: Node) -> Option<Node>;
       fn potential_kinds(&self) -> Option<BitSet>;
   }
   ```

2. Rule Construction:
   ```rust
   pub trait RuleBuilder<L: Language> {
       type Rule: RuleMatcher;
       fn pattern(pattern: &str) -> Result<Self::Rule, Error>;
       fn kind(kind: &str) -> Result<Self::Rule, Error>;
       fn compose(rules: Vec<Self::Rule>) -> Self::Rule;
   }
   ```

3. Configuration Management:
   ```rust
   pub trait ConfigManager<L: Language> {
       type Config;
       fn from_rules(rules: Vec<impl RuleMatcher>) -> Self::Config;
       fn scan(&self, source: &str) -> ScanResult;
   }
   ```

Files that would benefit from abstraction:
EOF

    local abstraction_candidates=(
        "src/rule_core.rs - Extract matching logic from serialization"
        "src/fixer.rs - Separate fix logic from config parsing"
        "src/transform/mod.rs - Abstract transformation logic"
    )

    for candidate in "${abstraction_candidates[@]}"; do
        echo "  • $candidate"
    done
}

# Function to create feature gate patches
create_feature_gate_patches() {
    print_section "Creating Feature Gate Patches"

    local patch_dir="$CRATE_DIR/separation_patches"
    mkdir -p "$patch_dir"

    # Create Cargo.toml patch
    cat > "$patch_dir/Cargo.toml.patch" << 'EOF'
# Add to [features] section
[features]
default = ["serde", "schema"]
serde = ["dep:serde", "dep:serde_yaml", "dep:serde_json"]
schema = ["dep:schemars", "serde"]

# Make serde dependencies optional
[dependencies]
serde = { workspace = true, optional = true }
serde_yaml = { workspace = true, optional = true }
serde_json = { workspace = true, optional = true }
schemars = { workspace = true, optional = true }
EOF

    # Create lib.rs patch
    cat > "$patch_dir/lib.rs.patch" << 'EOF'
// Add feature gates to imports
#[cfg(feature = "serde")]
use serde::Deserialize;
#[cfg(feature = "serde")]
use serde_yaml::{with::singleton_map_recursive::deserialize, Deserializer, Error as YamlError};

// Feature gate serialization functions
#[cfg(feature = "serde")]
pub fn from_str<'de, T: Deserialize<'de>>(s: &'de str) -> Result<T, YamlError> {
    let deserializer = Deserializer::from_str(s);
    deserialize(deserializer)
}

#[cfg(feature = "serde")]
pub fn from_yaml_string<'a, L: Language + Deserialize<'a>>(
    yamls: &'a str,
    registration: &GlobalRules,
) -> Result<Vec<RuleConfig<L>>, RuleConfigError> {
    // ... existing implementation
}
EOF

    print_success "Feature gate patches created in: $patch_dir/"
    echo "Review and apply these patches as appropriate."
}

# Function to run dependency analysis
run_dependency_analysis() {
    print_section "Detailed Dependency Analysis"

    if [[ -f "$CRATE_DIR/analyze_serialization.rs" ]]; then
        echo "Running custom dependency analysis..."
        cd "$CRATE_DIR"
        # Note: This would need the analysis tool to be compiled and executable
        echo "To run the analysis tool:"
        echo "  cd crates/rule-engine"
        echo "  rustc --edition 2021 analyze_serialization.rs -o analyze_serialization"
        echo "  ./analyze_serialization"
    else
        print_warning "Custom analysis tool not found. Using basic analysis..."
    fi

    echo ""
    echo "Manual checks to perform:"
    echo "1. Count serde derives: grep -r 'derive.*Serialize' src/ | wc -l"
    echo "2. Find serialization calls: grep -r 'deserialize\|serialize' src/ | wc -l"
    echo "3. Check schema usage: grep -r 'JsonSchema' src/ | wc -l"
    echo "4. Identify core logic: grep -r 'impl.*Matcher' src/"
}

# Function to generate separation roadmap
generate_roadmap() {
    print_section "Separation Roadmap"

    cat << 'EOF'
## Phase 1: Feature Gating (1-2 weeks)
- [ ] Add optional serde dependencies to Cargo.toml
- [ ] Feature gate imports in lib.rs
- [ ] Feature gate simple files (combined.rs, label.rs, etc.)
- [ ] Update tests to handle feature flags
- [ ] Verify compilation with/without serde feature

## Phase 2: Abstraction Layer (3-4 weeks)
- [ ] Design core traits (RuleMatcher, RuleBuilder)
- [ ] Implement traits for existing types
- [ ] Create non-serializable rule representations
- [ ] Add conversion between serializable/non-serializable
- [ ] Update internal APIs to use traits

## Phase 3: Core Logic Extraction (6-8 weeks)
- [ ] Extract matching logic from Rule enum
- [ ] Create separate runtime rule types
- [ ] Implement programmatic rule construction API
- [ ] Refactor RuleCore to use abstractions
- [ ] Update transform system

## Phase 4: Alternative APIs (4-6 weeks)
- [ ] Design builder pattern API
- [ ] Implement programmatic configuration
- [ ] Add direct rule construction methods
- [ ] Create migration guide
- [ ] Performance optimization

## Testing Strategy
- [ ] Create feature flag test matrix
- [ ] Add programmatic API tests
- [ ] Performance benchmarks
- [ ] Migration validation tests

## Documentation
- [ ] Update README with feature flags
- [ ] Document separation architecture
- [ ] Create migration guide
- [ ] Update examples
EOF
}

# Function to validate current state
validate_current_state() {
    print_section "Current State Validation"

    echo "Checking crate structure..."

    if [[ ! -f "$CRATE_DIR/Cargo.toml" ]]; then
        print_error "Cargo.toml not found in $CRATE_DIR"
        return 1
    fi

    if [[ ! -d "$SRC_DIR" ]]; then
        print_error "src/ directory not found"
        return 1
    fi

    # Check for key files
    local key_files=(
        "src/lib.rs"
        "src/rule_config.rs"
        "src/rule_core.rs"
        "src/rule/mod.rs"
    )

    for file in "${key_files[@]}"; do
        if [[ -f "$CRATE_DIR/$file" ]]; then
            print_success "$file exists"
        else
            print_error "$file not found"
        fi
    done

    echo ""
    echo "Checking dependencies..."
    if grep -q "serde.*=" "$CRATE_DIR/Cargo.toml"; then
        print_warning "Serde dependencies found (expected)"
    fi

    if grep -q "schemars.*=" "$CRATE_DIR/Cargo.toml"; then
        print_warning "Schemars dependency found (expected)"
    fi
}

# Main menu
show_menu() {
    echo ""
    echo "Available actions:"
    echo "1. Validate current state"
    echo "2. Analyze current serialization usage"
    echo "3. Identify feature gate candidates"
    echo "4. Suggest abstraction points"
    echo "5. Create feature gate patches"
    echo "6. Run dependency analysis"
    echo "7. Generate separation roadmap"
    echo "8. Run all analyses"
    echo "0. Exit"
    echo ""
}

# Main execution
main() {
    cd "$CRATE_DIR"

    while true; do
        show_menu
        read -p "Select an action (0-8): " choice
        echo ""

        case $choice in
            1) validate_current_state ;;
            2) analyze_current_usage ;;
            3) identify_feature_gate_candidates ;;
            4) suggest_abstractions ;;
            5) create_feature_gate_patches ;;
            6) run_dependency_analysis ;;
            7) generate_roadmap ;;
            8)
                validate_current_state
                echo ""
                analyze_current_usage
                echo ""
                identify_feature_gate_candidates
                echo ""
                suggest_abstractions
                echo ""
                generate_roadmap
                ;;
            0)
                echo "Goodbye!"
                exit 0
                ;;
            *)
                print_error "Invalid choice. Please select 0-8."
                ;;
        esac

        echo ""
        read -p "Press Enter to continue..."
    done
}

# Check prerequisites
if ! check_ast_grep; then
    echo ""
    print_warning "Some features will be limited without ast-grep"
    echo ""
fi

# Run main menu
main
