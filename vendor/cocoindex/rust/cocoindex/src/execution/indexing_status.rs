use crate::prelude::*;

use utils::fingerprint::{Fingerprint, Fingerprinter};

pub struct SourceLogicFingerprint {
    pub current: Fingerprint,
    pub legacy: Vec<Fingerprint>,
}

impl SourceLogicFingerprint {
    pub fn new(
        exec_plan: &plan::ExecutionPlan,
        source_idx: usize,
        export_exec_ctx: &[exec_ctx::ExportOpExecutionContext],
        legacy: Vec<Fingerprint>,
    ) -> Result<Self> {
        let import_op = &exec_plan.import_ops[source_idx];
        let mut fp = Fingerprinter::default();
        if exec_plan.export_ops.len() != export_exec_ctx.len() {
            internal_bail!("`export_ops` count does not match `export_exec_ctx` count");
        }
        for (export_op, export_op_exec_ctx) in
            std::iter::zip(exec_plan.export_ops.iter(), export_exec_ctx.iter())
        {
            if export_op.def_fp.source_op_names.contains(&import_op.name) {
                fp = fp.with(&(
                    &export_op.def_fp.fingerprint,
                    &export_op_exec_ctx.target_id,
                    &export_op_exec_ctx.schema_version_id,
                ))?;
            }
        }
        Ok(Self {
            current: fp.into_fingerprint(),
            legacy,
        })
    }

    pub fn matches(&self, other: impl AsRef<[u8]>) -> bool {
        self.current.as_slice() == other.as_ref()
            || self.legacy.iter().any(|fp| fp.as_slice() == other.as_ref())
    }
}
