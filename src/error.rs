use std::time::Duration;

/// Alias for `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// All error variants returned by the Threads API client.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An authentication or authorization failure (e.g. invalid/expired token).
    #[error("authentication error {code} ({error_type}): {message}")]
    Authentication {
        /// API error code.
        code: u16,
        /// Human-readable error message.
        message: String,
        /// Machine-readable error category.
        error_type: String,
        /// Additional detail string from the API.
        details: String,
        /// Whether the API flagged this error as transient.
        is_transient: bool,
        /// HTTP status code of the response.
        http_status_code: u16,
        /// API error sub-code for finer classification.
        error_subcode: u16,
    },

    /// The request was throttled by the API rate limiter.
    #[error("rate limit error {code}: {message}")]
    RateLimit {
        /// API error code.
        code: u16,
        /// Human-readable error message.
        message: String,
        /// Machine-readable error category.
        error_type: String,
        /// Additional detail string from the API.
        details: String,
        /// Suggested wait time before retrying.
        retry_after: Duration,
        /// Whether the API flagged this error as transient.
        is_transient: bool,
        /// HTTP status code of the response.
        http_status_code: u16,
        /// API error sub-code for finer classification.
        error_subcode: u16,
    },

    /// A request parameter or payload failed validation.
    #[error("validation error {code}: {message}")]
    Validation {
        /// API error code.
        code: u16,
        /// Human-readable error message.
        message: String,
        /// Machine-readable error category.
        error_type: String,
        /// Additional detail string from the API.
        details: String,
        /// Name of the invalid field.
        field: String,
        /// Whether the API flagged this error as transient.
        is_transient: bool,
        /// HTTP status code of the response.
        http_status_code: u16,
        /// API error sub-code for finer classification.
        error_subcode: u16,
    },

    /// A transport-level failure (DNS, TCP, TLS, timeout, etc.).
    #[error("network error: {message}")]
    Network {
        /// API error code.
        code: u16,
        /// Human-readable error message.
        message: String,
        /// Machine-readable error category.
        error_type: String,
        /// Additional detail string from the API.
        details: String,
        /// Whether the failure appears to be temporary.
        temporary: bool,
        /// Whether the API flagged this error as transient.
        is_transient: bool,
        /// HTTP status code of the response.
        http_status_code: u16,
        /// API error sub-code for finer classification.
        error_subcode: u16,
        /// Optional underlying reqwest error.
        #[source]
        cause: Option<reqwest::Error>,
    },

    /// A generic API error that does not fit a more specific variant.
    #[error("API error {code}: {message}")]
    Api {
        /// API error code.
        code: u16,
        /// Human-readable error message.
        message: String,
        /// Machine-readable error category.
        error_type: String,
        /// Additional detail string from the API.
        details: String,
        /// Server-assigned request identifier for support/debugging.
        request_id: String,
        /// Whether the API flagged this error as transient.
        is_transient: bool,
        /// HTTP status code of the response.
        http_status_code: u16,
        /// API error sub-code for finer classification.
        error_subcode: u16,
    },

    /// An HTTP-level error from the underlying reqwest client.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// A JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

/// Create a new authentication error.
pub fn new_authentication_error(code: u16, message: &str, details: &str) -> Error {
    Error::Authentication {
        code,
        message: message.to_owned(),
        error_type: "authentication_error".to_owned(),
        details: details.to_owned(),
        is_transient: false,
        http_status_code: 0,
        error_subcode: 0,
    }
}

/// Create a new rate-limit error.
pub fn new_rate_limit_error(
    code: u16,
    message: &str,
    details: &str,
    retry_after: Duration,
) -> Error {
    Error::RateLimit {
        code,
        message: message.to_owned(),
        error_type: "rate_limit_error".to_owned(),
        details: details.to_owned(),
        retry_after,
        is_transient: false,
        http_status_code: 0,
        error_subcode: 0,
    }
}

/// Create a new validation error.
pub fn new_validation_error(code: u16, message: &str, details: &str, field: &str) -> Error {
    Error::Validation {
        code,
        message: message.to_owned(),
        error_type: "validation_error".to_owned(),
        details: details.to_owned(),
        field: field.to_owned(),
        is_transient: false,
        http_status_code: 0,
        error_subcode: 0,
    }
}

/// Create a new network error without an underlying cause.
pub fn new_network_error(code: u16, message: &str, details: &str, temporary: bool) -> Error {
    new_network_error_with_cause(code, message, details, temporary, None)
}

/// Create a new network error wrapping an underlying reqwest cause.
pub fn new_network_error_with_cause(
    code: u16,
    message: &str,
    details: &str,
    temporary: bool,
    cause: Option<reqwest::Error>,
) -> Error {
    Error::Network {
        code,
        message: message.to_owned(),
        error_type: "network_error".to_owned(),
        details: details.to_owned(),
        temporary,
        is_transient: false,
        http_status_code: 0,
        error_subcode: 0,
        cause,
    }
}

/// Create a new generic API error.
pub fn new_api_error(code: u16, message: &str, details: &str, request_id: &str) -> Error {
    Error::Api {
        code,
        message: message.to_owned(),
        error_type: "api_error".to_owned(),
        details: details.to_owned(),
        request_id: request_id.to_owned(),
        is_transient: false,
        http_status_code: 0,
        error_subcode: 0,
    }
}

// ---------------------------------------------------------------------------
// Metadata helpers
// ---------------------------------------------------------------------------

/// Base fields extracted from any typed error variant.
pub struct BaseFields<'a> {
    /// API error code.
    pub code: u16,
    /// Human-readable error message.
    pub message: &'a str,
    /// Machine-readable error category.
    pub error_type: &'a str,
    /// Additional detail string from the API.
    pub details: &'a str,
    /// Whether the API flagged this error as transient.
    pub is_transient: bool,
    /// HTTP status code of the response.
    pub http_status_code: u16,
    /// API error sub-code for finer classification.
    pub error_subcode: u16,
}

/// Extract common base fields from a typed error variant.
/// Returns `None` for `Http` and `Json` variants.
pub fn extract_base_fields(err: &Error) -> Option<BaseFields<'_>> {
    match err {
        Error::Authentication {
            code,
            message,
            error_type,
            details,
            is_transient,
            http_status_code,
            error_subcode,
        } => Some(BaseFields {
            code: *code,
            message,
            error_type,
            details,
            is_transient: *is_transient,
            http_status_code: *http_status_code,
            error_subcode: *error_subcode,
        }),
        Error::RateLimit {
            code,
            message,
            error_type,
            details,
            is_transient,
            http_status_code,
            error_subcode,
            ..
        } => Some(BaseFields {
            code: *code,
            message,
            error_type,
            details,
            is_transient: *is_transient,
            http_status_code: *http_status_code,
            error_subcode: *error_subcode,
        }),
        Error::Validation {
            code,
            message,
            error_type,
            details,
            is_transient,
            http_status_code,
            error_subcode,
            ..
        } => Some(BaseFields {
            code: *code,
            message,
            error_type,
            details,
            is_transient: *is_transient,
            http_status_code: *http_status_code,
            error_subcode: *error_subcode,
        }),
        Error::Network {
            code,
            message,
            error_type,
            details,
            is_transient,
            http_status_code,
            error_subcode,
            ..
        } => Some(BaseFields {
            code: *code,
            message,
            error_type,
            details,
            is_transient: *is_transient,
            http_status_code: *http_status_code,
            error_subcode: *error_subcode,
        }),
        Error::Api {
            code,
            message,
            error_type,
            details,
            is_transient,
            http_status_code,
            error_subcode,
            ..
        } => Some(BaseFields {
            code: *code,
            message,
            error_type,
            details,
            is_transient: *is_transient,
            http_status_code: *http_status_code,
            error_subcode: *error_subcode,
        }),
        Error::Http(_) | Error::Json(_) => None,
    }
}

/// Set transient flag, HTTP status code, and error subcode on a typed error.
pub fn set_error_metadata(
    err: &mut Error,
    is_transient: bool,
    http_status_code: u16,
    error_subcode: u16,
) {
    match err {
        Error::Authentication {
            is_transient: t,
            http_status_code: h,
            error_subcode: s,
            ..
        }
        | Error::RateLimit {
            is_transient: t,
            http_status_code: h,
            error_subcode: s,
            ..
        }
        | Error::Validation {
            is_transient: t,
            http_status_code: h,
            error_subcode: s,
            ..
        }
        | Error::Network {
            is_transient: t,
            http_status_code: h,
            error_subcode: s,
            ..
        }
        | Error::Api {
            is_transient: t,
            http_status_code: h,
            error_subcode: s,
            ..
        } => {
            *t = is_transient;
            *h = http_status_code;
            *s = error_subcode;
        }
        Error::Http(_) | Error::Json(_) => {}
    }
}

// ---------------------------------------------------------------------------
// Type-checking helpers
// ---------------------------------------------------------------------------

impl Error {
    /// Returns `true` if this is an authentication error.
    pub fn is_authentication(&self) -> bool {
        matches!(self, Error::Authentication { .. })
    }

    /// Returns `true` if this is a rate-limit error.
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, Error::RateLimit { .. })
    }

    /// Returns `true` if this is a validation error.
    pub fn is_validation(&self) -> bool {
        matches!(self, Error::Validation { .. })
    }

    /// Returns `true` if this is a network error.
    pub fn is_network(&self) -> bool {
        matches!(self, Error::Network { .. })
    }

    /// Returns `true` if this is a generic API error.
    pub fn is_api(&self) -> bool {
        matches!(self, Error::Api { .. })
    }

    /// Returns `true` if the API flagged this error as transient.
    pub fn is_transient(&self) -> bool {
        match self {
            Error::Authentication { is_transient, .. }
            | Error::RateLimit { is_transient, .. }
            | Error::Validation { is_transient, .. }
            | Error::Network { is_transient, .. }
            | Error::Api { is_transient, .. } => *is_transient,
            _ => false,
        }
    }

    /// Returns `true` if the request can be retried (rate-limit, transient, or temporary network).
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::RateLimit { .. } => true,
            Error::Network {
                temporary,
                is_transient,
                ..
            } => *temporary || *is_transient,
            _ => self.is_transient(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_authentication_error() {
        let err = new_authentication_error(401, "Invalid token", "Token expired");
        assert!(err.is_authentication());
        assert!(!err.is_rate_limit());
        assert!(!err.is_transient());
        assert!(!err.is_retryable());
        assert!(err.to_string().contains("401"));
        assert!(err.to_string().contains("Invalid token"));
    }

    #[test]
    fn test_new_rate_limit_error() {
        let err = new_rate_limit_error(429, "Too many requests", "", Duration::from_secs(60));
        assert!(err.is_rate_limit());
        assert!(!err.is_authentication());
        assert!(err.is_retryable());
    }

    #[test]
    fn test_new_validation_error() {
        let err = new_validation_error(400, "Bad input", "text too long", "text");
        assert!(err.is_validation());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_new_network_error() {
        let err = new_network_error(0, "Connection refused", "", true);
        assert!(err.is_network());
        assert!(err.is_retryable());
    }

    #[test]
    fn test_new_network_error_not_temporary() {
        let err = new_network_error(0, "DNS failure", "", false);
        assert!(err.is_network());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_network_error_transient_is_retryable() {
        let mut err = new_network_error(0, "Transient failure", "", false);
        assert!(!err.is_retryable());
        set_error_metadata(&mut err, true, 503, 0);
        assert!(err.is_transient());
        assert!(err.is_retryable());
    }

    #[test]
    fn test_new_api_error() {
        let err = new_api_error(500, "Internal error", "", "req-123");
        assert!(err.is_api());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_set_error_metadata() {
        let mut err = new_api_error(500, "Internal error", "", "");
        assert!(!err.is_transient());
        set_error_metadata(&mut err, true, 503, 42);
        assert!(err.is_transient());
        assert!(err.is_retryable());
        let base = extract_base_fields(&err).unwrap();
        assert_eq!(base.http_status_code, 503);
        assert_eq!(base.error_subcode, 42);
    }

    #[test]
    fn test_extract_base_fields_http() {
        // Http and Json variants should return None
        let err = Error::Json(serde_json::from_str::<String>("invalid").unwrap_err());
        assert!(extract_base_fields(&err).is_none());
    }

    #[test]
    fn test_is_helpers_exhaustive() {
        let auth = new_authentication_error(401, "x", "");
        assert!(auth.is_authentication());
        assert!(!auth.is_rate_limit());
        assert!(!auth.is_validation());
        assert!(!auth.is_network());
        assert!(!auth.is_api());

        let rate = new_rate_limit_error(429, "x", "", Duration::from_secs(1));
        assert!(!rate.is_authentication());
        assert!(rate.is_rate_limit());

        let val = new_validation_error(400, "x", "", "f");
        assert!(val.is_validation());

        let net = new_network_error(0, "x", "", false);
        assert!(net.is_network());

        let api = new_api_error(500, "x", "", "");
        assert!(api.is_api());
    }
}
