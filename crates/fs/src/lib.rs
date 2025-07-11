// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later


use std::path::Path;

use anyhow::{Context, Result};

use ignore::WalkParallel;
use thread_ag::{MatchStrictness, Matcher, Pattern};
use thread_ag::{Language, LanguageExt};

use crate::config::ProjectConfig;
use thread_threadlang::ThreadLang;
use crate::utils::ErrorContext as EC;
use crate::utils::{ContextArgs, InputArgs, MatchUnit, OutputArgs, filter_file_pattern};
use crate::utils::{DebugFormat, FileTrace, RunTrace};
use crate::utils::{Items, PathWorker, StdInWorker, Worker};

fn lang_help() -> String {
    format!(
        "The language of the pattern. Supported languages are:\n{:?}",
        ThreadLang::all_languages()
    )
}

const LANG_HELP_LONG: &str =
    "The language of the pattern. For full language list, see [`SupportedLanguage`](https://github.com/knitli/thread/blobs)";

#[derive(Clone)]
struct Strictness(MatchStrictness);
impl ValueEnum for Strictness {
    fn value_variants<'a>() -> &'a [Self] {
        use MatchStrictness as M;
        &[
            Strictness(M::Cst),
            Strictness(M::Smart),
            Strictness(M::Ast),
            Strictness(M::Relaxed),
            Strictness(M::Signature),
        ]
    }
    fn to_possible_value(&self) -> Option<PossibleValue> {
        use MatchStrictness as M;
        Some(match &self.0 {
            M::Cst => PossibleValue::new("cst").help("Match exact all node"),
            M::Smart => {
                PossibleValue::new("smart").help("Match all node except source trivial nodes")
            }
            M::Ast => PossibleValue::new("ast").help("Match only ast nodes"),
            M::Relaxed => PossibleValue::new("relaxed").help("Match ast node except comments"),
            M::Signature => {
                PossibleValue::new("signature").help("Match ast node except comments, without text")
            }
        })
    }
}


impl RunArg {
    fn build_pattern(&self, lang: ThreadLang) -> Result<Pattern> {
        let pattern = if let Some(sel) = &self.selector {
            Pattern::contextual(&self.pattern, sel, lang)
        } else {
            Pattern::try_new(&self.pattern, lang)
        }
        .context(EC::ParsePattern)?;
        if let Some(strictness) = &self.strictness {
            Ok(pattern.with_strictness(strictness.0.clone()))
        } else {
            Ok(pattern)
        }
    }

    // do not unwrap pattern here, we should allow non-pattern to be debugged as tree
    fn debug_pattern_if_needed(&self, pattern_ret: &Result<Pattern>, lang: ThreadLang) {
        let Some(debug_query) = &self.debug_query else {
            return;
        };
        let colored = self.output.color.should_use_color();
        if !matches!(debug_query, DebugFormat::Pattern) {
            debug_query.debug_tree(&self.pattern, lang, colored);
        } else if let Ok(pattern) = pattern_ret {
            debug_query.debug_pattern(pattern, lang, colored);
        }
    }
}

// Every run will include Search or Replace
// Search or Replace by arguments `pattern` and `rewrite` passed from CLI
pub fn run_with_pattern(arg: RunArg, project: Result<ProjectConfig>) -> Result<()> {
    let proj = arg.output.inspect.project_trace();
    proj.print_project(&project)?;
    let context = arg.context.get();
    if let Some(json) = arg.output.json {
        let printer = JSONPrinter::stdout(json).context(context);
        return run_pattern_with_printer(arg, printer);
    }
    let printer = ColoredPrinter::stdout(arg.output.color)
        .heading(arg.heading)
        .context(context);
    let interactive = arg.output.needs_interactive();
    if interactive {
        let from_stdin = arg.input.stdin;
        let printer = InteractivePrinter::new(printer, arg.output.update_all, from_stdin)?;
        run_pattern_with_printer(arg, printer)
    } else {
        run_pattern_with_printer(arg, printer)
    }
}

fn run_pattern_with_printer(arg: RunArg, printer: impl Printer + 'static) -> Result<()> {
    let trace = arg.output.inspect.run_trace();
    if arg.input.stdin {
        RunWithSpecificLang::new(arg, trace)?.run_std_in(printer)
    } else if arg.lang.is_some() {
        RunWithSpecificLang::new(arg, trace)?.run_path(printer)
    } else {
        RunWithInferredLang { arg, trace }.run_path(printer)
    }
}

struct RunWithInferredLang {
    arg: RunArg,
    trace: RunTrace,
}
impl Worker for RunWithInferredLang {
    fn consume_items<P: Printer>(&self, items: Items<P::Processed>, mut printer: P) -> Result<()> {
        let printer = &mut printer;
        printer.before_print()?;
        for item in items {
            printer.process(item)?;
        }
        printer.after_print()?;
        self.trace.print()?;
        Ok(())
    }
}

impl PathWorker for RunWithInferredLang {
    fn build_walk(&self) -> Result<WalkParallel> {
        self.arg.input.walk()
    }
    fn get_trace(&self) -> &FileTrace {
        &self.trace.inner
    }

    fn produce_item<P: Printer>(
        &self,
        path: &Path,
        processor: &P::Processor,
    ) -> Result<Vec<P::Processed>> {
        let Some(lang) = ThreadLang::from_path(path) else {
            return Ok(vec![]);
        };
        self.trace.print_file(path, lang)?;
        let matcher = self.arg.build_pattern(lang)?;
        // match sub region
        let sub_languages = lang.injectable_sg_languages().into_iter().flatten();
        let sub_matchers = sub_languages
            .filter_map(|l| {
                let maybe_pattern = self.arg.build_pattern(l);
                maybe_pattern.ok().map(|pattern| (l, pattern))
            })
            .collect::<Vec<_>>();

        let items = filter_file_pattern(path, lang, Some(&matcher), &sub_matchers)?;
        let mut ret = Vec::with_capacity(items.len());
        let rewrite_str = self.arg.rewrite.as_ref();

        for unit in items {
            let i_lang = unit.grep.lang();
            let rewrite = rewrite_str
                .map(|s| Fixer::from_str(s, i_lang))
                .transpose()
                .unwrap_or_else(|e| {
                    eprintln!(
                        "⚠️  Rewriting was skipped because pattern fails to parse. Error detail:"
                    );
                    eprintln!("╰▻ {e}");
                    None
                });
            let Some(processed) = match_one_file(processor, &unit, &rewrite)? else {
                continue;
            };
            ret.push(processed);
        }
        Ok(ret)
    }
}

struct RunWithSpecificLang {
    arg: RunArg,
    pattern: Pattern,
    rewrite: Option<Fixer>,
    stats: RunTrace,
}

impl RunWithSpecificLang {
    fn new(arg: RunArg, stats: RunTrace) -> Result<Self> {
        let lang = arg.lang.ok_or(anyhow::anyhow!(EC::LanguageNotSpecified))?;
        // do not unwrap result here
        let pattern_ret = arg.build_pattern(lang);
        arg.debug_pattern_if_needed(&pattern_ret, lang);
        let rewrite = if let Some(s) = &arg.rewrite {
            Some(Fixer::from_str(s, &lang).context(EC::ParsePattern)?)
        } else {
            None
        };
        Ok(Self {
            arg,
            pattern: pattern_ret?,
            rewrite,
            stats,
        })
    }
}

impl Worker for RunWithSpecificLang {
    fn consume_items<P: Printer>(&self, items: Items<P::Processed>, mut printer: P) -> Result<()> {
        printer.before_print()?;
        let mut has_matches = false;
        for item in items {
            printer.process(item)?;
            has_matches = true;
        }
        printer.after_print()?;
        self.stats.print()?;
        if !has_matches && self.pattern.has_error() {
            Err(anyhow::anyhow!(EC::PatternHasError))
        } else {
            Ok(())
        }
    }
}

impl PathWorker for RunWithSpecificLang {
    fn build_walk(&self) -> Result<WalkParallel> {
        let lang = self.arg.lang.expect("must present");
        self.arg.input.walk_lang(lang)
    }
    fn get_trace(&self) -> &FileTrace {
        &self.stats.inner
    }
    fn produce_item<P: Printer>(
        &self,
        path: &Path,
        processor: &P::Processor,
    ) -> Result<Vec<P::Processed>> {
        let arg = &self.arg;
        let pattern = &self.pattern;
        let lang = arg.lang.expect("must present");
        let Some(path_lang) = ThreadLang::from_path(path) else {
            return Ok(vec![]);
        };
        self.stats.print_file(path, path_lang)?;
        let (root_matcher, sub_matchers) = if path_lang == lang {
            (Some(pattern), vec![])
        } else {
            (None, vec![(lang, pattern.clone())])
        };
        let filtered = filter_file_pattern(path, path_lang, root_matcher, &sub_matchers)?;
        let mut ret = Vec::with_capacity(filtered.len());
        for unit in filtered {
            let Some(processed) = match_one_file(processor, &unit, &self.rewrite)? else {
                continue;
            };
            ret.push(processed);
        }
        Ok(ret)
    }
}

impl StdInWorker for RunWithSpecificLang {
    fn parse_stdin<P: Printer>(
        &self,
        src: String,
        processor: &P::Processor,
    ) -> Result<Vec<P::Processed>> {
        let lang = self.arg.lang.expect("must present");
        let grep = lang.ast_grep(src);
        let root = grep.root();
        let mut matches = root.find_all(&self.pattern).peekable();
        if matches.peek().is_none() {
            return Ok(vec![]);
        }
        let rewrite = &self.rewrite;
        let path = Path::new("STDIN");
        let processed = if let Some(rewrite) = rewrite {
            let diffs = matches.map(|m| Diff::generate(m, &self.pattern, rewrite));
            processor.print_diffs(diffs.collect(), path)?
        } else {
            processor.print_matches(matches.collect(), path)?
        };
        Ok(vec![processed])
    }
}
fn match_one_file<T, P: PrintProcessor<T>>(
    processor: &P,
    match_unit: &MatchUnit<impl Matcher>,
    rewrite: &Option<Fixer>,
) -> Result<Option<T>> {
    let MatchUnit {
        path,
        grep,
        matcher,
    } = match_unit;

    let root = grep.root();
    let mut matches = root.find_all(matcher).peekable();
    if matches.peek().is_none() {
        return Ok(None);
    }
    let ret = if let Some(rewrite) = rewrite {
        let diffs = matches.map(|m| Diff::generate(m, matcher, rewrite));
        processor.print_diffs(diffs.collect(), path)?
    } else {
        processor.print_matches(matches.collect(), path)?
    };
    Ok(Some(ret))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::print::ColorArg;
    use ast_grep_language::SupportedLanguage;
    use std::path::PathBuf;

    fn default_run_arg() -> RunArg {
        RunArg {
            pattern: String::new(),
            selector: None,
            rewrite: None,
            lang: None,
            heading: Heading::Never,
            debug_query: None,
            strictness: None,
            input: InputArgs {
                no_ignore: vec![],
                stdin: false,
                follow: false,
                paths: vec![PathBuf::from(".")],
                globs: vec![],
                threads: 0,
            },
            output: OutputArgs {
                color: ColorArg::Never,
                interactive: false,
                json: None,
                update_all: false,
                inspect: Default::default(),
            },
            context: ContextArgs {
                before: 0,
                after: 0,
                context: 0,
            },
        }
    }

    #[test]
    fn test_run_with_pattern() {
        let arg = RunArg {
            pattern: "console.log".to_string(),
            ..default_run_arg()
        };
        let proj = Err(anyhow::anyhow!("no project"));
        assert!(run_with_pattern(arg, proj).is_ok())
    }

    #[test]
    fn test_run_with_strictness() {
        let arg = RunArg {
            pattern: "console.log".to_string(),
            strictness: Some(Strictness(MatchStrictness::Ast)),
            ..default_run_arg()
        };
        let proj = Err(anyhow::anyhow!("no project"));
        assert!(run_with_pattern(arg, proj).is_ok())
    }

    #[test]
    fn test_run_with_specific_lang() {
        let arg = RunArg {
            pattern: "Some(result)".to_string(),
            lang: Some(SupportedLanguage::Rust.into()),
            ..default_run_arg()
        };
        let proj = Err(anyhow::anyhow!("no project"));
        assert!(run_with_pattern(arg, proj).is_ok())
    }
}
