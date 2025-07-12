#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "schema")]
use schemars::JsonSchema;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), cfg_attr(feature = "schema", derive(JsonSchema)), cfg_attr(feature = "serde", serde(rename_all = "camelCase")))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LabelStyle {
    /// Labels that describe the primary cause of a diagnostic.
    Primary,
    /// Labels that provide additional context for a diagnostic.
    Secondary,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), cfg_attr(feature = "schema", derive(JsonSchema)))]
#[derive(Clone)]
pub struct LabelConfig {
    pub style: LabelStyle,
    pub message: Option<String>,
}

/// A label is a way to mark a specific part of the code with a styled message.
/// It is used to provide diagnostic information in LSP or CLI.
/// 'r represents a lifetime for the message string from `rule`.
/// 't represents a lifetime for the node from a ast `tree`.
pub struct Label<'r, 't, D: Doc> {
    pub style: LabelStyle,
    pub message: Option<&'r str>,
    pub start_node: Node<'t, D>,
    pub end_node: Node<'t, D>,
}
