mod json_render;
mod support;
mod text_render;

use crate::{NodeBootstrapReport, OutputMode, OutputModeKind};
use json_render::render_json_report;
use text_render::render_text_report;

pub(crate) fn render_bootstrap_report(report: &NodeBootstrapReport, mode: OutputMode) -> String {
    match mode.kind {
        OutputModeKind::Text => render_text_report(report),
        OutputModeKind::Json => render_json_report(report),
    }
}
