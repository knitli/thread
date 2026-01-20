// SPDX-FileCopyrightText: 2022 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT

use super::pre_process_pattern;
use thread_ast_engine::Language;
#[cfg(feature = "matching")]
use thread_ast_engine::matcher::KindMatcher;
#[cfg(feature = "matching")]
use thread_ast_engine::matcher::{Pattern, PatternBuilder, PatternError};
use thread_ast_engine::tree_sitter::{LanguageExt, TSLanguage};
#[cfg(feature = "matching")]
use thread_ast_engine::tree_sitter::{StrDoc, TSRange};
#[cfg(feature = "matching")]
use thread_ast_engine::{Doc, Node};
#[cfg(feature = "html-embedded")]
use thread_utils::RapidMap;

/// HTML language implementation with language injection capabilities.
///
/// Uses `z` as the expando character for metavariables since HTML attributes
/// and tag names have specific character restrictions.
///
/// ## Language Injection
///
/// Automatically detects and extracts embedded languages:
/// - **JavaScript** in `<script>` elements (default when no `lang` attribute)
/// - **CSS** in `<style>` elements (default when no `lang` attribute)
/// - **Other languages** when specified via `lang` attribute
///
/// ## Examples
///
/// ```rust
/// use thread_language::Html;
/// use thread_ast_engine::{Language, LanguageExt};
///
/// let html = Html;
/// let source = r#"
/// <script>console.log('hello');</script>
/// <style>.class { color: red; }</style>
/// <script lang="ts">const x: number = 42;</script>
/// "#;
///
/// let tree = html.ast_grep(source);
/// let injections = html.extract_injections(tree.root());
/// // injections contains JavaScript, CSS, and TypeScript ranges
/// ```
///
/// ## Note
/// tree-sitter-html uses locale-dependent `iswalnum` for tag names.
/// See: <https://github.com/tree-sitter/tree-sitter-html/blob/b5d9758e22b4d3d25704b72526670759a9e4d195/src/scanner.c#L194>
#[derive(Clone, Copy, Debug)]
pub struct Html;
impl Language for Html {
    fn expando_char(&self) -> char {
        'z'
    }
    fn pre_process_pattern<'q>(&self, query: &'q str) -> std::borrow::Cow<'q, str> {
        pre_process_pattern(self.expando_char(), query)
    }
    fn kind_to_id(&self, kind: &str) -> u16 {
        crate::parsers::language_html().id_for_node_kind(kind, true)
    }
    fn field_to_id(&self, field: &str) -> Option<u16> {
        crate::parsers::language_html()
            .field_id_for_name(field)
            .map(|f| f.get())
    }
    #[cfg(feature = "matching")]
    fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
        builder.build(|src| StrDoc::try_new(src, *self))
    }
}
impl LanguageExt for Html {
    fn get_ts_language(&self) -> TSLanguage {
        crate::parsers::language_html()
    }
    fn injectable_languages(&self) -> Option<&'static [&'static str]> {
        Some(&["css", "js", "ts", "tsx", "scss", "less", "stylus", "coffee"])
    }
    #[cfg(feature = "html-embedded")]
    fn extract_injections<L: LanguageExt>(
        &self,
        root: Node<StrDoc<L>>,
    ) -> RapidMap<String, Vec<TSRange>> {
        let lang = root.lang();
        let mut map = RapidMap::default();

        // Pre-allocate common language vectors to avoid repeated allocations
        let mut js_ranges = Vec::new();
        let mut css_ranges = Vec::new();
        let mut other_ranges: RapidMap<String, Vec<TSRange>> = RapidMap::default();

        // Process script elements
        let script_matcher = KindMatcher::new("script_element", lang);
        for script in root.find_all(script_matcher) {
            if let Some(content) = script.children().find(|c| c.kind() == "raw_text") {
                let range = node_to_range(&content);

                // Fast path for common languages
                match find_lang(&script) {
                    Some(lang_name) => {
                        if lang_name == "js" || lang_name == "javascript" {
                            js_ranges.push(range);
                        } else {
                            other_ranges.entry(lang_name).or_default().push(range);
                        }
                    }
                    None => js_ranges.push(range), // Default to JavaScript
                }
            }
        }

        // Process style elements
        let style_matcher = KindMatcher::new("style_element", lang);
        for style in root.find_all(style_matcher) {
            if let Some(content) = style.children().find(|c| c.kind() == "raw_text") {
                let range = node_to_range(&content);

                // Fast path for CSS (most common)
                match find_lang(&style) {
                    Some(lang_name) => {
                        if lang_name == "css" {
                            css_ranges.push(range);
                        } else {
                            other_ranges.entry(lang_name).or_default().push(range);
                        }
                    }
                    None => css_ranges.push(range), // Default to CSS
                }
            }
        }

        // Only insert non-empty vectors to reduce map size
        if !js_ranges.is_empty() {
            map.insert("js".to_string(), js_ranges);
        }
        if !css_ranges.is_empty() {
            map.insert("css".to_string(), css_ranges);
        }

        // Merge other languages
        for (lang_name, ranges) in other_ranges {
            if !ranges.is_empty() {
                map.insert(lang_name, ranges);
            }
        }

        map
    }
}

#[cfg(feature = "html-embedded")]
fn find_lang<D: Doc>(node: &Node<D>) -> Option<String> {
    let html = node.lang();
    let attr_matcher = KindMatcher::new("attribute", html);
    let name_matcher = KindMatcher::new("attribute_name", html);
    let val_matcher = KindMatcher::new("attribute_value", html);
    node.find_all(attr_matcher).find_map(|attr| {
        let name = attr.find(&name_matcher)?;
        if name.text() != "lang" {
            return None;
        }
        let val = attr.find(&val_matcher)?;
        Some(val.text().to_string())
    })
}
#[cfg(feature = "matching")]
fn node_to_range<D: Doc>(node: &Node<D>) -> TSRange {
    let r = node.range();
    let start = node.start_pos();
    let sp = start.byte_point();
    let sp = tree_sitter::Point::new(sp.0, sp.1);
    let end = node.end_pos();
    let ep = end.byte_point();
    let ep = tree_sitter::Point::new(ep.0, ep.1);
    TSRange {
        start_byte: r.start,
        end_byte: r.end,
        start_point: sp,
        end_point: ep,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_match(query: &str, source: &str) {
        use crate::test::test_match_lang;
        test_match_lang(query, source, Html);
    }

    fn test_non_match(query: &str, source: &str) {
        use crate::test::test_non_match_lang;
        test_non_match_lang(query, source, Html);
    }

    #[test]
    fn test_html_match() {
        test_match("<input>", "<input>");
        test_match("<$TAG>", "<input>");
        test_match("<$TAG class='foo'>$$$</$TAG>", "<div class='foo'></div>");
        test_match("<div>$$$</div>", "<div>123</div>");
        test_non_match("<$TAG class='foo'>$$$</$TAG>", "<div></div>");
        test_non_match("<div>$$$</div>", "<div class='foo'>123</div>");
    }

    fn test_replace(src: &str, pattern: &str, replacer: &str) -> String {
        use crate::test::test_replace_lang;
        test_replace_lang(src, pattern, replacer, Html)
    }

    #[test]
    fn test_html_replace() {
        let ret = test_replace(
            r#"<div class='foo'>bar</div>"#,
            r#"<$TAG class='foo'>$$$B</$TAG>"#,
            r#"<$TAG class='$$$B'>foo</$TAG>"#,
        );
        assert_eq!(ret, r#"<div class='bar'>foo</div>"#);
    }

    fn extract(src: &str) -> RapidMap<String, Vec<TSRange>> {
        let root = Html.ast_grep(src);
        Html.extract_injections(root.root())
    }

    #[test]
    fn test_html_extraction() {
        let map = extract("<script>a</script><style>.a{}</style>");
        assert!(map.contains_key("css"));
        assert!(map.contains_key("js"));
        assert_eq!(map["css"].len(), 1);
        assert_eq!(map["js"].len(), 1);
    }

    #[test]
    fn test_explicit_lang() {
        let map = extract(
            "<script lang='ts'>a</script><script lang=ts>.a{}</script><style lang=scss></style><style lang=\"scss\"></style>",
        );
        assert!(map.contains_key("ts"));
        assert_eq!(map["ts"].len(), 2);
        assert_eq!(map["scss"].len(), 2);
    }
}
