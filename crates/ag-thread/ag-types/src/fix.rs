/// A pattern string or fix object to auto fix the issue.
/// It can reference metavariables appeared in rule.
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum SerializableFixer {
    Str(String),
    Config(Box<SerializableFixConfig>),
    List(Vec<SerializableFixConfig>),
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SerializableFixConfig {
    template: String,
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    expand_end: Maybe<Relation>,
    #[serde(default, skip_serializing_if = "Maybe::is_absent")]
    expand_start: Maybe<Relation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

// two lifetime to represent env root lifetime and lang/trans lifetime
struct Ctx<'b, 'c, D: Doc> {
    rewriters: &'b FastMap<String, RuleCore>,
    env: &'b mut MetaVarEnv<'c, D>,
    enclosing_env: &'b MetaVarEnv<'c, D>,
}

#[derive(Debug, Error)]
pub enum FixerError {
    #[error("Fixer template is invalid.")]
    InvalidTemplate(#[from] TemplateFixError),
    #[error("Fixer expansion contains invalid rule.")]
    WrongExpansion(#[from] RuleSerializeError),
    #[error("Rewriter must have exactly one fixer.")]
    InvalidRewriter,
    #[error("Fixer in list must have title.")]
    MissingTitle,
}

#[derive(Debug, Error)]
pub enum TransformError {
    #[error("Cannot parse transform string.")]
    Parse(#[from] ParseTransError),
    #[error("`{0}` has a cyclic dependency.")]
    Cyclic(String),
    #[error("Transform var `{0}` has already defined.")]
    AlreadyDefined(String),
    #[error("source `{0}` should be $-prefixed.")]
    MalformedVar(String),
}

pub struct Transform {
    transforms: Vec<(String, Trans<MetaVariable>)>,
}

struct Expansion {
    matches: Rule,
    stop_by: StopBy,
}

pub struct Fixer {
    template: TemplateFix,
    expand_start: Option<Expansion>,
    expand_end: Option<Expansion>,
    title: Option<String>,
}

/// Extracts a substring from the meta variable's text content.
///
/// Both `start_char` and `end_char` support negative indexing,
/// which counts character from the end of an array, moving backwards.
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Substring<T> {
  /// source meta variable to be transformed
  pub source: T,
  /// optional starting character index of the substring, defaults to 0.
  pub start_char: Option<i32>,
  /// optional ending character index of the substring, defaults to the end of the string.
  pub end_char: Option<i32>,
}

/// An enumeration representing different cases for strings.
#[derive(Serialize, Deserialize, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StringCase {
  LowerCase,
  UpperCase,
  Capitalize,
  CamelCase,
  SnakeCase,
  KebabCase,
  PascalCase,
}

#[derive(Serialize, Deserialize, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// Separator to split string. e.g. `user_accountName` -> `user`, `accountName`
/// It will be rejoin according to `StringCase`.
pub enum Separator {
  CaseChange,
  Dash,
  Dot,
  Slash,
  Space,
  Underscore,
}

#[derive(PartialEq, Eq)]
/// CaseState is used to record the case change between two characters.
/// It will be used if separator is CaseChange.
enum CaseState {
  Lower,
  OneUpper,
  /// MultiUpper records consecutive uppercase characters.
  /// char is the last uppercase char, used to calculate the split range.
  MultiUpper(char),
  IgnoreCase,
}

struct Delimiter {
  left: usize,
  right: usize,
  state: CaseState,
  delimiter: Vec<char>,
}

/// Replaces a substring in the meta variable's text content with another string.
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Replace<T> {
  /// source meta variable to be transformed
  pub source: T,
  /// a regex to find substring to be replaced
  pub replace: String,
  /// the replacement string
  pub by: String,
}

/// Converts the source meta variable's text content to a specified case format.
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Convert<T> {
  /// source meta variable to be transformed
  pub source: T,
  /// the target case format to convert the text content to
  pub to_case: StringCase,
  /// optional separators to specify how to separate word
  pub separated_by: Option<Vec<Separator>>,
}

/// Represents a transformation that can be applied to a matched AST node.
/// Available transformations are `substring`, `replace` and `convert`.
#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Trans<T> {
  Substring(Substring<T>),
  Replace(Replace<T>),
  Convert(Convert<T>),
  Rewrite(Rewrite<T>),
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Rewrite<T> {
    pub source: T,
    pub rewriters: Vec<String>,
    // do we need this?
    // sort_by: Option<String>,
    pub join_by: Option<String>,
}

#[derive(Debug, Error)]
pub enum ParseTransError {
  #[error("`{0}` has syntax error.")]
  Syntax(String),
  #[error("`{0}` is not a valid transformation.")]
  InvalidTransform(String),
  #[error("`{0}` is not a valid argument.")]
  InvalidArg(String),
  #[error("Argument `{0}` is required.")]
  RequiredArg(&'static str),
  #[error("Invalid argument value.")]
  ArgValue(#[from] serde_yaml::Error),
}

struct DecomposedTransString<'a> {
  func: &'a str,
  source: &'a str,
  args: Vec<(&'a str, &'a str)>,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(untagged)]
pub enum Transformation {
    Simplied(String),
    Object(Trans<String>),
}
