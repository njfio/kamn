mod filesystem;
mod preflight;
mod probing;

pub(crate) use preflight::ensure_external_execution_preflight;
pub(crate) use probing::probe_external_runtime;
#[cfg(test)]
pub(crate) use probing::{
    probe_binary_invocation_with_status_runner, probe_command_args_for_label,
    should_retry_text_file_busy, ETXTBSY_ERRNO, TEXT_FILE_BUSY_RETRY_LIMIT,
};
