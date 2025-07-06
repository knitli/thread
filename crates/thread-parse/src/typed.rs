// crates/thread-parser/src/typed.rs
use tree_sitter::{Parser, Tree, Node};
use anyhow::Result;

// Include generated type-sitter nodes
#[cfg(feature = "rust")]
include!(env!("RUST_NODES_PATH"));

pub struct TypedParser {
    parser: Parser,
    language: SupportedLanguage,
}

#[derive(Debug, Clone)]
pub enum SupportedLanguage {
    #[cfg(feature = "rust")]
    Rust,
    #[cfg(feature = "javascript")]
    JavaScript,
    #[cfg(feature = "typescript")]
    TypeScript,
}

impl TypedParser {
    pub fn new(language: SupportedLanguage) -> Result<Self> {
        let mut parser = Parser::new();

        let ts_language = match language {
            #[cfg(feature = "rust")]
            SupportedLanguage::Rust => tree_sitter_rust::language(),
            #[cfg(feature = "javascript")]
            SupportedLanguage::JavaScript => tree_sitter_javascript::language(),
            #[cfg(feature = "typescript")]
            SupportedLanguage::TypeScript => tree_sitter_typescript::language_typescript(),
        };

        parser.set_language(ts_language)?;
        Ok(Self { parser, language })
    }

    pub fn parse(&mut self, content: &str) -> Result<TypedTree> {
        let tree = self.parser.parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse content"))?;

        Ok(TypedTree {
            tree,
            content: content.to_string(),
            language: self.language.clone(),
        })
    }
}

pub struct TypedTree {
    tree: Tree,
    content: String,
    language: SupportedLanguage,
}

impl TypedTree {
    pub fn root_node(&self) -> TypedNode {
        TypedNode {
            node: self.tree.root_node(),
            content: &self.content,
            language: &self.language,
        }
    }
}

pub struct TypedNode<'a> {
    node: Node<'a>,
    content: &'a str,
    language: &'a SupportedLanguage,
}

impl<'a> TypedNode<'a> {
    pub fn text(&self) -> &'a str {
        self.node.utf8_text(self.content.as_bytes()).unwrap_or("")
    }

    pub fn kind(&self) -> &'a str {
        self.node.kind()
    }

    pub fn children(&self) -> impl Iterator<Item = TypedNode<'a>> {
        let mut cursor = self.node.walk();
        cursor.goto_first_child();

        std::iter::from_fn(move || {
            if cursor.goto_next_sibling() {
                Some(TypedNode {
                    node: cursor.node(),
                    content: self.content,
                    language: self.language,
                })
            } else {
                None
            }
        })
    }

    pub fn start_position(&self) -> (usize, usize) {
        let start = self.node.start_position();
        (start.row, start.column)
    }

    pub fn end_position(&self) -> (usize, usize) {
        let end = self.node.end_position();
        (end.row, end.column)
    }
}
