//! CLI argument parsing for the desktop binary.
//!
//! The `--serve` flag (`WebUI` headless mode) is handled inline in `lib.rs` for
//! historical reasons. This module parses the session-preload CLI surface:
//!
//! ```text
//! --session <uuid|abs-path>     # UUID regex → Uuid; abs path → Path; else None
//! --session-folder <name>       # Exact sesslog folder name under ~/.claude/projects/
//! --session-title <text>        # Substring match, case-insensitive, resolved client-side
//! ```
//!
//! The resolved hint is exposed to the frontend via the `get_startup_session_hint`
//! Tauri command; the React side then navigates to the target session (or opens
//! a picker modal for ambiguous title matches) once projects are loaded.
//!
//! Stage B precedence: `--session` > `--session-folder` > `--session-title` if
//! more than one is passed.

use crate::cli_args::extract_flag_value;
use serde::Serialize;
use tauri::State;

/// Newtype wrapper so we can pass `Option<SessionHint>` through Tauri's typed
/// managed-state API. `tauri::State<T>` keys by type, so wrapping in a named
/// struct avoids any accidental collision with a future `Option<T>` managed by
/// another subsystem.
#[derive(Default)]
pub struct StartupSessionHint(pub Option<SessionHint>);

/// Tauri command returning the CLI-supplied session hint, if any.
///
/// The frontend calls this on mount after projects have loaded; `None` means
/// "no preload requested, run the normal UI".
#[tauri::command]
#[must_use]
pub fn get_startup_session_hint(state: State<'_, StartupSessionHint>) -> Option<SessionHint> {
    state.0.clone()
}

/// A CLI-supplied hint asking the frontend to preload a specific session.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionHint {
    /// Resolution strategy: how the frontend should interpret `value`.
    pub kind: SessionHintKind,
    /// The raw value as supplied on the command line.
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionHintKind {
    /// Full UUID or UUID prefix (hex + dash, 8..=36 chars).
    Uuid,
    /// Absolute filesystem path to a session `.jsonl` file.
    Path,
    /// Exact sesslog folder name (the directory under `~/.claude/projects/`).
    Folder,
    /// Free-text, substring matched case-insensitively against session titles.
    Title,
}

/// Parse the session-preload CLI surface from a raw argv vec.
///
/// Precedence: `--session` > `--session-folder` > `--session-title`. Returns
/// `None` if no recognized flag is present or the value cannot be classified.
#[must_use]
pub fn parse_session_hint(args: &[String]) -> Option<SessionHint> {
    if let Some(raw) = extract_flag_value(args, "--session") {
        return classify_session_value(raw);
    }
    if let Some(raw) = extract_flag_value(args, "--session-folder") {
        if raw.is_empty() {
            return None;
        }
        return Some(SessionHint {
            kind: SessionHintKind::Folder,
            value: raw,
        });
    }
    if let Some(raw) = extract_flag_value(args, "--session-title") {
        if raw.is_empty() {
            return None;
        }
        return Some(SessionHint {
            kind: SessionHintKind::Title,
            value: raw,
        });
    }
    None
}

/// Classify a `--session <value>` argument as either a UUID hint or a Path hint.
/// Values that look like neither are rejected (the dedicated `--session-folder` /
/// `--session-title` flags exist for disambiguation).
fn classify_session_value(value: String) -> Option<SessionHint> {
    if is_uuid_like(&value) {
        return Some(SessionHint {
            kind: SessionHintKind::Uuid,
            value,
        });
    }
    if looks_like_abs_path(&value) {
        return Some(SessionHint {
            kind: SessionHintKind::Path,
            value,
        });
    }
    None
}

/// A UUID is 36 chars with four dashes; a prefix is any 8-35 char slice of
/// the canonical form. We accept anything hex-or-dash of length 8..=36.
fn is_uuid_like(value: &str) -> bool {
    let len = value.len();
    if !(8..=36).contains(&len) {
        return false;
    }
    value.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// Heuristic absolute-path detection that works on both Unix and Windows.
/// Unix: starts with `/`. Windows: drive-letter prefix like `C:\` or `C:/` or UNC `\\`.
fn looks_like_abs_path(value: &str) -> bool {
    if value.starts_with('/') {
        return true;
    }
    if value.starts_with("\\\\") {
        return true; // UNC path
    }
    let bytes = value.as_bytes();
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
    {
        return true; // Windows drive-letter path
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    // ===== Stage A (UUID) tests — preserved from PR #261 =====

    #[test]
    fn returns_none_when_no_flag_present() {
        let args = argv(&["app", "--serve"]);
        assert!(parse_session_hint(&args).is_none());
    }

    #[test]
    fn parses_space_separated_uuid() {
        let args = argv(&["app", "--session", "1265cd74-caa9-472e-b343-c4f44b5cf12c"]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.kind, SessionHintKind::Uuid);
        assert_eq!(hint.value, "1265cd74-caa9-472e-b343-c4f44b5cf12c");
    }

    #[test]
    fn parses_equals_form() {
        let args = argv(&["app", "--session=1265cd74-caa9-472e-b343-c4f44b5cf12c"]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.value, "1265cd74-caa9-472e-b343-c4f44b5cf12c");
    }

    #[test]
    fn accepts_uuid_prefix() {
        let args = argv(&["app", "--session", "1265cd74"]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.value, "1265cd74");
    }

    #[test]
    fn rejects_non_hex_value_that_is_not_a_path() {
        let args = argv(&["app", "--session", "hello-world-not-a-uuid"]);
        // Contains non-hex chars and no leading `/` or drive letter — reject.
        assert!(parse_session_hint(&args).is_none());
    }

    #[test]
    fn rejects_too_short_value() {
        let args = argv(&["app", "--session", "1265cd7"]);
        assert!(parse_session_hint(&args).is_none());
    }

    #[test]
    fn rejects_too_long_non_path_value() {
        let args = argv(&[
            "app",
            "--session",
            "1265cd74-caa9-472e-b343-c4f44b5cf12c-extra",
        ]);
        assert!(parse_session_hint(&args).is_none());
    }

    #[test]
    fn returns_none_when_flag_value_is_another_flag() {
        let args = argv(&["app", "--session", "--serve"]);
        assert!(parse_session_hint(&args).is_none());
    }

    #[test]
    fn returns_none_when_flag_has_no_following_argument() {
        let args = argv(&["app", "--session"]);
        assert!(parse_session_hint(&args).is_none());
    }

    #[test]
    fn returns_none_when_equals_form_has_empty_value() {
        let args = argv(&["app", "--session="]);
        assert!(parse_session_hint(&args).is_none());
    }

    // ===== Stage B Path tests =====

    #[test]
    fn parses_abs_unix_path_as_path_kind() {
        let args = argv(&[
            "app",
            "--session",
            "/Users/jack/.claude/projects/demo/abc123.jsonl",
        ]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.kind, SessionHintKind::Path);
        assert_eq!(hint.value, "/Users/jack/.claude/projects/demo/abc123.jsonl");
    }

    #[test]
    fn parses_abs_windows_backslash_path_as_path_kind() {
        let args = argv(&[
            "app",
            "--session",
            "C:\\Users\\jack\\.claude\\projects\\demo\\abc.jsonl",
        ]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.kind, SessionHintKind::Path);
    }

    #[test]
    fn parses_abs_windows_forwardslash_path_as_path_kind() {
        let args = argv(&["app", "--session", "C:/Users/jack/session.jsonl"]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.kind, SessionHintKind::Path);
    }

    #[test]
    fn parses_unc_path_as_path_kind() {
        let args = argv(&["app", "--session", "\\\\server\\share\\session.jsonl"]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.kind, SessionHintKind::Path);
    }

    #[test]
    fn rejects_relative_path_as_session_value() {
        // A relative path like "demo" or "demo/session.jsonl" is not accepted
        // under --session — use --session-folder or a full path instead.
        let args = argv(&["app", "--session", "demo/session.jsonl"]);
        assert!(parse_session_hint(&args).is_none());
    }

    // ===== Stage B Folder tests =====

    #[test]
    fn parses_session_folder_flag() {
        let args = argv(&["app", "--session-folder", "demo"]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.kind, SessionHintKind::Folder);
        assert_eq!(hint.value, "demo");
    }

    #[test]
    fn parses_session_folder_equals_form() {
        let args = argv(&["app", "--session-folder=demo"]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.kind, SessionHintKind::Folder);
    }

    #[test]
    fn rejects_empty_session_folder() {
        let args = argv(&["app", "--session-folder="]);
        assert!(parse_session_hint(&args).is_none());
    }

    // ===== Stage B Title tests =====

    #[test]
    fn parses_session_title_flag() {
        let args = argv(&["app", "--session-title", "auth bug"]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.kind, SessionHintKind::Title);
        assert_eq!(hint.value, "auth bug");
    }

    #[test]
    fn parses_session_title_equals_form() {
        let args = argv(&["app", "--session-title=refactoring"]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.kind, SessionHintKind::Title);
    }

    #[test]
    fn rejects_empty_session_title() {
        let args = argv(&["app", "--session-title="]);
        assert!(parse_session_hint(&args).is_none());
    }

    // ===== Stage B Precedence tests =====

    #[test]
    fn session_wins_over_session_folder() {
        let args = argv(&[
            "app",
            "--session",
            "1265cd74-caa9-472e-b343-c4f44b5cf12c",
            "--session-folder",
            "demo",
        ]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.kind, SessionHintKind::Uuid);
    }

    #[test]
    fn session_folder_wins_over_session_title() {
        let args = argv(&["app", "--session-folder", "demo", "--session-title", "auth"]);
        let hint = parse_session_hint(&args).expect("hint");
        assert_eq!(hint.kind, SessionHintKind::Folder);
    }

    #[test]
    fn malformed_session_does_not_fall_through_to_folder() {
        // When --session has a malformed value we reject it outright rather
        // than walking to the next flag (which would silently change the
        // user's intent). The return is a silent None; the frontend then
        // treats the launch as if no --session was passed.
        let args = argv(&[
            "app",
            "--session",
            "not-a-uuid-and-not-a-path",
            "--session-folder",
            "demo",
        ]);
        assert!(parse_session_hint(&args).is_none());
    }
}
