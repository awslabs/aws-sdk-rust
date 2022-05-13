/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;
use std::time::Duration;

use aws_smithy_types::timeout;

/// Parse a given string as a [`Duration`] that will be used to set a timeout. This will return an
/// error result if the given string is negative, infinite, equal to zero, NaN, or if the string
/// can't be parsed as an `f32`. The `name` and `set_by` fields are used to provide context when an
/// error occurs
///
/// # Example
///
/// ```dont_run
/// # use std::time::Duration;
/// use aws_config::parsing::parse_str_as_timeout;
/// let duration = parse_str_as_timeout("8", "timeout".into(), "test_success".into()).unwrap();
/// assert_eq!(duration, Duration::from_secs_f32(8.0));
///
/// // This line will panic with `InvalidTimeout { name: "timeout", reason: "timeout must not be less than or equal to zero", set_by: "test_error" }`
/// let _ = parse_str_as_timeout("-1.0", "timeout".into(), "test_error".into()).unwrap();
/// ```
pub(crate) fn parse_str_as_timeout(
    timeout: &str,
    name: Cow<'static, str>,
    set_by: Cow<'static, str>,
) -> Result<Duration, timeout::ConfigError> {
    match timeout.parse::<f32>() {
        Ok(timeout) if timeout <= 0.0 => Err(timeout::ConfigError::InvalidTimeout {
            set_by,
            name,
            reason: "timeout must not be less than or equal to zero".into(),
        }),
        Ok(timeout) if timeout.is_nan() => Err(timeout::ConfigError::InvalidTimeout {
            set_by,
            name,
            reason: "timeout must not be NaN".into(),
        }),
        Ok(timeout) if timeout.is_infinite() => Err(timeout::ConfigError::InvalidTimeout {
            set_by,
            name,
            reason: "timeout must not be infinite".into(),
        }),
        Ok(timeout) => Ok(Duration::from_secs_f32(timeout)),
        Err(err) => Err(timeout::ConfigError::ParseError {
            set_by,
            name,
            source: Box::new(err),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_str_as_timeout;
    use std::time::Duration;

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
