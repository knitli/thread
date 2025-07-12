use crate::language::Language;
use crate::matcher::NodeMatch;
use crate::rule::RuleConfig;
use crate::ast::Doc;

pub struct ScanResult<'t, 'r, D: Doc, L: Language> {
    pub diffs: Vec<(&'r RuleConfig<L>, NodeMatch<'t, D>)>,
    pub matches: Vec<(&'r RuleConfig<L>, Vec<NodeMatch<'t, D>>)>,
}

/// store the index to the rule and the matched node
/// it will be converted to ScanResult by resolving the rule
pub struct ScanResultInner<'t, D: Doc> {
    diffs: Vec<(usize, NodeMatch<'t, D>)>,
    matches: FastMap<usize, Vec<NodeMatch<'t, D>>>,
    unused_suppressions: Vec<NodeMatch<'t, D>>,
}

pub enum SuppressKind {
    /// suppress the whole file
    File,
    /// suppress specific line
    Line(usize),
}

pub struct Suppressions {
    file: Option<Suppression>,
    /// line number which may be suppressed
    lines: thread_utils::FastMap<usize, Suppression>,
}

pub struct Suppression {
    /// None = suppress all
    suppressed: Option<thread_utils::FastSet<String>>,
    node_id: usize,
}

pub enum MaySuppressed<'a> {
    Yes(&'a Suppression),
    No,
}

/// A struct to group all rules according to their potential kinds.
/// This can greatly reduce traversal times and skip unmatchable rules.
/// Rules are referenced by their index in the rules vector.
pub struct CombinedScan<'r, L: Language> {
    rules: Vec<&'r RuleConfig<L>>,
    /// a vec of vec, mapping from kind to a list of rule index
    kind_rule_mapping: Vec<Vec<usize>>,
    /// a rule for unused_suppressions
    unused_suppression_rule: Option<&'r RuleConfig<L>>,
}
