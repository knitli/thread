//! Serialization dependency analysis tool for the rule-engine crate.
//!
//! This tool helps identify and categorize all serialization-related code
//! to support the separation of serialization logic from core functionality.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SerializationDependency {
    pub file_path: String,
    pub line_number: usize,
    pub dependency_type: DependencyType,
    pub code_snippet: String,
    pub context: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyType {
    // Direct serde usage
    SerdeDerive,
    SerdeImport,
    SerializableType,

    // Serialization functions
    SerializationCall,
    DeserializationCall,
    YamlCall,


    // Schema generation
    JsonSchemaUsage,
    SchemaGeneration,

    // Serde field attributes
    SerdeAttribute,

    // Crate-specific patterns
    DeserializeEnvUsage,
    MaybeWrapper,
    TransformFunction,
    ConfigCreation,

    // Error handling
    SerializationError,
}

impl DependencyType {
    pub fn category(&self) -> &'static str {
        match self {
            DependencyType::SerdeDerive |
            DependencyType::SerdeImport |
            DependencyType::SerializableType |
            DependencyType::SerdeAttribute => "Core Serialization",

            DependencyType::SerializationCall |
            DependencyType::DeserializationCall |
            DependencyType::YamlCall => "Serialization Operations",

            DependencyType::JsonSchemaUsage |
            DependencyType::SchemaGeneration => "Schema Generation",

            DependencyType::DeserializeEnvUsage |
            DependencyType::MaybeWrapper |
            DependencyType::TransformFunction |
            DependencyType::ConfigCreation => "Crate-Specific Serialization",

            DependencyType::SerializationError => "Error Handling",
        }
    }

    pub fn severity(&self) -> SerializationSeverity {
        match self {
            // High impact - these are fundamental to serialization
            DependencyType::SerdeDerive |
            DependencyType::SerializableType |
            DependencyType::DeserializeEnvUsage => SerializationSeverity::High,

            // Medium impact - important but could potentially be abstracted
            DependencyType::SerializationCall |
            DependencyType::DeserializationCall |
            DependencyType::YamlCall |
            DependencyType::JsonSchemaUsage |
            DependencyType::TransformFunction |
            DependencyType::ConfigCreation => SerializationSeverity::Medium,

            // Low impact - imports and attributes that could be feature-gated
            DependencyType::SerdeImport |
            DependencyType::SerdeAttribute |
            DependencyType::SchemaGeneration |
            DependencyType::MaybeWrapper |
            DependencyType::SerializationError => SerializationSeverity::Low,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializationSeverity {
    High,    // Core to serialization, hard to separate
    Medium,  // Important but could be abstracted
    Low,     // Can be feature-gated or easily separated
}

#[derive(Debug)]
pub struct FileAnalysis {
    pub file_path: String,
    pub dependencies: Vec<SerializationDependency>,
    pub serialization_density: f64, // Percentage of lines with serialization code
    pub core_functionality: Vec<String>, // Non-serialization functions/types
    pub separation_difficulty: SerializationSeverity,
}

#[derive(Debug)]
pub struct SerializationAnalysisReport {
    pub files: Vec<FileAnalysis>,
    pub dependency_summary: HashMap<DependencyType, usize>,
    pub category_summary: HashMap<String, usize>,
    pub high_impact_files: Vec<String>,
    pub separation_strategy: SeparationStrategy,
}

#[derive(Debug)]
pub struct SeparationStrategy {
    pub feature_gate_candidates: Vec<String>,
    pub abstraction_layer_needed: Vec<String>,
    pub core_logic_files: Vec<String>,
    pub serialization_only_files: Vec<String>,
    pub mixed_responsibility_files: Vec<String>,
}

impl SerializationAnalysisReport {
    /// Generate a comprehensive analysis report
    pub fn generate_report() -> Self {
        let mut report = Self {
            files: Vec::new(),
            dependency_summary: HashMap::new(),
            category_summary: HashMap::new(),
            high_impact_files: Vec::new(),
            separation_strategy: SeparationStrategy {
                feature_gate_candidates: Vec::new(),
                abstraction_layer_needed: Vec::new(),
                core_logic_files: Vec::new(),
                serialization_only_files: Vec::new(),
                mixed_responsibility_files: Vec::new(),
            },
        };

        // Analyze each Rust file in the src directory
        if let Ok(entries) = fs::read_dir("../src") {
            for entry in entries.flatten() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "rs" {
                        let file_analysis = Self::analyze_file(&entry.path());
                        report.process_file_analysis(file_analysis);
                    }
                }
            }
        }

        report.generate_strategy();
        report
    }

    fn analyze_file(file_path: &Path) -> FileAnalysis {
        let file_content = fs::read_to_string(file_path).unwrap_or_default();
        let lines: Vec<&str> = file_content.lines().collect();
        let mut dependencies = Vec::new();
        let mut core_functionality = Vec::new();

        // Simulate AST-Grep analysis (in real implementation, this would call ast-grep)
        for (line_num, line) in lines.iter().enumerate() {
            // Check for serialization patterns
            if let Some(dep) = Self::detect_serialization_dependency(line, line_num + 1) {
                dependencies.push(dep);
            }

            // Detect core functionality (functions, structs, impls that don't seem serialization-related)
            if Self::is_core_functionality(line) {
                core_functionality.push(line.trim().to_string());
            }
        }

        let total_lines = lines.len();
        let serialization_lines = dependencies.len();
        let serialization_density = if total_lines > 0 {
            (serialization_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        let separation_difficulty = Self::assess_separation_difficulty(&dependencies);

        FileAnalysis {
            file_path: file_path.to_string_lossy().to_string(),
            dependencies,
            serialization_density,
            core_functionality,
            separation_difficulty,
        }
    }

    fn detect_serialization_dependency(line: &str, line_number: usize) -> Option<SerializationDependency> {
        let line = line.trim();

        // Check for various serialization patterns
        if line.contains("#[derive(") && (line.contains("Serialize") || line.contains("Deserialize")) {
            return Some(SerializationDependency {
                file_path: String::new(), // Will be set by caller
                line_number,
                dependency_type: DependencyType::SerdeDerive,
                code_snippet: line.to_string(),
                context: "Serde derive macro".to_string(),
            });
        }

        if line.starts_with("use serde") || line.contains("use serde_yaml") || line.contains("use serde_json") || line.contains("use schemars") {
            return Some(SerializationDependency {
                file_path: String::new(),
                line_number,
                dependency_type: DependencyType::SerdeImport,
                code_snippet: line.to_string(),
                context: "Serialization import".to_string(),
            });
        }

        if line.contains("deserialize(") || line.contains("serialize(") || line.contains("yaml::") || line.contains("serde_yaml::") || line.contains("from_yaml_string") {
            let dep_type = if line.contains("deserialize(") {
                DependencyType::DeserializationCall
            } else if line.contains("serialize(") {
                DependencyType::SerializationCall
            } else if line.contains("yaml::") || line.contains("serde_yaml::") || line.contains("from_yaml_string") {
                DependencyType::YamlCall
            };

            return Some(SerializationDependency {
                file_path: String::new(),
                line_number,
                dependency_type: dep_type,
                code_snippet: line.to_string(),
                context: "Serialization function call".to_string(),
            });
        }

        if line.contains("JsonSchema") {
            return Some(SerializationDependency {
                file_path: String::new(),
                line_number,
                dependency_type: DependencyType::JsonSchemaUsage,
                code_snippet: line.to_string(),
                context: "JSON schema usage".to_string(),
            });
        }

        if line.contains("DeserializeEnv") {
            return Some(SerializationDependency {
                file_path: String::new(),
                line_number,
                dependency_type: DependencyType::DeserializeEnvUsage,
                code_snippet: line.to_string(),
                context: "Deserialization environment".to_string(),
            });
        }

        if line.contains("Maybe::") || line.contains(": Maybe<") {
            return Some(SerializationDependency {
                file_path: String::new(),
                line_number,
                dependency_type: DependencyType::MaybeWrapper,
                code_snippet: line.to_string(),
                context: "Maybe wrapper for optional serialization".to_string(),
            });
        }

        None
    }

    fn is_core_functionality(line: &str) -> bool {
        let line = line.trim();

        // Look for core functionality patterns
        if line.starts_with("impl Matcher") ||
           line.starts_with("impl Pattern") ||
           line.starts_with("impl Rule") ||
           line.starts_with("impl<") && line.contains("RuleMatcher") ||
           line.starts_with("impl<") && line.contains("Matcher") ||
           line.starts_with("fn match_node") ||
           line.starts_with("fn potential_kinds") ||
           line.contains("find(") ||
           line.contains("ast_grep(") {
            return true;
        }

        false
    }

    fn assess_separation_difficulty(dependencies: &[SerializationDependency]) -> SerializationSeverity {
        let high_count = dependencies.iter().filter(|d| d.dependency_type.severity() == SerializationSeverity::High).count();
        let medium_count = dependencies.iter().filter(|d| d.dependency_type.severity() == SerializationSeverity::Medium).count();

        if high_count > 5 {
            SerializationSeverity::High
        } else if high_count > 0 || medium_count > 10 {
            SerializationSeverity::Medium
        } else {
            SerializationSeverity::Low
        }
    }

    fn process_file_analysis(&mut self, mut file_analysis: FileAnalysis) {
        // Update file paths in dependencies
        for dep in &mut file_analysis.dependencies {
            dep.file_path = file_analysis.file_path.clone();
        }

        // Update summaries
        for dep in &file_analysis.dependencies {
            *self.dependency_summary.entry(dep.dependency_type.clone()).or_insert(0) += 1;
            *self.category_summary.entry(dep.dependency_type.category().to_string()).or_insert(0) += 1;
        }

        // Track high-impact files
        if file_analysis.separation_difficulty == SerializationSeverity::High {
            self.high_impact_files.push(file_analysis.file_path.clone());
        }

        self.files.push(file_analysis);
    }

    fn generate_strategy(&mut self) {
        for file in &self.files {
            match file.separation_difficulty {
                SerializationSeverity::Low => {
                    if file.serialization_density > 50.0 {
                        self.separation_strategy.serialization_only_files.push(file.file_path.clone());
                    } else {
                        self.separation_strategy.feature_gate_candidates.push(file.file_path.clone());
                    }
                }
                SerializationSeverity::Medium => {
                    if file.core_functionality.len() > file.dependencies.len() {
                        self.separation_strategy.abstraction_layer_needed.push(file.file_path.clone());
                    } else {
                        self.separation_strategy.mixed_responsibility_files.push(file.file_path.clone());
                    }
                }
                SerializationSeverity::High => {
                    if !file.core_functionality.is_empty() {
                        self.separation_strategy.mixed_responsibility_files.push(file.file_path.clone());
                    }
                }
            }

            // Identify files with primarily core logic
            if file.serialization_density < 25.0 && !file.core_functionality.is_empty() {
                self.separation_strategy.core_logic_files.push(file.file_path.clone());
            }
        }
    }

    /// Generate a detailed report as a string
    pub fn format_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# SERIALIZATION DEPENDENCY ANALYSIS REPORT\n\n");
        report.push_str("## Executive Summary\n\n");
        report.push_str(&format!("- **Total files analyzed**: {}\n", self.files.len()));
        report.push_str(&format!("- **High-impact files**: {}\n", self.high_impact_files.len()));
        report.push_str(&format!("- **Total serialization dependencies**: {}\n",
            self.dependency_summary.values().sum::<usize>()));

        report.push_str("\n## Dependency Categories\n\n");
        for (category, count) in &self.category_summary {
            report.push_str(&format!("- **{}**: {} occurrences\n", category, count));
        }

        report.push_str("\n## Detailed Dependency Breakdown\n\n");
        for (dep_type, count) in &self.dependency_summary {
            report.push_str(&format!("- **{:?}**: {} ({})\n",
                dep_type, count, dep_type.category()));
        }

        report.push_str("\n## High-Impact Files (Difficult to Separate)\n\n");
        for file in &self.high_impact_files {
            if let Some(analysis) = self.files.iter().find(|f| f.file_path == *file) {
                report.push_str(&format!("### {}\n", file));
                report.push_str(&format!("- Serialization density: {:.1}%\n", analysis.serialization_density));
                report.push_str(&format!("- Dependencies: {}\n", analysis.dependencies.len()));
                report.push_str(&format!("- Core functions: {}\n\n", analysis.core_functionality.len()));
            }
        }

        report.push_str("\n## SEPARATION STRATEGY\n\n");

        report.push_str("### 1. Feature Gate Candidates (Easy wins)\n");
        for file in &self.separation_strategy.feature_gate_candidates {
            report.push_str(&format!("- `{}`\n", file));
        }

        report.push_str("\n### 2. Core Logic Files (Keep in core)\n");
        for file in &self.separation_strategy.core_logic_files {
            report.push_str(&format!("- `{}`\n", file));
        }

        report.push_str("\n### 3. Serialization-Only Files (Move to separate module)\n");
        for file in &self.separation_strategy.serialization_only_files {
            report.push_str(&format!("- `{}`\n", file));
        }

        report.push_str("\n### 4. Need Abstraction Layer\n");
        for file in &self.separation_strategy.abstraction_layer_needed {
            report.push_str(&format!("- `{}`\n", file));
        }

        report.push_str("\n### 5. Mixed Responsibility (Requires Refactoring)\n");
        for file in &self.separation_strategy.mixed_responsibility_files {
            report.push_str(&format!("- `{}`\n", file));
        }

        report.push_str("\n## RECOMMENDATIONS\n\n");
        report.push_str("1. **Immediate actions**: Feature-gate files with low serialization impact\n");
        report.push_str("2. **Short-term**: Create abstraction layer for files needing it\n");
        report.push_str("3. **Medium-term**: Refactor mixed responsibility files\n");
        report.push_str("4. **Long-term**: Consider trait-based abstraction for core serialization needs\n");

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_categorization() {
        assert_eq!(DependencyType::SerdeDerive.category(), "Core Serialization");
        assert_eq!(DependencyType::SerializationCall.category(), "Serialization Operations");
        assert_eq!(DependencyType::JsonSchemaUsage.category(), "Schema Generation");
    }

    #[test]
    fn test_severity_assessment() {
        assert_eq!(DependencyType::SerdeDerive.severity(), SerializationSeverity::High);
        assert_eq!(DependencyType::SerdeImport.severity(), SerializationSeverity::Low);
        assert_eq!(DependencyType::SerializationCall.severity(), SerializationSeverity::Medium);
    }
}

// Example usage in main function or CLI
fn main() {
    let report = SerializationAnalysisReport::generate_report();
    println!("{}", report.format_report());

    // Optionally save to file
    if let Err(e) = fs::write("serialization_analysis_report.md", report.format_report()) {
        eprintln!("Failed to write report: {}", e);
    } else {
        println!("\nReport saved to serialization_analysis_report.md");
    }
}
