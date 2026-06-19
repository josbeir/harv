use std::path::PathBuf;

/// Top-level error type for the harv ecosystem.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HarvError {
    #[error("Authentication required. Run `harv connect` to log in.")]
    NotAuthenticated,

    #[error("Config not found at {0}. Run `harv connect` to log in.")]
    ConfigNotFound(PathBuf),

    #[error("Config file is malformed: {0}")]
    ConfigMalformed(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid date: {0}")]
    InvalidDate(String),

    #[error("Invalid time: {0}")]
    InvalidTime(String),

    #[error("No running timer found.")]
    NoRunningTimer,

    #[error("No project assignments found.")]
    NoProjectAssignments,

    #[error("No task assignments found for project {project_id}.")]
    NoTaskAssignments { project_id: u64 },

    #[error("Alias '{0}' not found.")]
    AliasNotFound(String),

    #[error("Supplied access token and account id could not be retrieved from OAuth2 response.")]
    OAuthFailed,

    #[error("OAuth2 authorization was denied.")]
    OAuthDenied,

    #[error("{0}")]
    Other(String),
}

impl HarvError {
    /// User-friendly message suitable for display in the CLI.
    pub fn user_message(&self) -> String {
        match self {
            Self::NotAuthenticated => crate::locale::t("err-not-authenticated"),
            Self::ConfigNotFound(_) => crate::locale::t("err-config-not-found"),
            Self::Api { status, message } => crate::locale::t_args(
                "err-api",
                &[("status", status.to_string()), ("message", message.clone())],
            ),
            Self::NoRunningTimer => crate::locale::t("err-no-running-timer"),
            Self::NoProjectAssignments => crate::locale::t("err-no-project-assignments"),
            Self::NoTaskAssignments { project_id } => crate::locale::t_args(
                "err-no-task-assignments",
                &[("project_id", project_id.to_string())],
            ),
            Self::AliasNotFound(name) => {
                crate::locale::t_args("err-alias-not-found", &[("name", name.clone())])
            }
            Self::OAuthDenied => crate::locale::t("err-oauth-denied"),
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::path::PathBuf;

    fn ensure_locale() {
        crate::locale::init(None);
    }

    #[serial]
    #[test]
    fn test_not_authenticated_display() {
        ensure_locale();
        let err = HarvError::NotAuthenticated;
        assert!(err.to_string().contains("harv connect"));
    }

    #[serial]
    #[test]
    fn test_config_not_found_display() {
        ensure_locale();
        let path = PathBuf::from("/home/user/.config/harv/config.toml");
        let err = HarvError::ConfigNotFound(path.clone());
        let msg = err.to_string();
        assert!(msg.contains(&*path.to_string_lossy()));
        assert!(msg.contains("harv connect"));
    }

    #[serial]
    #[test]
    fn test_config_not_found_user_message() {
        ensure_locale();
        let err = HarvError::ConfigNotFound(PathBuf::from("/tmp/config.toml"));
        assert!(err.user_message().contains("harv connect"));
    }

    #[serial]
    #[test]
    fn test_api_error_display() {
        ensure_locale();
        let err = HarvError::Api {
            status: 422,
            message: "Validation failed".into(),
        };
        assert_eq!(err.to_string(), "API error (422): Validation failed");
    }

    #[serial]
    #[test]
    fn test_api_error_user_message() {
        ensure_locale();
        let err = HarvError::Api {
            status: 404,
            message: "Not found".into(),
        };
        assert!(err.user_message().contains("404"));
        assert!(err.user_message().contains("Not found"));
    }

    #[serial]
    #[test]
    fn test_no_running_timer_display() {
        ensure_locale();
        let err = HarvError::NoRunningTimer;
        assert_eq!(err.to_string(), "No running timer found.");
    }

    #[serial]
    #[test]
    fn test_no_running_timer_user_message() {
        ensure_locale();
        let err = HarvError::NoRunningTimer;
        assert_eq!(err.user_message(), "No timer is currently running.");
    }

    #[serial]
    #[test]
    fn test_alias_not_found_user_message() {
        ensure_locale();
        let err = HarvError::AliasNotFound("myalias".into());
        assert!(err.user_message().contains("myalias"));
        assert!(err.user_message().contains("harv alias list"));
    }

    #[serial]
    #[test]
    fn test_io_error_from() {
        ensure_locale();
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let harv_err: HarvError = io_err.into();
        let msg = harv_err.to_string();
        assert!(msg.contains("IO error"));
        assert!(msg.contains("file not found"));
    }

    #[serial]
    #[test]
    fn test_other_error() {
        ensure_locale();
        let err = HarvError::Other("something went wrong".into());
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[serial]
    #[test]
    fn test_config_malformed() {
        ensure_locale();
        let err = HarvError::ConfigMalformed("bad json".into());
        assert!(err.to_string().contains("malformed"));
        assert!(err.to_string().contains("bad json"));
    }

    #[serial]
    #[test]
    fn test_http_error() {
        ensure_locale();
        let err = HarvError::Http("connection refused".into());
        assert!(err.to_string().contains("HTTP error"));
        assert!(err.to_string().contains("connection refused"));
    }

    #[serial]
    #[test]
    fn test_invalid_date() {
        ensure_locale();
        let err = HarvError::InvalidDate("2026-13-01".into());
        assert!(err.to_string().contains("Invalid date"));
    }

    #[serial]
    #[test]
    fn test_invalid_time() {
        ensure_locale();
        let err = HarvError::InvalidTime("25:00".into());
        assert!(err.to_string().contains("Invalid time"));
    }

    #[serial]
    #[test]
    fn test_no_project_assignments_display() {
        ensure_locale();
        let err = HarvError::NoProjectAssignments;
        assert!(err.to_string().contains("No project assignments"));
    }

    #[serial]
    #[test]
    fn test_no_project_assignments_user_message() {
        ensure_locale();
        let err = HarvError::NoProjectAssignments;
        assert_eq!(err.user_message(), "You have no project assignments.");
    }

    #[serial]
    #[test]
    fn test_no_task_assignments_display() {
        ensure_locale();
        let err = HarvError::NoTaskAssignments { project_id: 42 };
        assert!(err.to_string().contains("project 42"));
    }

    #[serial]
    #[test]
    fn test_no_task_assignments_user_message() {
        ensure_locale();
        let err = HarvError::NoTaskAssignments { project_id: 42 };
        assert!(err.user_message().contains("42"));
    }

    #[serial]
    #[test]
    fn test_oauth_failed_display() {
        ensure_locale();
        let err = HarvError::OAuthFailed;
        assert!(err.to_string().contains("access token"));
    }

    #[serial]
    #[test]
    fn test_oauth_denied_display() {
        ensure_locale();
        let err = HarvError::OAuthDenied;
        assert!(err.to_string().contains("denied"));
    }

    #[serial]
    #[test]
    fn test_oauth_denied_user_message() {
        ensure_locale();
        let err = HarvError::OAuthDenied;
        assert!(err.user_message().contains("harv connect"));
    }

    #[serial]
    #[test]
    fn test_oauth_failed_user_message() {
        ensure_locale();
        let err = HarvError::OAuthFailed;
        assert!(err.user_message().contains("access token"));
    }

    #[serial]
    #[test]
    fn test_config_malformed_user_message() {
        ensure_locale();
        let err = HarvError::ConfigMalformed("bad toml".into());
        assert!(err.user_message().contains("malformed"));
        assert!(err.user_message().contains("bad toml"));
    }
}
