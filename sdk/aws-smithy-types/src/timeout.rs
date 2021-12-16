/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! This module defines types that describe timeouts for the various stages of an HTTP request.

use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;

/// Configuration for timeouts
///
/// # Example
///
/// ```rust
/// # use std::time::Duration;
///
/// # fn main() {
/// use aws_smithy_types::timeout::TimeoutConfig;
/// let timeout_config = TimeoutConfig::new()
///     .with_api_call_timeout(Some(Duration::from_secs(2)))
///     .with_api_call_attempt_timeout(Some(Duration::from_secs_f32(0.5)));
///
/// assert_eq!(
///     timeout_config.api_call_timeout(),
///     Some(Duration::from_secs(2))
/// );
///
/// assert_eq!(
///     timeout_config.api_call_attempt_timeout(),
///     Some(Duration::from_secs_f32(0.5))
/// );
/// # }
/// ```
#[derive(Clone, PartialEq, Default)]
pub struct TimeoutConfig {
    connect_timeout: Option<Duration>,
    tls_negotiation_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    api_call_attempt_timeout: Option<Duration>,
    api_call_timeout: Option<Duration>,
}

impl TimeoutConfig {
    /// Returns true if any of the possible timeouts are set
    pub fn has_timeouts(&self) -> bool {
        self.connect_timeout.is_some()
            || self.tls_negotiation_timeout.is_some()
            || self.read_timeout.is_some()
            || self.api_call_attempt_timeout.is_some()
            || self.api_call_timeout.is_some()
    }
}

impl Debug for TimeoutConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"Timeouts:
Connect (time to first byte):{}
TLS negotiation:{}
HTTP read:{}
API requests:{}
HTTP requests:{}
"#,
            format_timeout(self.connect_timeout),
            format_timeout(self.tls_negotiation_timeout),
            format_timeout(self.read_timeout),
            format_timeout(self.api_call_timeout),
            format_timeout(self.api_call_attempt_timeout),
        )
    }
}

fn format_timeout(timeout: Option<Duration>) -> String {
    timeout
        .map(|d| format!("\t{}s", d.as_secs_f32()))
        .unwrap_or_else(|| "(unset)".to_owned())
}

/// Parse a given string as a [`Duration`] that will be used to set a timeout. This will return an
/// error result if the given string is negative, infinite, equal to zero, NaN, or if the string
/// can't be parsed as an `f32`. The `name` and `set_by` fields are used to provide context when an
/// error occurs
///
/// # Example
///
/// ```should_panic
/// # use std::time::Duration;
/// # use aws_smithy_types::timeout::parse_str_as_timeout;
/// let duration = parse_str_as_timeout("8", "timeout".into(), "test_success".into()).unwrap();
/// assert_eq!(duration, Duration::from_secs_f32(8.0));
///
/// // This line will panic with "InvalidTimeout { name: "timeout", reason: "timeout must not be less than or equal to zero", set_by: "test_error" }"
/// let _ = parse_str_as_timeout("-1.0", "timeout".into(), "test_error".into()).unwrap();
/// ```
pub fn parse_str_as_timeout(
    timeout: &str,
    name: Cow<'static, str>,
    set_by: Cow<'static, str>,
) -> Result<Duration, TimeoutConfigError> {
    match timeout.parse::<f32>() {
        Ok(timeout) if timeout <= 0.0 => Err(TimeoutConfigError::InvalidTimeout {
            set_by,
            name,
            reason: "timeout must not be less than or equal to zero".into(),
        }),
        Ok(timeout) if timeout.is_nan() => Err(TimeoutConfigError::InvalidTimeout {
            set_by,
            name,
            reason: "timeout must not be NaN".into(),
        }),
        Ok(timeout) if timeout.is_infinite() => Err(TimeoutConfigError::InvalidTimeout {
            set_by,
            name,
            reason: "timeout must not be infinite".into(),
        }),
        Ok(timeout) => Ok(Duration::from_secs_f32(timeout)),
        Err(err) => Err(TimeoutConfigError::ParseError {
            set_by,
            name,
            source: Box::new(err),
        }),
    }
}

impl TimeoutConfig {
    /// Create a new `TimeoutConfig` with no timeouts set
    pub fn new() -> Self {
        Default::default()
    }

    /// A limit on the amount of time after making an initial connect attempt on a socket to complete the connect-handshake.
    pub fn connect_timeout(&self) -> Option<Duration> {
        self.connect_timeout
    }

    /// A limit on the amount of time a TLS handshake takes from when the `CLIENT HELLO` message is
    /// sent to the time the client and server have fully negotiated ciphers and exchanged keys.
    pub fn tls_negotiation_timeout(&self) -> Option<Duration> {
        self.tls_negotiation_timeout
    }

    /// A limit on the amount of time an application takes to attempt to read the first byte over an
    /// established, open connection after write request. This is also known as the
    /// "time to first byte" timeout.
    pub fn read_timeout(&self) -> Option<Duration> {
        self.read_timeout
    }

    /// A limit on the amount of time it takes for the first byte to be sent over an established,
    /// open connection and when the last byte is received from the service for a single attempt.
    /// If you want to set a timeout for an entire request including retry attempts,
    /// use [`TimeoutConfig::api_call_timeout`] instead.
    pub fn api_call_attempt_timeout(&self) -> Option<Duration> {
        self.api_call_attempt_timeout
    }

    /// A limit on the amount of time it takes for request to complete. A single request may be
    /// comprised of several attemps depending on an app's [`RetryConfig`](super::retry::RetryConfig). If you want
    /// to control timeouts for a single attempt, use [`TimeoutConfig::api_call_attempt_timeout`].
    pub fn api_call_timeout(&self) -> Option<Duration> {
        self.api_call_timeout
    }

    /// Consume a `TimeoutConfig` to create a new one, setting the connect timeout
    pub fn with_connect_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Consume a `TimeoutConfig` to create a new one, setting the TLS negotiation timeout
    pub fn with_tls_negotiation_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.tls_negotiation_timeout = timeout;
        self
    }

    /// Consume a `TimeoutConfig` to create a new one, setting the read timeout
    pub fn with_read_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.read_timeout = timeout;
        self
    }

    /// Consume a `TimeoutConfig` to create a new one, setting the API call attempt timeout
    pub fn with_api_call_attempt_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.api_call_attempt_timeout = timeout;
        self
    }

    /// Consume a `TimeoutConfig` to create a new one, setting the API call timeout
    pub fn with_api_call_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.api_call_timeout = timeout;
        self
    }

    /// Merges two builders together.
    ///
    /// Values from `other` will only be used as a fallback for values
    /// from `self`. Useful for merging configs from different sources together when you want to
    /// handle "precedence" per value instead of at the config level
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::time::Duration;
    /// # use aws_smithy_types::timeout::TimeoutConfig;
    /// let a = TimeoutConfig::new().with_read_timeout(Some(Duration::from_secs(2)));
    /// let b = TimeoutConfig::new()
    ///     .with_read_timeout(Some(Duration::from_secs(10)))
    ///     .with_connect_timeout(Some(Duration::from_secs(3)));
    /// let timeout_config = a.take_unset_from(b);
    /// // A's value take precedence over B's value
    /// assert_eq!(timeout_config.read_timeout(), Some(Duration::from_secs(2)));
    /// // A never set a connect timeout so B's value was used
    /// assert_eq!(timeout_config.connect_timeout(), Some(Duration::from_secs(3)));
    /// ```
    pub fn take_unset_from(self, other: Self) -> Self {
        Self {
            connect_timeout: self.connect_timeout.or(other.connect_timeout),
            tls_negotiation_timeout: self
                .tls_negotiation_timeout
                .or(other.tls_negotiation_timeout),
            read_timeout: self.read_timeout.or(other.read_timeout),
            api_call_attempt_timeout: self
                .api_call_attempt_timeout
                .or(other.api_call_attempt_timeout),
            api_call_timeout: self.api_call_timeout.or(other.api_call_timeout),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
/// An error that occurs during construction of a `TimeoutConfig`
pub enum TimeoutConfigError {
    /// A timeout value was set to an invalid value:
    /// - Any number less than 0
    /// - Infinity or negative infinity
    /// - `NaN`
    InvalidTimeout {
        /// The name of the invalid value
        name: Cow<'static, str>,
        /// The reason that why the timeout was considered invalid
        reason: Cow<'static, str>,
        /// Where the invalid value originated from
        set_by: Cow<'static, str>,
    },
    /// The timeout value couln't be parsed as an `f32`
    ParseError {
        /// The name of the invalid value
        name: Cow<'static, str>,
        /// Where the invalid value originated from
        set_by: Cow<'static, str>,
        /// The source of this error
        source: Box<dyn std::error::Error>,
    },
}

impl Display for TimeoutConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use TimeoutConfigError::*;
        match self {
            InvalidTimeout {
                name,
                set_by,
                reason,
            } => {
                write!(
                    f,
                    "invalid timeout '{}' set by {} is invalid: {}",
                    name, set_by, reason
                )
            }
            ParseError {
                name,
                set_by,
                source,
            } => {
                write!(
                    f,
                    "timeout '{}' set by {} could not be parsed as an f32: {}",
                    name, set_by, source
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_str_as_timeout, TimeoutConfig};
    use std::time::Duration;

    #[test]
    fn retry_config_builder_merge_with_favors_self_values_over_other_values() {
        let one_second = Some(Duration::from_secs(1));
        let two_seconds = Some(Duration::from_secs(2));

        let self_config = TimeoutConfig::new()
            .with_connect_timeout(one_second)
            .with_read_timeout(one_second)
            .with_tls_negotiation_timeout(one_second)
            .with_api_call_timeout(one_second)
            .with_api_call_attempt_timeout(one_second);
        let other_config = TimeoutConfig::new()
            .with_connect_timeout(two_seconds)
            .with_read_timeout(two_seconds)
            .with_tls_negotiation_timeout(two_seconds)
            .with_api_call_timeout(two_seconds)
            .with_api_call_attempt_timeout(two_seconds);
        let timeout_config = self_config.take_unset_from(other_config);

        assert_eq!(timeout_config.connect_timeout(), one_second);
        assert_eq!(timeout_config.read_timeout(), one_second);
        assert_eq!(timeout_config.tls_negotiation_timeout(), one_second);
        assert_eq!(timeout_config.api_call_timeout(), one_second);
        assert_eq!(timeout_config.api_call_attempt_timeout(), one_second);
    }

    #[test]
    fn test_integer_timeouts_are_parseable() {
        let duration = parse_str_as_timeout("8", "timeout".into(), "test".into()).unwrap();
        assert_eq!(duration, Duration::from_secs_f32(8.0));
    }

    #[test]
    #[should_panic = r#"ParseError { name: "timeout", set_by: "test", source: ParseFloatError { kind: Invalid } }"#]
    fn test_unparseable_timeouts_produce_an_error() {
        let _ = parse_str_as_timeout(
            "this sentence cant be parsed as a number",
            "timeout".into(),
            "test".into(),
        )
        .unwrap();
    }

    #[test]
    #[should_panic = r#"InvalidTimeout { name: "timeout", reason: "timeout must not be less than or equal to zero", set_by: "test" }"#]
    fn test_negative_timeouts_are_invalid() {
        let _ = parse_str_as_timeout("-1.0", "timeout".into(), "test".into()).unwrap();
    }

    #[test]
    #[should_panic = r#"InvalidTimeout { name: "timeout", reason: "timeout must not be less than or equal to zero", set_by: "test" }"#]
    fn test_setting_timeout_to_zero_is_invalid() {
        let _ = parse_str_as_timeout("0", "timeout".into(), "test".into()).unwrap();
    }

    #[test]
    #[should_panic = r#"InvalidTimeout { name: "timeout", reason: "timeout must not be NaN", set_by: "test" }"#]
    fn test_nan_timeouts_are_invalid() {
        let _ = parse_str_as_timeout("NaN", "timeout".into(), "test".into()).unwrap();
    }

    #[test]
    #[should_panic = r#"InvalidTimeout { name: "timeout", reason: "timeout must not be infinite", set_by: "test" }"#]
    fn test_infinite_timeouts_are_invalid() {
        let _ = parse_str_as_timeout("inf", "timeout".into(), "test".into()).unwrap();
    }
}
