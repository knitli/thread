use thread_ast_grep::{ProjectConfig, RuleCollection, RuleConfig};
use super::threadlang::ThreadLang;

use anyhow::Result;

#[cfg(feature = "cli_support")]
use clap::ValueEnum;

use std::fmt;
use std::io::{Stderr, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Granularity of tracing information.
#[derive(Clone, Copy, #[cfg(feature = "cli_support")] ValueEnum, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Granularity {
  /// Do not show any tracing information
  #[default]
  Nothing = 0,
  /// Show summary about how many files are scanned and skipped
  Summary = 1,
  /// Show per-file/per-rule tracing information
  Entity = 2,
  // Detail,
}

impl fmt::Debug for Granularity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Granularity::Nothing => write!(f, "nothing"),
      Granularity::Summary => write!(f, "summary"),
      Granularity::Entity => write!(f, "entity"),
    }
  }
}

impl Granularity {
  pub fn project_trace(&self) -> ProjectTrace {
    self.project_trace_impl(std::io::stderr())
  }
  fn project_trace_impl<W: Write>(&self, w: W) -> TraceInfo<(), W> {
    TraceInfo {
      level: *self,
      inner: (),
      output: Mutex::new(w),
    }
  }

  /// Create a run trace with the given granularity.
  pub fn run_trace(&self) -> RunTrace {
    self.run_trace_impl(std::io::stderr())
  }
  fn run_trace_impl<W: Write>(&self, w: W) -> TraceInfo<FileTrace, W> {
    TraceInfo {
      level: *self,
      inner: Default::default(),
      output: Mutex::new(w),
    }
  }

  /// Create a scan trace with the given rule statistics.
  pub fn scan_trace(&self, rule_stats: RuleTrace) -> ScanTrace {
    self.scan_trace_impl(rule_stats, std::io::stderr())
  }
  fn scan_trace_impl<W: Write>(&self, rule_stats: RuleTrace, w: W) -> TraceInfo<RuleTrace, W> {
    TraceInfo {
      level: *self,
      inner: rule_stats,
      output: Mutex::new(w),
    }
  }
}

// total = scanned + skipped
//       = (matched + unmatched) + skipped
#[derive(Default)]
pub struct FileTrace {
  files_scanned: AtomicUsize,
  files_skipped: AtomicUsize,
}

impl FileTrace {
  /// Add a file that is scanned.
  pub fn add_scanned(&self) {
    self.files_scanned.fetch_add(1, Ordering::AcqRel);
  }
  pub fn add_skipped(&self) {
    /// Add a file that is skipped.
    self.files_skipped.fetch_add(1, Ordering::AcqRel);
  }
}

pub struct ThreadTraceInfo<T> {
    pub level: Granularity,
    pub inner: T,
    pub output: Box<dyn TraceOutput>,
}

impl<T> ThreadTraceInfo<T> {
    async fn emit_trace(&mut self, event: TraceEvent) -> Result<()> {
        if self.level >= event.minimum_level() {
            self.output.call(event).await?;
        }
        Ok(())
    }
}

impl ThreadTraceInfo<RuleTrace> {
    pub async fn print(&mut self) -> Result<()> {
        self.emit_trace(TraceEvent::FileStats {
            scanned: self.inner.file_trace.files_scanned.load(Ordering::Acquire),
            skipped: self.inner.file_trace.files_skipped.load(Ordering::Acquire),
        }).await?;

        self.emit_trace(TraceEvent::RuleStats {
            effective: self.inner.effective_rule_count,
            skipped: self.inner.skipped_rule_count,
        }).await?;

        Ok(())
    }

    pub async fn print_file(
        &mut self,
        path: &Path,
        lang: ThreadLang,
        rules: &[&RuleConfig<ThreadLang>]
    ) -> Result<()> {
        self.emit_trace(TraceEvent::FileScan {
            path: path.to_path_buf(),
            lang,
        }).await
    }
}
