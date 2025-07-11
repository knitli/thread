use crate::maybe::Maybe;
use thread_utils::{FastMap, FastSet};

pub enum CheckHint<'r> {
  Global,
  Normal,
  Rewriter(&'r FastSet<&'r str>),
}

/// A rule object to find matching AST nodes. We have three categories of rules in ast-grep.
///
/// * Atomic: the most basic rule to match AST. We have two variants: Pattern and Kind.
///
/// * Relational: filter matched target according to their position relative to other nodes.
///
/// * Composite: use logic operation all/any/not to compose the above rules to larger rules.
///
/// Every rule has it's unique name so we can combine several rules in one object.
#[derive(Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SerializableRule {
    // avoid embedding AtomicRule/RelationalRule/CompositeRule with flatten here for better error message

    // atomic
    /// A pattern string or a pattern object.
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    pub pattern: Maybe<PatternStyle>,
    /// The kind name of the node to match. You can look up code's kind names in playground.
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    pub kind: Maybe<String>,
    /// A Rust regular expression to match the node's text. https://docs.rs/regex/latest/regex/#syntax
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    pub regex: Maybe<String>,
    /// `nth_child` accepts number, string or object.
    /// It specifies the position in nodes' sibling list.
    #[serde(default, skip_serializing_if = "Maybe::is_absent", rename = "nthChild")]
    pub nth_child: Maybe<SerializableNthChild>,
    /// `range` accepts a range object.
    /// the target node must exactly appear in the range.
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    pub range: Maybe<SerializableRange>,

    // relational
    /// `inside` accepts a relational rule object.
    /// the target node must appear inside of another node matching the `inside` sub-rule.
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    pub inside: Maybe<Box<Relation>>,
    /// `has` accepts a relational rule object.
    /// the target node must has a descendant node matching the `has` sub-rule.
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    pub has: Maybe<Box<Relation>>,
    /// `precedes` accepts a relational rule object.
    /// the target node must appear before another node matching the `precedes` sub-rule.
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    pub precedes: Maybe<Box<Relation>>,
    /// `follows` accepts a relational rule object.
    /// the target node must appear after another node matching the `follows` sub-rule.
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    pub follows: Maybe<Box<Relation>>,
    // composite
    /// A list of sub rules and matches a node if all of sub rules match.
    /// The meta variables of the matched node contain all variables from the sub-rules.
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    pub all: Maybe<Vec<SerializableRule>>,
    /// A list of sub rules and matches a node if any of sub rules match.
    /// The meta variables of the matched node only contain those of the matched sub-rule.
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    pub any: Maybe<Vec<SerializableRule>>,
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    /// A single sub-rule and matches a node if the sub rule does not match.
    pub not: Maybe<Box<SerializableRule>>,
    /// A utility rule id and matches a node if the utility rule matches.
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    pub matches: Maybe<String>,
}

pub struct AtomicRule {
    pub pattern: Option<PatternStyle>,
    pub kind: Option<String>,
    pub regex: Option<String>,
    pub nth_child: Option<SerializableNthChild>,
    pub range: Option<SerializableRange>,
}
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Strictness {
    /// all nodes are matched
    Cst,
    /// all nodes except source trivial nodes are matched.
    Smart,
    /// only ast nodes are matched
    Ast,
    /// ast-nodes excluding comments are matched
    Relaxed,
    /// ast-nodes excluding comments, without text
    Signature,
}

/// A String pattern will match one single AST node according to pattern syntax.
/// Or an object with field `context`, `selector` and optionally `strictness`.
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum PatternStyle {
    Str(String),
    Contextual {
        /// The surrounding code that helps to resolve any ambiguity in the syntax.
        context: String,
        /// The sub-syntax node kind that is the actual matcher of the pattern.
        selector: Option<String>,
        /// Strictness of the pattern. More strict pattern matches fewer nodes.
        strictness: Option<Strictness>,
    },
}

pub struct RelationalRule {
    pub inside: Option<Box<Relation>>,
    pub has: Option<Box<Relation>>,
    pub precedes: Option<Box<Relation>>,
    pub follows: Option<Box<Relation>>,
}

pub struct CompositeRule {
    pub all: Option<Vec<SerializableRule>>,
    pub any: Option<Vec<SerializableRule>>,
    pub not: Option<Box<SerializableRule>>,
    pub matches: Option<String>,
}

pub enum Rule {
    // atomic
    Pattern(Pattern),
    Kind(KindMatcher),
    Regex(RegexMatcher),
    NthChild(NthChild),
    Range(RangeMatcher),
    // relational
    Inside(Box<Inside>),
    Has(Box<Has>),
    Precedes(Box<Precedes>),
    Follows(Box<Follows>),
    // composite
    All(o::All<Rule>),
    Any(o::Any<Rule>),
    Not(Box<o::Not<Rule>>),
    Matches(ReferentRule),
}

struct ContingentRule<L: Language> {
  rule: RuleConfig<L>,
  files_globs: Option<GlobSet>,
  ignore_globs: Option<GlobSet>,
}

#[derive(Debug, Error)]
pub enum RuleSerializeError {
    #[error("Rule must have one positive matcher.")]
    MissPositiveMatcher,
    #[error("Rule contains invalid kind matcher.")]
    InvalidKind(#[from] KindMatcherError),
    #[error("Rule contains invalid pattern matcher.")]
    InvalidPattern(#[from] PatternError),
    #[error("Rule contains invalid nthChild.")]
    NthChild(#[from] NthChildError),
    #[error("Rule contains invalid regex matcher.")]
    WrongRegex(#[from] RegexMatcherError),
    #[error("Rule contains invalid matches reference.")]
    MatchesReference(#[from] ReferentRuleError),
    #[error("Rule contains invalid range matcher.")]
    InvalidRange(#[from] RangeMatcherError),
    #[error("field is only supported in has/inside.")]
    FieldNotSupported,
    #[error("Relational rule contains invalid field {0}.")]
    InvalidField(String),
}

/// A collection of rules to run one round of scanning.
/// Rules will be grouped together based on their language, path globbing and pattern rule.
pub struct RuleCollection<L: Language + Eq> {
  // use vec since we don't have many languages
  /// a list of rule buckets grouped by languages.
  /// Tenured rules will always run against a file of that language type.
  tenured: Vec<RuleBucket<L>>,
  /// contingent rules will run against a file if it matches file/ignore glob.
  contingent: Vec<ContingentRule<L>>,
}

/// RuleBucket stores rules of the same language id.
/// Rules for different language will stay in separate buckets.
pub struct RuleBucket<L: Language> {
  rules: Vec<RuleConfig<L>>,
  lang: L,
}

#[derive(Serialize, Deserialize, Clone, Default, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
  #[default]
  /// Suggest improvements to the code.
  Hint,
  /// A firmer suggestion that code can be improved or optimized.
  Info,
  /// A warning that code might produce bugs or does not follow best practice.
  Warning,
  /// An error that code produces bugs or has logic errors.
  Error,
  /// Turns off the rule.
  Off,
}

#[derive(Debug, Error)]
pub enum RuleConfigError {
  #[error("Fail to parse yaml as RuleConfig")]
  Yaml(#[from] YamlError),
  #[error("Fail to parse yaml as Rule.")]
  Core(#[from] RuleCoreError),
  #[error("Rewriter rule `{1}` is not configured correctly.")]
  Rewriter(#[source] RuleCoreError, String),
  #[error("Undefined rewriter `{0}` used in transform.")]
  UndefinedRewriter(String),
  #[error("Rewriter rule `{0}` should have `fix`.")]
  NoFixInRewriter(String),
  #[error("Label meta-variable `{0}` must be defined in `rule` or `constraints`.")]
  LabelVariable(String),
  #[error("Rule must specify a set of AST kinds to match. Try adding `kind` rule.")]
  MissingPotentialKinds,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct SerializableRewriter {
  #[serde(flatten)]
  pub core: SerializableRuleCore,
  /// Unique, descriptive identifier, e.g., no-unused-variable
  pub id: String,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct SerializableRuleConfig<L: Language> {
  #[serde(flatten)]
  pub core: SerializableRuleCore,
  /// Unique, descriptive identifier, e.g., no-unused-variable
  pub id: String,
  /// Specify the language to parse and the file extension to include in matching.
  pub language: L,
  /// Rewrite rules for `rewrite` transformation
  pub rewriters: Option<Vec<SerializableRewriter>>,
  /// Main message highlighting why this rule fired. It should be single line and concise,
  /// but specific enough to be understood without additional context.
  #[serde(default)]
  pub message: String,
  /// Additional notes to elaborate the message and provide potential fix to the issue.
  /// `notes` can contain markdown syntax, but it cannot reference meta-variables.
  pub note: Option<String>,
  /// One of: hint, info, warning, or error
  #[serde(default)]
  pub severity: Severity,
  /// Custom label dictionary to configure reporting. Key is the meta-variable name and
  /// value is the label message and label style.
  pub labels: Option<FastMap<String, LabelConfig>>,
  /// Glob patterns to specify that the rule only applies to matching files
  pub files: Option<Vec<String>>,
  /// Glob patterns that exclude rules from applying to files
  pub ignores: Option<Vec<String>>,
  /// Documentation link to this rule
  pub url: Option<String>,
  /// Extra information for the rule
  pub metadata: Option<Metadata>,
}

/// A trivial wrapper around a FastMap to work around
/// the limitation of `serde_yaml::Value` not implementing `JsonSchema`.
#[derive(Serialize, Deserialize, Clone)]
pub struct Metadata(FastMap<String, serde_yaml::Value>);

pub struct RuleConfig<L: Language> {
  inner: SerializableRuleConfig<L>,
  pub matcher: RuleCore,
}

#[derive(Debug, Error)]
pub enum RuleCoreError {
    #[error("Fail to parse yaml as RuleConfig")]
    Yaml(#[from] YamlError),
    #[error("`utils` is not configured correctly.")]
    Utils(#[source] RuleSerializeError),
    #[error("`rule` is not configured correctly.")]
    Rule(#[from] RuleSerializeError),
    #[error("`constraints` is not configured correctly.")]
    Constraints(#[source] RuleSerializeError),
    #[error("`transform` is not configured correctly.")]
    Transform(#[from] TransformError),
    #[error("`fix` pattern is invalid.")]
    Fixer(#[from] FixerError),
    #[error("Undefined meta var `{0}` used in `{1}`.")]
    UndefinedMetaVar(String, &'static str),
}

// Used for global rules, rewriters, and pyo3/napi
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct SerializableRuleCore {
    /// A rule object to find matching AST nodes
    pub rule: SerializableRule,
    /// Additional meta variables pattern to filter matching
    pub constraints: Option<FastMap<String, SerializableRule>>,
    /// Utility rules that can be used in `matches`
    pub utils: Option<FastMap<String, SerializableRule>>,
    /// A dictionary for metavariable manipulation. Dict key is the new variable name.
    /// Dict value is a [transformation] that specifies how meta var is processed.
    /// See [transformation doc](https://ast-grep.github.io/reference/yaml/transformation.html).
    pub transform: Option<FastMap<String, Transformation>>,
    /// A pattern string or a FixConfig object to auto fix the issue.
    /// It can reference metavariables appeared in rule.
    /// See details in fix [object reference](https://ast-grep.github.io/reference/yaml/fix.html#fixconfig).
    pub fix: Option<SerializableFixer>,
}

pub struct RuleCore {
    rule: Rule,
    constraints: FastMap<String, Rule>,
    kinds: Option<BitSet>,
    pub(crate) transform: Option<Transform>,
    pub fixer: Vec<Fixer>,
    // this is required to hold util rule reference
    registration: RuleRegistration,
}

// NB StopBy's JsonSchema is changed in xtask/schema.rs
// revise schema is easier than manually implementation
#[derive(Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SerializableStopBy {
  #[default]
  Neighbor,
  End,
  Rule(Box<SerializableRule>),
}

pub enum StopBy {
  Neighbor,
  End,
  Rule(Rule),
}

pub struct Inside {
    outer: Rule,
    field: Option<u16>,
    stop_by: StopBy,
}

pub struct Has {
    inner: Rule,
    stop_by: StopBy,
    field: Option<u16>,
}

pub struct Precedes {
    later: Rule,
    stop_by: StopBy,
}

pub struct Follows {
    former: Rule,
    stop_by: StopBy,
}

pub struct Registration<R>(Arc<FastMap<String, R>>);

#[derive(Clone, Default)]
pub struct RuleRegistration {
  /// utility rule to every RuleCore, every sub-rule has its own local utility
  local: Registration<Rule>,
  /// global rules are shared by all RuleConfigs. It is a singleton.
  global: Registration<RuleCore>,
  /// Every RuleConfig has its own rewriters. But sub-rules share parent's rewriters.
  rewriters: Registration<RuleCore>,
}

/// RegistrationRef must use Weak pointer to avoid
/// cyclic reference in RuleRegistration
struct RegistrationRef {
  local: Weak<FastMap<String, Rule>>,
  global: Weak<FastMap<String, RuleCore>>,
}

#[derive(Debug, Error)]
pub enum ReferentRuleError {
  #[error("Rule `{0}` is not defined.")]
  UndefinedUtil(String),
  #[error("Duplicate rule id `{0}` is found.")]
  DuplicateRule(String),
  #[error("Rule `{0}` has a cyclic dependency in its `matches` sub-rule.")]
  CyclicRule(String),
}

pub struct ReferentRule {
  pub(crate) rule_id: String,
  reg_ref: RegistrationRef,
}

pub type GlobalRules = Registration<RuleCore>;

/// Represents a zero-based character-wise position in a document
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct SerializablePosition {
  /// 0-based line number in the source code
  pub line: usize,
  /// 0-based column number in the source code
  pub column: usize,
}

/// Represents a position in source code using 0-based line and column numbers
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct SerializableRange {
  /// start position in the source code
  pub start: SerializablePosition,
  /// end position in the source code
  pub end: SerializablePosition,
}

/// Errors that can occur when creating or using a RangeMatcher
#[derive(Debug, Error)]
pub enum RangeMatcherError {
  /// Returned when the range is invalid. This can occur when:
  /// - start position is after end position
  /// - positions contain invalid line/column values
  #[error("The start position must be before the end position.")]
  InvalidRange,
}

pub struct RangeMatcher {
  start: SerializablePosition,
  end: SerializablePosition,
}

#[derive(Debug, Error)]
pub enum NthChildError {
  #[error("Illegal character {0} encountered")]
  IllegalCharacter(char),
  #[error("Invalid syntax")]
  InvalidSyntax,
  #[error("Invalid ofRule")]
  InvalidRule(#[from] Box<RuleSerializeError>),
}

/// A string or number describing the indices of matching nodes in a list of siblings.
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum NthChildSimple {
  /// A number indicating the precise element index
  Numeric(usize),
  /// Functional notation like CSS's An + B
  Functional(String),
}

/// `nthChild` accepts either a number, a string or an object.
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(untagged, rename_all = "camelCase")]
pub enum SerializableNthChild {
  /// Simple syntax
  Simple(NthChildSimple),
  /// Object style syntax
  #[serde(rename_all = "camelCase")]
  Complex {
    /// nth-child syntax
    position: NthChildSimple,
    /// select the nth node that matches the rule, like CSS's of syntax
    of_rule: Option<Box<SerializableRule>>,
    /// matches from the end instead like CSS's nth-last-child
    #[serde(default)]
    reverse: bool,
  },
}

pub struct NthChild {
  position: FunctionalPosition,
  of_rule: Option<Box<Rule>>,
  reverse: bool,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct SerializableGlobalRule<L: Language> {
  #[serde(flatten)]
  pub core: SerializableRuleCore,
  /// Unique, descriptive identifier, e.g., no-unused-variable
  pub id: String,
  /// Specify the language to parse and the file extension to include in matching.
  pub language: L,
}

/// A struct to store information to deserialize rules.
#[derive(Clone)]
pub struct DeserializeEnv<L: Language> {
  /// registration for global utility rules and local utility rules.
  pub(crate) registration: RuleRegistration,
  /// current rules' language
  pub(crate) lang: L,
}
