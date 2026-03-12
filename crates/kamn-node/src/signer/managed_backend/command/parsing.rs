use kamn_core::ConfigError;

use super::super::ManagedSignerCommandSpec;
use crate::KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV;

fn push_current_arg(argv: &mut Vec<String>, current: &mut String) {
    if !current.is_empty() {
        argv.push(std::mem::take(current));
    }
}

fn parse_escaped_character(character: char, current: &mut String, escaping: &mut bool) {
    current.push(character);
    *escaping = false;
}

fn parse_single_quoted_character(
    character: char,
    current: &mut String,
    in_single_quotes: &mut bool,
) {
    if character == '\'' {
        *in_single_quotes = false;
    } else {
        current.push(character);
    }
}

fn parse_double_quoted_character(
    character: char,
    current: &mut String,
    in_double_quotes: &mut bool,
    escaping: &mut bool,
) {
    match character {
        '"' => *in_double_quotes = false,
        '\\' => *escaping = true,
        _ => current.push(character),
    }
}

fn parse_unquoted_character(
    character: char,
    argv: &mut Vec<String>,
    current: &mut String,
    in_single_quotes: &mut bool,
    in_double_quotes: &mut bool,
    escaping: &mut bool,
) {
    match character {
        '\'' => *in_single_quotes = true,
        '"' => *in_double_quotes = true,
        '\\' => *escaping = true,
        character if character.is_whitespace() => push_current_arg(argv, current),
        _ => current.push(character),
    }
}

fn ensure_command_parse_completed(
    argv: &mut Vec<String>,
    current: String,
    escaping: bool,
    in_single_quotes: bool,
    in_double_quotes: bool,
) -> Result<(), ConfigError> {
    if escaping || in_single_quotes || in_double_quotes {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} contains unterminated quoting or escaping (managed_signer_backend_unavailable)"
        )));
    }
    if !current.is_empty() {
        argv.push(current);
    }
    if argv.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} must contain at least one argv token (managed_signer_backend_unavailable)"
        )));
    }
    Ok(())
}

pub(crate) fn parse_kolme_live_managed_signer_command_spec(
    command: &str,
) -> Result<ManagedSignerCommandSpec, ConfigError> {
    let mut argv = Vec::new();
    let mut current = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut escaping = false;

    for character in command.chars() {
        if escaping {
            parse_escaped_character(character, &mut current, &mut escaping);
            continue;
        }
        if in_single_quotes {
            parse_single_quoted_character(character, &mut current, &mut in_single_quotes);
            continue;
        }
        if in_double_quotes {
            parse_double_quoted_character(
                character,
                &mut current,
                &mut in_double_quotes,
                &mut escaping,
            );
            continue;
        }
        parse_unquoted_character(
            character,
            &mut argv,
            &mut current,
            &mut in_single_quotes,
            &mut in_double_quotes,
            &mut escaping,
        );
    }

    ensure_command_parse_completed(
        &mut argv,
        current,
        escaping,
        in_single_quotes,
        in_double_quotes,
    )?;
    let executable = argv.remove(0);
    Ok(ManagedSignerCommandSpec {
        executable,
        args: argv,
    })
}
