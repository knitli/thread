use super::{Diff, NodeMatch, PrintProcessor, Printer};
use thread_threadlang::ThreadLang;
use ag_service_rule::{RuleConfig, Severity};

use anyhow::Result;
use codespan_reporting::files::SimpleFile;
use std::io::{Stdout, Write};

use std::borrow::Cow;
use std::path::{Path, PathBuf};

#[derive(PartialEq, Eq, Clone)]
pub enum Platform {
  GitHub,
}

pub struct CloudPrinter<W: Write> {
  writer: W,
}

impl<W: Write> CloudPrinter<W> {
  pub fn new(writer: W) -> Self {
    Self { writer }
  }
}

impl CloudPrinter<Stdout> {
  pub fn stdout() -> Self {
    Self::new(std::io::stdout())
  }
}
impl<W: Write> Printer for CloudPrinter<W> {
  type Processed = Vec<u8>;
  type Processor = CloudProcessor;

  fn get_processor(&self) -> Self::Processor {
    CloudProcessor
  }

  fn process(&mut self, processed: Self::Processed) -> Result<()> {
    self.writer.write_all(&processed)?;
    Ok(())
  }
}

pub struct CloudProcessor;

impl PrintProcessor<Vec<u8>> for CloudProcessor {
  fn print_rule(
    &self,
    matches: Vec<NodeMatch>,
    file: SimpleFile<Cow<str>, &str>,
    rule: &RuleConfig<ThreadLang>,
  ) -> Result<Vec<u8>> {
    let mut ret = vec![];
    let path = PathBuf::from(file.name().to_string());
    for m in matches {
      print_rule(&mut ret, m, &path, rule)?;
    }
    Ok(ret)
  }

  fn print_matches(&self, _m: Vec<NodeMatch>, _p: &Path) -> Result<Vec<u8>> {
    unreachable!("cloud printer does not support pattern search")
  }

  fn print_diffs(&self, _d: Vec<Diff>, _p: &Path) -> Result<Vec<u8>> {
    unreachable!("cloud printer does not support pattern rewrite")
  }

  fn print_rule_diffs(
    &self,
    diffs: Vec<(Diff<'_>, &RuleConfig<ThreadLang>)>,
    path: &Path,
  ) -> Result<Vec<u8>> {
    let mut ret = vec![];
    for (diff, rule) in diffs {
      print_rule(&mut ret, diff.node_match, path, rule)?;
    }
    Ok(ret)
  }
}

fn print_rule<W: Write>(
  writer: &mut W,
  m: NodeMatch,
  path: &Path,
  rule: &RuleConfig<ThreadLang>,
) -> Result<()> {
  let level = match rule.severity {
    Severity::Error => "error",
    Severity::Warning => "warning",
    Severity::Info => "notice",
    Severity::Hint => return Ok(()),
    Severity::Off => unreachable!("turned-off rule should not have match."),
  };
  let title = &rule.id;
  let name = path.display();
  let line = m.start_pos().line() + 1;
  let end_line = m.end_pos().line() + 1;
  let message = rule.get_message(&m);
  writeln!(
    writer,
    "::{level} file={name},line={line},endLine={end_line},title={title}::{message}"
  )?;
  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;
  use ag_service_rule::{from_yaml_string, GlobalRules};
  use thread_languages::{LanguageExt, SupportedLanguage};
  use codespan_reporting::term::termcolor::Buffer;

  fn make_test_printer() -> CloudPrinter<Buffer> {
    CloudPrinter::new(Buffer::no_color())
  }
  fn get_text(printer: &mut CloudPrinter<Buffer>) -> String {
    let buffer = &mut printer.writer;
    let bytes = buffer.as_slice();
    std::str::from_utf8(bytes)
      .expect("buffer should be valid utf8")
      .to_owned()
  }

  fn make_rule(rule: &str) -> RuleConfig<ThreadLang> {
    let globals = GlobalRules::default();
    from_yaml_string(
      &format!(
        r"
id: test
message: test rule
language: TypeScript
{rule}"
      ),
      &globals,
    )
    .unwrap()
    .pop()
    .unwrap()
  }

  fn test_output(src: &str, rule_str: &str, expect: &str) {
    let mut printer = make_test_printer();
    let grep = ThreadLang::from(SupportedLanguage::Tsx).ast_grep(src);
    let rule = make_rule(rule_str);
    let matches = grep.root().find_all(&rule.matcher).collect();
    let file = SimpleFile::new(Cow::Borrowed("test.tsx"), src);
    let buffer = printer
      .get_processor()
      .print_rule(matches, file, &rule)
      .unwrap();
    printer.process(buffer).expect("should work");
    let actual = get_text(&mut printer);
    assert_eq!(actual, expect);
  }

  #[test]
  fn test_no_match_output() {
    test_output("let a = 123", "rule: { pattern: console }", "");
    test_output(
      "let a = 123",
      "
rule: { pattern: console }
severity: error",
      "",
    );
  }

  #[test]
  fn test_hint_output() {
    test_output(
      "console.log(123)",
      "
rule: { pattern: console }
severity: hint
",
      "",
    );
  }

  #[test]
  fn test_info_output() {
    test_output(
      "console.log(123)",
      "
rule: { pattern: console }
severity: info
",
      "::notice file=test.tsx,line=1,endLine=1,title=test::test rule\n",
    );
  }

  #[test]
  fn test_warning_output() {
    test_output(
      "console.log(123)",
      "
rule: { pattern: console }
severity: warning
",
      "::warning file=test.tsx,line=1,endLine=1,title=test::test rule\n",
    );
  }

  #[test]
  fn test_error_output() {
    test_output(
      "console.log(123)",
      "
rule: { pattern: console }
severity: error
",
      "::error file=test.tsx,line=1,endLine=1,title=test::test rule\n",
    );
  }
}
