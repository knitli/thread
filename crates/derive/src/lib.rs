// crates/thread-derive/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Attribute, Lit, Meta, NestedMeta};

/// Derive macro for LanguageParser trait
///
/// This dramatically simplifies adding new language support to thread.
/// Instead of implementing ~200 lines of boilerplate, you just specify
/// the tree-sitter queries for your language.
///
/// # Example
///
/// ```rust
/// #[derive(LanguageParser)]
/// #[language(
///     id = "rust",
///     extensions = [".rs"],
///     tree_sitter = "tree_sitter_rust::language"
/// )]
/// #[queries(
///     function = r#"
///         (function_item
///             name: (identifier) @name
///             parameters: (parameters) @parameters
///             return_type: (type)? @return_type
///         ) @function
///     "#,
///     struct = r#"
///         (struct_item
///             name: (type_identifier) @name
///         ) @struct
///     "#,
///     import = r#"
///         (use_declaration
///             argument: (use_clause) @clause
///         ) @use
///     "#
/// )]
/// struct RustParser;
/// ```
#[proc_macro_derive(LanguageParser, attributes(language, queries, dependencies))]
pub fn derive_language_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract language configuration
    let language_config = extract_language_config(&input.attrs)
        .expect("Missing #[language(...)] attribute");

    // Extract query definitions
    let queries = extract_queries(&input.attrs)
        .expect("Missing #[queries(...)] attribute");

    // Extract dependency patterns (optional)
    let dependency_patterns = extract_dependency_patterns(&input.attrs);

    let struct_name = &input.ident;
    let language_id = &language_config.id;
    let extensions = &language_config.extensions;
    let tree_sitter_fn = &language_config.tree_sitter_fn;

    // Generate the implementation
    let expanded = quote! {
        impl #struct_name {
            /// Create a new parser instance
            pub fn new() -> thread_core::Result<Self> {
                Ok(Self)
            }

            /// Get compiled queries for this language
            fn get_compiled_queries() -> std::collections::HashMap<thread_core::ElementKind, tree_sitter::Query> {
                use std::sync::OnceLock;
                static QUERIES: OnceLock<std::collections::HashMap<thread_core::ElementKind, tree_sitter::Query>> = OnceLock::new();

                QUERIES.get_or_init(|| {
                    let language = #tree_sitter_fn();
                    let mut queries = std::collections::HashMap::new();

                    #(
                        if let Ok(query) = tree_sitter::Query::new(language, #queries) {
                            queries.insert(thread_core::ElementKind::#query_kinds, query);
                        }
                    )*

                    queries
                }).clone()
            }
        }

        impl thread_core::LanguageParser for #struct_name {
            fn language_id(&self) -> &'static str {
                #language_id
            }

            fn file_extensions(&self) -> &'static [&'static str] {
                &[#(#extensions),*]
            }

            fn parse_file(&self, content: &str, file_path: &std::path::Path) -> thread_core::Result<thread_core::FileParseResult> {
                let start_time = std::time::Instant::now();

                // Parse with tree-sitter
                let mut parser = tree_sitter::Parser::new();
                parser.set_language(#tree_sitter_fn())
                    .map_err(|e| thread_core::ThreadError::TreeSitter(format!("Failed to set language: {}", e)))?;

                let tree = parser.parse(content, None)
                    .ok_or_else(|| thread_core::ThreadError::ParseError("Failed to parse file".to_string()))?;

                // Extract elements using compiled queries
                let elements = self.extract_elements_from_tree(&tree, content, file_path)?;

                // Extract imports and dependencies
                let imports = self.extract_imports(&tree, content, file_path)?;
                let exports = self.extract_exports(&elements);

                // Generate content hash
                let content_hash = thread_core::ContentHash::from_content(content);

                Ok(thread_core::FileParseResult {
                    file_path: file_path.to_string_lossy().to_string(),
                    language: #language_id.to_string(),
                    elements,
                    imports,
                    exports,
                    content_hash,
                    parse_time_ms: start_time.elapsed().as_millis() as u64,
                })
            }

            fn extract_dependencies(&self, content: &str, file_path: &std::path::Path) -> thread_core::Result<Vec<String>> {
                // Generated dependency extraction logic
                self.extract_dependencies_impl(content, file_path)
            }
        }

        // Generated helper methods
        impl #struct_name {
            fn extract_elements_from_tree(
                &self,
                tree: &tree_sitter::Tree,
                content: &str,
                file_path: &std::path::Path,
            ) -> thread_core::Result<Vec<thread_core::CodeElement>> {
                let mut elements = Vec::new();
                let queries = Self::get_compiled_queries();
                let mut cursor = tree_sitter::QueryCursor::new();

                // Run each query and extract elements
                for (kind, query) in &queries {
                    let matches = cursor.matches(query, tree.root_node(), content.as_bytes());

                    for m in matches {
                        if let Some(element) = self.extract_element_from_match(kind, &m, query, content, file_path)? {
                            elements.push(element);
                        }
                    }
                }

                // Sort by location for consistent output
                elements.sort_by_key(|e| (e.location.start_byte, e.location.end_byte));

                Ok(elements)
            }

            fn extract_element_from_match(
                &self,
                kind: &thread_core::ElementKind,
                m: &tree_sitter::QueryMatch,
                query: &tree_sitter::Query,
                content: &str,
                file_path: &std::path::Path,
            ) -> thread_core::Result<Option<thread_core::CodeElement>> {
                let capture_names = query.capture_names();
                let mut captures = std::collections::HashMap::new();

                // Collect all captures
                for capture in m.captures {
                    let name = &capture_names[capture.index as usize];
                    let node = capture.node;
                    let text = node.utf8_text(content.as_bytes())
                        .map_err(|e| thread_core::ThreadError::TreeSitter(format!("UTF-8 error: {}", e)))?;

                    captures.insert(name.clone(), (node, text.to_string()));
                }

                // Extract name (required for all elements)
                let name_capture = captures.get("name")
                    .ok_or_else(|| thread_core::ThreadError::TreeSitter("Element name not captured".to_string()))?;

                let name = name_capture.1.clone();
                let node = name_capture.0;

                // Build signature (language-specific logic could be generated here)
                let signature = self.build_signature(kind, &captures)?;

                // Create location
                let location = thread_core::SourceLocation::from_node(&node, file_path.to_string_lossy().to_string(), content);

                // Generate content hash
                let content_hash = thread_core::ContentHash::from_content(&signature);

                // Create element ID
                let element_id = thread_core::ElementId(format!("{}::{}", file_path.to_string_lossy(), name));

                Ok(Some(thread_core::CodeElement {
                    id: element_id,
                    kind: kind.clone(),
                    name,
                    signature,
                    location,
                    content_hash,
                    dependencies: Vec::new(), // TODO: Extract from captures
                    metadata: self.extract_metadata(&captures)?,
                }))
            }

            fn build_signature(
                &self,
                kind: &thread_core::ElementKind,
                captures: &std::collections::HashMap<String, (tree_sitter::Node, String)>,
            ) -> thread_core::Result<String> {
                // Generate language-specific signature building logic
                match kind {
                    thread_core::ElementKind::Function => {
                        let name = captures.get("name").map(|(_, text)| text).unwrap_or(&"unknown".to_string());
                        let params = captures.get("parameters").map(|(_, text)| text).unwrap_or(&"()".to_string());
                        Ok(format!("{}({})", name, params))
                    }
                    _ => {
                        let name = captures.get("name").map(|(_, text)| text).unwrap_or(&"unknown".to_string());
                        Ok(name.clone())
                    }
                }
            }

            fn extract_metadata(
                &self,
                captures: &std::collections::HashMap<String, (tree_sitter::Node, String)>,
            ) -> thread_core::Result<thread_core::ElementMetadata> {
                Ok(thread_core::ElementMetadata {
                    visibility: self.extract_visibility(captures),
                    is_async: captures.contains_key("async"),
                    is_generic: captures.contains_key("type_parameters"),
                    docstring: captures.get("docstring").map(|(_, text)| text.clone()),
                    annotations: Vec::new(), // TODO: Extract from captures
                    return_type: captures.get("return_type").map(|(_, text)| text.clone()),
                    parameters: Vec::new(), // TODO: Parse parameters
                    extra: std::collections::HashMap::new(),
                })
            }

            fn extract_visibility(&self, captures: &std::collections::HashMap<String, (tree_sitter::Node, String)>) -> Option<thread_core::Visibility> {
                if captures.contains_key("pub") || captures.get("visibility").map(|(_, text)| text.contains("pub")).unwrap_or(false) {
                    Some(thread_core::Visibility::Public)
                } else if captures.get("visibility").map(|(_, text)| text.contains("private")).unwrap_or(false) {
                    Some(thread_core::Visibility::Private)
                } else {
                    None
                }
            }

            fn extract_imports(
                &self,
                tree: &tree_sitter::Tree,
                content: &str,
                file_path: &std::path::Path,
            ) -> thread_core::Result<Vec<thread_core::Import>> {
                // Generated import extraction logic based on #[dependencies] patterns
                Ok(Vec::new()) // Placeholder
            }

            fn extract_exports(&self, elements: &[thread_core::CodeElement]) -> Vec<thread_core::Export> {
                elements
                    .iter()
                    .filter(|element| matches!(element.metadata.visibility, Some(thread_core::Visibility::Public)))
                    .map(|element| thread_core::Export {
                        name: element.name.clone(),
                        kind: element.kind.clone(),
                        location: element.location.clone(),
                    })
                    .collect()
            }

            fn extract_dependencies_impl(&self, content: &str, file_path: &std::path::Path) -> thread_core::Result<Vec<String>> {
                // Generated dependency extraction based on patterns
                Ok(Vec::new()) // Placeholder
            }
        }

        impl Default for #struct_name {
            fn default() -> Self {
                Self::new().expect("Failed to create default parser")
            }
        }
    };

    TokenStream::from(expanded)
}

// Helper structs for parsing macro attributes
struct LanguageConfig {
    id: String,
    extensions: Vec<String>,
    tree_sitter_fn: String,
}

fn extract_language_config(attrs: &[Attribute]) -> Option<LanguageConfig> {
    // Parse #[language(id = "rust", extensions = [".rs"], tree_sitter = "tree_sitter_rust::language")]
    // Implementation would parse the attribute syntax
    None // Placeholder
}

fn extract_queries(attrs: &[Attribute]) -> Option<Vec<(String, String)>> {
    // Parse #[queries(function = "...", struct = "...", ...)]
    // Implementation would parse query definitions
    None // Placeholder
}

fn extract_dependency_patterns(attrs: &[Attribute]) -> Option<Vec<String>> {
    // Parse #[dependencies(patterns = ["...", "..."])]
    // Implementation would parse dependency extraction patterns
    None // Placeholder
}
