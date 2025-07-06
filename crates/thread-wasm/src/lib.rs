// crates/thread-wasm/src/lib.rs
use wasm_bindgen::prelude::*;
use thread_core::ThreadAnalysisEngine;
use js_sys::Array;
use web_sys::console;

mod utils;

// Use dlmalloc for memory management in WASM
#[global_allocator]
static ALLOC: std::sync::GlobalDlmalloc = std::sync::GlobalDlmalloc;

#[wasm_bindgen]
pub struct ThreadWasm {
    engine: ThreadAnalysisEngine,
}

#[wasm_bindgen]
impl ThreadWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ThreadWasm, JsValue> {
        let engine = ThreadAnalysisEngine::new()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(ThreadWasm { engine })
    }

    #[wasm_bindgen]
    pub fn analyze_code(&mut self, content: &str, language: &str) -> Result<JsValue, JsValue> {
        let lang = match language {
            "rust" => thread_lang::SupportedLanguage::Rust,
            "javascript" => thread_lang::SupportedLanguage::JavaScript,
            "typescript" => thread_lang::SupportedLanguage::TypeScript,
            "python" => thread_lang::SupportedLanguage::Python,
            _ => return Err(JsValue::from_str("Unsupported language")),
        };

        // Create temporary file path for analysis
        let temp_path = std::path::Path::new("temp.rs"); // Would be dynamic in real implementation

        // This would need to be adapted for WASM - can't directly use file system
        // Instead, we'd parse the content directly
        let result = self.parse_content_directly(content, &lang)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Convert to JS-compatible format
        let js_result = serde_wasm_bindgen::to_value(&result)?;
        Ok(js_result)
    }

    fn parse_content_directly(&mut self, content: &str, language: &thread_lang::SupportedLanguage) -> Result<WasmAnalysisResult, Box<dyn std::error::Error>> {
        // Direct parsing without file system access
        let ast_graph = self.engine.parse_to_graph(content, language)?;

        Ok(WasmAnalysisResult {
            node_count: ast_graph.node_count(),
            edge_count: ast_graph.edge_count(),
            language: format!("{:?}", language),
            line_count: content.lines().count(),
        })
    }
}

#[derive(serde::Serialize)]
struct WasmAnalysisResult {
    node_count: usize,
    edge_count: usize,
    language: String,
    line_count: usize,
}
