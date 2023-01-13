/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Credentials Provider for external process

use crate::json_credentials::{json_parse_loop, InvalidJsonCredentials, RefreshableCredentials};
use aws_credential_types::provider::{self, error::CredentialsError, future, ProvideCredentials};
use aws_credential_types::Credentials;
use aws_smithy_json::deserialize::Token;
use std::fmt;
use std::process::Command;
use std::time::SystemTime;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Clone)]
pub(crate) struct CommandWithSensitiveArgs<T>(T);

impl<T> CommandWithSensitiveArgs<T>
where
    T: AsRef<str>,
{
    pub(crate) fn new(value: T) -> Self {
        Self(value)
    }

    pub(crate) fn unredacted(&self) -> &str {
        self.0.as_ref()
    }
}

impl<T> fmt::Display for CommandWithSensitiveArgs<T>
where
    T: AsRef<str>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Security: The arguments for command must be redacted since they can be sensitive
        let command = self.0.as_ref();
        match command.find(char::is_whitespace) {
            Some(index) => write!(f, "{} ** arguments redacted **", &command[0..index]),
            None => write!(f, "{}", command),
        }
    }
}

impl<T> fmt::Debug for CommandWithSensitiveArgs<T>
where
    T: AsRef<str>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", format!("{}", self))
    }
}

/// External process credentials provider
///
/// This credentials provider runs a configured external process and parses
/// its output to retrieve credentials.
///
/// The external process must exit with status 0 and output the following
/// JSON format to `stdout` to provide credentials:
///
/// ```json
/// {
///     "Version:" 1,
///     "AccessKeyId": "access key id",
///     "SecretAccessKey": "secret access key",
///     "SessionToken": "session token",
///     "Expiration": "time that the expiration will expire"
/// }
/// ```
///
/// The `Version` must be set to 1. `AccessKeyId` and `SecretAccessKey` are always required.
/// `SessionToken` must be set if a session token is associated with the `AccessKeyId`.
/// The `Expiration` is optional, and must be given in the RFC 3339 date time format (e.g.,
/// `2022-05-26T12:34:56.789Z`).
///
/// If the external process exits with a non-zero status, then the contents of `stderr`
/// will be output as part of the credentials provider error message.
///
/// This credentials provider is included in the profile credentials provider, and can be
/// configured using the `credential_process` attribute. For example:
///
/// ```plain
/// [profile example]
/// credential_process = /path/to/my/process --some --arguments
/// ```
#[derive(Debug)]
pub struct CredentialProcessProvider {
    command: CommandWithSensitiveArgs<String>,
}

impl ProvideCredentials for CredentialProcessProvider {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        future::ProvideCredentials::new(self.credentials())
    }
}

impl CredentialProcessProvider {
    /// Create new [`CredentialProcessProvider`] with the `command` needed to execute the external process.
    pub fn new(command: String) -> Self {
        Self {
            command: CommandWithSensitiveArgs::new(command),
        }
    }

    async fn credentials(&self) -> provider::Result {
        // Security: command arguments must be redacted at debug level
        tracing::debug!(command = %self.command, "loading credentials from external process");

        let mut command = if cfg!(windows) {
            let mut command = Command::new("cmd.exe");
            command.args(["/C", self.command.unredacted()]);
            command
        } else {
            let mut command = Command::new("sh");
            command.args(["-c", self.command.unredacted()]);
            command
        };

        let output = command.output().map_err(|e| {
            CredentialsError::provider_error(format!(
                "Error retrieving credentials from external process: {}",
                e
            ))
        })?;

        // Security: command arguments can be logged at trace level
        tracing::trace!(command = ?command, status = ?output.status, "executed command (unredacted)");

        if !output.status.success() {
            let reason =
                std::str::from_utf8(&output.stderr).unwrap_or("could not decode stderr as UTF-8");
            return Err(CredentialsError::provider_error(format!(
                "Error retrieving credentials: external process exited with code {}. Stderr: {}",
                output.status, reason
            )));
        }

        let output = std::str::from_utf8(&output.stdout).map_err(|e| {
            CredentialsError::provider_error(format!(
                "Error retrieving credentials from external process: could not decode output as UTF-8: {}",
                e
            ))
        })?;

        match parse_credential_process_json_credentials(output) {
            Ok(RefreshableCredentials {
                access_key_id,
                secret_access_key,
                session_token,
                expiration,
                ..
            }) => Ok(Credentials::new(
                access_key_id,
                secret_access_key,
                Some(session_token.to_string()),
                expiration.into(),
                "CredentialProcess",
            )),
            Err(invalid) => Err(CredentialsError::provider_error(format!(
                "Error retrieving credentials from external process, could not parse response: {}",
                invalid
            ))),
        }
    }
}

/// Deserialize a credential_process response from a string
///
/// Returns an error if the response cannot be successfully parsed or is missing keys.
///
/// Keys are case insensitive.
pub(crate) fn parse_credential_process_json_credentials(
    credentials_response: &str,
) -> Result<RefreshableCredentials<'_>, InvalidJsonCredentials> {
    let mut version = None;
    let mut access_key_id = None;
    let mut secret_access_key = None;
    let mut session_token = None;
    let mut expiration = None;
    json_parse_loop(credentials_response.as_bytes(), |key, value| {
        match (key, value) {
            /*
             "Version": 1,
             "AccessKeyId": "ASIARTESTID",
             "SecretAccessKey": "TESTSECRETKEY",
             "SessionToken": "TESTSESSIONTOKEN",
             "Expiration": "2022-05-02T18:36:00+00:00"
            */
            (key, Token::ValueNumber { value, .. }) if key.eq_ignore_ascii_case("Version") => {
                version = Some(i32::try_from(*value).map_err(|err| {
                    InvalidJsonCredentials::InvalidField {
                        field: "Version",
                        err: err.into(),
                    }
                })?);
            }
            (key, Token::ValueString { value, .. }) if key.eq_ignore_ascii_case("AccessKeyId") => {
                access_key_id = Some(value.to_unescaped()?)
            }
            (key, Token::ValueString { value, .. })
                if key.eq_ignore_ascii_case("SecretAccessKey") =>
            {
                secret_access_key = Some(value.to_unescaped()?)
            }
            (key, Token::ValueString { value, .. }) if key.eq_ignore_ascii_case("SessionToken") => {
                session_token = Some(value.to_unescaped()?)
            }
            (key, Token::ValueString { value, .. }) if key.eq_ignore_ascii_case("Expiration") => {
                expiration = Some(value.to_unescaped()?)
            }

            _ => {}
        };
        Ok(())
    })?;

    match version {
        Some(1) => { /* continue */ }
        None => return Err(InvalidJsonCredentials::MissingField("Version")),
        Some(version) => {
            return Err(InvalidJsonCredentials::InvalidField {
                field: "version",
                err: format!("unknown version number: {}", version).into(),
            })
        }
    }

    let access_key_id = access_key_id.ok_or(InvalidJsonCredentials::MissingField("AccessKeyId"))?;
    let secret_access_key =
        secret_access_key.ok_or(InvalidJsonCredentials::MissingField("SecretAccessKey"))?;
    let session_token = session_token.ok_or(InvalidJsonCredentials::MissingField("Token"))?;
    let expiration = expiration.ok_or(InvalidJsonCredentials::MissingField("Expiration"))?;
    let expiration =
        SystemTime::try_from(OffsetDateTime::parse(&expiration, &Rfc3339).map_err(|err| {
            InvalidJsonCredentials::InvalidField {
                field: "Expiration",
                err: err.into(),
            }
        })?)
        .map_err(|_| {
            InvalidJsonCredentials::Other(
                "credential expiration time cannot be represented by a DateTime".into(),
            )
        })?;
    Ok(RefreshableCredentials {
        access_key_id,
        secret_access_key,
        session_token,
        expiration,
    })
}

#[cfg(test)]
mod test {
    use crate::credential_process::CredentialProcessProvider;
    use aws_credential_types::provider::ProvideCredentials;
    use std::time::SystemTime;
    use time::format_description::well_known::Rfc3339;
    use time::OffsetDateTime;

    #[tokio::test]
    async fn test_credential_process() {
        let provider = CredentialProcessProvider::new(String::from(
            r#"echo '{ "Version": 1, "AccessKeyId": "ASIARTESTID", "SecretAccessKey": "TESTSECRETKEY", "SessionToken": "TESTSESSIONTOKEN", "Expiration": "2022-05-02T18:36:00+00:00" }'"#,
        ));
        let creds = provider.provide_credentials().await.expect("valid creds");
        assert_eq!(creds.access_key_id(), "ASIARTESTID");
        assert_eq!(creds.secret_access_key(), "TESTSECRETKEY");
        assert_eq!(creds.session_token(), Some("TESTSESSIONTOKEN"));
        assert_eq!(
            creds.expiry(),
            Some(
                SystemTime::try_from(
                    OffsetDateTime::parse("2022-05-02T18:36:00+00:00", &Rfc3339)
                        .expect("static datetime")
                )
                .expect("static datetime")
            )
        );
    }
}
