---
description:
globs:
alwaysApply: false
---

# use ast-grep to search code

Your task is to help users to write ast-grep rules to search code.
User will query you by natural language, and you will write ast-grep rules to search code.

You need to translate user's query into ast-grep rules.
And use ast-grep-mcp to develop a rule, test the rule and then search the codebase.

## General Process

1. Clearly understand the user's query. Clarify any ambiguities and if needed, ask user for more details.
2. Write a simple example code snippet that matches the user's query.
3. Write an ast-grep rule that matches the example code snippet.
4. Test the rule against the example code snippet to ensure it matches. Use ast-grep mcp tool `test_match_code_rule` to verify the rule.
   a. if the rule does not match, revise the rule by removing some sub rules and debugging unmatching parts.
   b. if you are using `inside` or `has` relational rules, ensure to use `stopBy: end` to ensure the search goes to the end of the direction.
5. Use the ast-grep mcp tool to search code using the rule.

## Tips for Writing Rules

0. always use `stopBy: end` for relational rules to ensure the search goes to the end of the direction.

```yaml
has:
  pattern: await $EXPR
  stopBy: end
```

1. if relational rules are used but no match is found, try adding `stopBy: end` to the relational rule to ensure it searches to the end of the direction.
2. use pattern only if the code structure is simple and does not require complex matching (e.g. matching function calls, variable names, etc.).
3. use rule if the code structure is complex and can be broken down into smaller parts (e.g. find call inside certain function).
4. if pattern is not working, try using `kind` to match the node type first, then use `has` or `inside` to match the code structure.

## Rule Development Process

1. Break down the user's query into smaller parts.
2. Identify sub rules that can be used to match the code.
3. Combine the sub rules into a single rule using relational rules or composite rules.
4. if rule does not match example code, revise the rule by removing some sub rules and debugging unmatching parts.
5. Use ast-grep mcp tool to dump AST or dump pattern query
6. Use ast-grep mcp tool to test the rule against the example code snippet.

## ast-grep mcp tool usage

ast-grep mcp has several tools:

- dump_syntax_tree will dump the AST of the code, this is useful for debugging and understanding the code structure and patterns
- test_match_code_rule will test a rule agains a code snippet, this is useful to ensure the rule matches the code

## Rule Format

# ast-grep Rule Documentation for Claude Code

## 1. Introduction to ast-grep Rules

ast-grep rules are declarative specifications for matching and filtering Abstract Syntax Tree (AST) nodes. They enable structural code search and analysis by defining conditions an AST node must meet to be matched.

### 1.1 Overview of Rule Categories

ast-grep rules are categorized into three types for modularity and comprehensive definition :

- **Atomic Rules**: Match individual AST nodes based on intrinsic properties like code patterns (`pattern`), node type (`kind`), or text content (`regex`).
- **Relational Rules**: Define conditions based on a target node's position or relationship to other nodes (e.g., `inside`, `has`, `precedes`, `follows`).
- **Composite Rules**: Combine other rules using logical operations (AND, OR, NOT) to form complex matching criteria (e.g., `all`, `any`, `not`, `matches`).

## 2. Anatomy of an ast-grep Rule Object

The ast-grep rule object is the core configuration unit defining how ast-grep identifies and filters AST nodes. It's typically a YAML.

### 2.1 General Structure and Optionality

Every field within an ast-grep Rule Object is optional, but at least one "positive" key (e.g., `kind`, `pattern`) must be present.

A node matches a rule if it satisfies all fields defined within that rule object, implying an implicit logical AND operation.

For rules using metavariables that depend on prior matching, explicit `all` composite rules are recommended to guarantee execution order.

**Table 1: ast-grep Rule Object Properties Overview**

| Property   | Type                   | Category   | Purpose                                                | Example                                                                  |
| :--------- | :--------------------- | :--------- | :----------------------------------------------------- | :----------------------------------------------------------------------- |
| `pattern`  | String or Object       | Atomic     | Matches AST node by code pattern.                      | `pattern: console.log($ARG)`                                             |
| `kind`     | String                 | Atomic     | Matches AST node by its kind name.                     | `kind: call_expression`                                                  |
| `regex`    | String                 | Atomic     | Matches node's text by Rust regex.                     | `regex: ^[a-z]+$`                                                        |
| `nthChild` | number, string, Object | Atomic     | Matches nodes by their index within parent's children. | `nthChild: 1`                                                            |
| `range`    | RangeObject            | Atomic     | Matches node by character-based start/end positions.   | `range: { start: { line: 0, column: 0 }, end: { line: 0, column: 10 } }` |
| `inside`   | Object                 | Relational | Target node must be inside node matching sub-rule.     | `inside: { pattern: class $C { $$$ }, stopBy: end }`                     |
| `has`      | Object                 | Relational | Target node must have descendant matching sub-rule.    | `has: { pattern: await $EXPR, stopBy: end }`                             |
| `precedes` | Object                 | Relational | Target node must appear before node matching sub-rule. | `precedes: { pattern: return $VAL }`                                     |
| `follows`  | Object                 | Relational | Target node must appear after node matching sub-rule.  | `follows: { pattern: import $M from '$P' }`                              |
| `all`      | Array<Rule>            | Composite  | Matches if all sub-rules match.                        | `all: [ { kind: call_expression }, { pattern: foo($A) } ]`               |
| `any`      | Array<Rule>            | Composite  | Matches if any sub-rules match.                        | `any: [ { pattern: foo() }, { pattern: bar() } ]`                        |
| `not`      | Object                 | Composite  | Matches if sub-rule does not match.                    | `not: { pattern: console.log($ARG) }`                                    |
| `matches`  | String                 | Composite  | Matches if predefined utility rule matches.            | `matches: my-utility-rule-id`                                            |

## 3. Atomic Rules: Fundamental Matching Building Blocks

Atomic rules match individual AST nodes based on their intrinsic properties.

### 3.1 `pattern`: String and Object Forms

The `pattern` rule matches a single AST node based on a code pattern.

- **String Pattern**: Directly matches using ast-grep's pattern syntax with metavariables.
  - Example: `pattern: console.log($ARG)`
- **Object Pattern**: Offers granular control for ambiguous patterns or specific contexts.

  - `selector`: Pinpoints a specific part of the parsed pattern to match.

    ```yaml
    pattern:
      selector: field_definition
      context: class { $F }
    ```

  - `context`: Provides surrounding code context for correct parsing.
  - `strictness`: Modifies the pattern's matching algorithm (`cst`, `smart`, `ast`, `relaxed`, `signature`).

    ```yaml
    pattern:
      context: foo($BAR)
      strictness: relaxed
    ```

### 3.2 `kind`: Matching by Node Type

The `kind` rule matches an AST node by its `tree_sitter_node_kind` name, derived from the language's Tree-sitter grammar. Useful for targeting constructs like `call_expression` or `function_declaration`.

- Example: `kind: call_expression`

### 3.3 `regex`: Text-Based Node Matching

The `regex` rule matches the entire text content of an AST node using a Rust regular expression. It's not a "positive" rule, meaning it matches any node whose text satisfies the regex, regardless of its structural kind.

### 3.4 `nthChild`: Positional Node Matching

The `nthChild` rule finds nodes by their 1-based index within their parent's children list, counting only named nodes by default.

- `number`: Matches the exact nth child. Example: `nthChild: 1`
- `string`: Matches positions using An+B formula. Example: `2n+1`
- `Object`: Provides granular control:
  - `position`: `number` or An+B string.
  - `reverse`: `true` to count from the end.
  - `ofRule`: An ast-grep rule to filter the sibling list before counting.

### 3.5 `range`: Position-Based Node Matching

The `range` rule matches an AST node based on its character-based start and end positions. A `RangeObject` defines `start` and `end` fields, each with 0-based `line` and `column`. `start` is inclusive, `end` is exclusive.

## 4. Relational Rules: Contextual and Hierarchical Matching

Relational rules filter targets based on their position relative to other AST nodes. They can include `stopBy` and `field` options.

---

### 4.1 `inside`: Matching Within a Parent Node

Requires the target node to be inside another node matching the `inside` sub-rule.

- Example:

  ```yaml
  inside:
    pattern: class $C { $$$ }
    stopBy: end
  ```

### 4.2 `has`: Matching with a Descendant Node

Requires the target node to have a descendant node matching the `has` sub-rule.

- Example:

  ```yaml
  has:
    pattern: await $EXPR
    stopBy: end
  ```

### 4.3 `precedes` and `follows`: Sequential Node Matching

- `precedes`: Target node must appear before a node matching the `precedes` sub-rule.
- `follows`: Target node must appear after a node matching the `follows` sub-rule.

Both include `stopBy` but not `field`.

### 4.4 `stopBy` and `field`: Refining Relational Searches

- `stopBy`: Controls search termination for relational rules.
  - `"neighbor"` (default): Stops when immediate surrounding node doesn't match.
  - `"end"`: Searches to the end of the direction (root for `inside`, leaf for `has`).
  - `Rule object`: Stops when a surrounding node matches the provided rule (inclusive).
- `field`: Specifies a sub-node within the target node that should match the relational rule. Only for `inside` and `has`.

When you are not sure, always use `stopBy: end` to ensure the search goes to the end of the direction.

## 5. Composite Rules: Logical Combination of Conditions

Composite rules combine atomic and relational rules using logical operations.

### 5.1 `all`: Conjunction (AND) of Rules

Matches a node only if all sub-rules in the list match. Guarantees order of rule matching, important for metavariables.

- Example:

  ```yaml
  all:
    - kind: call_expression
    - pattern: console.log($ARG)
  ```

### 5.2 `any`: Disjunction (OR) of Rules

Matches a node if any sub-rules in the list match.

- Example:

  ```yaml
  any:
    - pattern: console.log($ARG)
    - pattern: console.warn($ARG)
    - pattern: console.error($ARG)
  ```

### 5.3 `not`: Negation (NOT) of a Rule

Matches a node if the single sub-rule does not match.

- Example:

  ```yaml
  not:
    pattern: console.log($ARG)
  ```

### 5.4 `matches`: Rule Reuse and Utility Rules

Takes a rule-id string, matching if the referenced utility rule matches. Enables rule reuse and recursive rules.

## 6. Metavariables: Dynamic Content Matching

Metavariables are placeholders in patterns to match dynamic content in the AST.

### 6.1 `$VAR`: Single Named Node Capture

Captures a single named node in the AST.

- **Valid**: `$META`, `$META_VAR`, `$_`
- **Invalid**: `$invalid`, `$123`, `$KEBAB-CASE`
- **Example**: `console.log($GREETING)` matches `console.log('Hello World')`.
- **Reuse**: `$A == $A` matches `a == a` but not `a == b`.

### 6.2 `$$VAR`: Single Unnamed Node Capture

Captures a single unnamed node (e.g., operators, punctuation).

- **Example**: To match the operator in `a + b`, use `$$OP`.

  ```yaml
  rule:
    kind: binary_expression
    has:
      field: operator
      pattern: $$OP
  ```

### 6.3 `$$$MULTI_META_VARIABLE`: Multi-Node Capture

Matches zero or more AST nodes (non-greedy). Useful for variable numbers of arguments or statements.

- **Example**: `console.log($$$)` matches `console.log()`, `console.log('hello')`, and `console.log('debug:', key, value)`.
- **Example**: `function $FUNC($$$ARGS) { $$$ }` matches functions with varying parameters/statements.

### 6.4 Non-Capturing Metavariables (`_VAR`)

Metavariables starting with an underscore (`_`) are not captured. They can match different content even if named identically, optimizing performance.

- **Example**: `$_FUNC($_FUNC)` matches `test(a)` and `testFunc(1 + 1)`.

### 6.5 Important Considerations for Metavariable Detection

- **Syntax Matching**: Only exact metavariable syntax (e.g., `$A`, `$$B`, `$$$C`) is recognized.
- **Exclusive Content**: Metavariable text must be the only text within an AST node.
- **Non-working**: `obj.on$EVENT`, `"Hello $WORLD"`, `a $OP b`, `$jq`.

The ast-grep playground is useful for debugging patterns and visualizing metavariables.
