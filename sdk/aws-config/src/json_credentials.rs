/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_smithy_json::deserialize::token::skip_value;
use aws_smithy_json::deserialize::{json_token_iter, EscapeError, Token};
use aws_smithy_types::date_time::Format;
use aws_smithy_types::DateTime;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;

#[derive(Debug)]
pub(crate) enum InvalidJsonCredentials {
    /// The response did not contain valid JSON
    JsonError(Box<dyn Error + Send + Sync>),
    /// The response was missing a required field
    MissingField(&'static str),

    /// Another unhandled error occurred
    Other(Cow<'static, str>),
}

impl From<EscapeError> for InvalidJsonCredentials {
    fn from(err: EscapeError) -> Self {
        InvalidJsonCredentials::JsonError(err.into())
    }
}

impl From<aws_smithy_json::deserialize::Error> for InvalidJsonCredentials {
    fn from(err: aws_smithy_json::deserialize::Error) -> Self {
        InvalidJsonCredentials::JsonError(err.into())
    }
}

impl Display for InvalidJsonCredentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidJsonCredentials::JsonError(json) => {
                write!(f, "invalid JSON in response: {}", json)
            }
            InvalidJsonCredentials::MissingField(field) => write!(
                f,
                "Expected field `{}` in response but it was missing",
                field
            ),
            InvalidJsonCredentials::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for InvalidJsonCredentials {}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum JsonCredentials<'a> {
    RefreshableCredentials {
        access_key_id: Cow<'a, str>,
        secret_access_key: Cow<'a, str>,
        session_token: Cow<'a, str>,
        expiration: SystemTime,
    },
    Error {
        code: Cow<'a, str>,
        message: Cow<'a, str>,
    }, // TODO(https://github.com/awslabs/aws-sdk-rust/issues/340): Add support for static credentials:
       //  {
       //    "AccessKeyId" : "MUA...",
       //    "SecretAccessKey" : "/7PC5om...."
       //  }

       // TODO(https://github.com/awslabs/aws-sdk-rust/issues/340): Add support for Assume role credentials:
       //   {
       //     // fields to construct STS client:
       //     "Region": "sts-region-name",
       //     "AccessKeyId" : "MUA...",
       //     "Expiration" : "2016-02-25T06:03:31Z", // optional
       //     "SecretAccessKey" : "/7PC5om....",
       //     "Token" : "AQoDY....=", // optional
       //     // fields controlling the STS role:
       //     "RoleArn": "...", // required
       //     "RoleSessionName": "...", // required
       //     // and also: DurationSeconds, ExternalId, SerialNumber, TokenCode, Policy
       //     ...
       //   }
}

/// Deserialize an IMDS response from a string
///
/// There are two levels of error here: the top level distinguishes between a successfully parsed
/// response from the credential provider vs. something invalid / unexpected. The inner error
/// distinguishes between a successful response that contains credentials vs. an error with a code and
/// error message.
///
/// Keys are case insensitive.
pub(crate) fn parse_json_credentials(
    credentials_response: &str,
) -> Result<JsonCredentials, InvalidJsonCredentials> {
    let mut tokens = json_token_iter(credentials_response.as_bytes()).peekable();
    let mut code = None;
    let mut access_key_id = None;
    let mut secret_access_key = None;
    let mut session_token = None;
    let mut expiration = None;
    let mut message = None;
    if !matches!(tokens.next().transpose()?, Some(Token::StartObject { .. })) {
        return Err(InvalidJsonCredentials::JsonError(
            "expected a JSON document starting with `{`".into(),
        ));
    }
    loop {
        match tokens.next().transpose()? {
            Some(Token::EndObject { .. }) => break,
            Some(Token::ObjectKey { key, .. }) => {
                if let Some(Ok(Token::ValueString { value, .. })) = tokens.peek() {
                    match key.to_unescaped()? {
                        /*
                         "Code": "Success",
                         "Type": "AWS-HMAC",
                         "AccessKeyId" : "accessKey",
                         "SecretAccessKey" : "secret",
                         "Token" : "token",
                         "Expiration" : "....",
                         "LastUpdated" : "2009-11-23T0:00:00Z"
                        */
                        c if c.eq_ignore_ascii_case("Code") => code = Some(value.to_unescaped()?),
                        c if c.eq_ignore_ascii_case("AccessKeyId") => {
                            access_key_id = Some(value.to_unescaped()?)
                        }
                        c if c.eq_ignore_ascii_case("SecretAccessKey") => {
                            secret_access_key = Some(value.to_unescaped()?)
                        }
                        c if c.eq_ignore_ascii_case("Token") => {
                            session_token = Some(value.to_unescaped()?)
                        }
                        c if c.eq_ignore_ascii_case("Expiration") => {
                            expiration = Some(value.to_unescaped()?)
                        }

                        // Error case handling: message will be set
                        c if c.eq_ignore_ascii_case("Message") => {
                            message = Some(value.to_unescaped()?)
                        }
                        _ => {}
                    }
                }
                skip_value(&mut tokens)?;
            }
            other => {
                return Err(InvalidJsonCredentials::Other(
                    format!("expected object key, found: {:?}", other,).into(),
                ));
            }
        }
    }
    if tokens.next().is_some() {
        return Err(InvalidJsonCredentials::Other(
            "found more JSON tokens after completing parsing".into(),
        ));
    }
    match code {
        // IMDS does not appear to reply with a `Code` missing, but documentation indicates it
        // may be possible
        None | Some(Cow::Borrowed("Success")) => {
            let access_key_id =
                access_key_id.ok_or(InvalidJsonCredentials::MissingField("AccessKeyId"))?;
            let secret_access_key =
                secret_access_key.ok_or(InvalidJsonCredentials::MissingField("SecretAccessKey"))?;
            let session_token =
                session_token.ok_or(InvalidJsonCredentials::MissingField("Token"))?;
            let expiration =
                expiration.ok_or(InvalidJsonCredentials::MissingField("Expiration"))?;
            let expiration = SystemTime::try_from(
                DateTime::from_str(expiration.as_ref(), Format::DateTime).map_err(|err| {
                    InvalidJsonCredentials::Other(format!("invalid date: {}", err).into())
                })?,
            )
            .map_err(|_| {
                InvalidJsonCredentials::Other(
                    "credential expiration time cannot be represented by a SystemTime".into(),
                )
            })?;
            Ok(JsonCredentials::RefreshableCredentials {
                access_key_id,
                secret_access_key,
                session_token,
                expiration,
            })
        }
        Some(other) => Ok(JsonCredentials::Error {
            code: other,
            message: message.unwrap_or_else(|| "no message".into()),
        }),
    }
}

#[cfg(test)]
mod test {
    use crate::json_credentials::{
        parse_json_credentials, InvalidJsonCredentials, JsonCredentials,
    };
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn json_credentials_success_response() {
        let response = r#"
        {
          "Code" : "Success",
          "LastUpdated" : "2021-09-17T20:57:08Z",
          "Type" : "AWS-HMAC",
          "AccessKeyId" : "ASIARTEST",
          "SecretAccessKey" : "xjtest",
          "Token" : "IQote///test",
          "Expiration" : "2021-09-18T03:31:56Z"
        }"#;
        let parsed = parse_json_credentials(response).expect("valid JSON");
        assert_eq!(
            parsed,
            JsonCredentials::RefreshableCredentials {
                access_key_id: "ASIARTEST".into(),
                secret_access_key: "xjtest".into(),
                session_token: "IQote///test".into(),
                expiration: UNIX_EPOCH + Duration::from_secs(1631935916),
            }
        )
    }

    #[test]
    fn json_credentials_invalid_json() {
        let error = parse_json_credentials("404: not found").expect_err("no json");
        match error {
            InvalidJsonCredentials::JsonError(_) => {} // ok.
            err => panic!("incorrect error: {:?}", err),
        }
    }

    #[test]
    fn json_credentials_not_json_object() {
        let error = parse_json_credentials("[1,2,3]").expect_err("no json");
        match error {
            InvalidJsonCredentials::JsonError(_) => {} // ok.
            _ => panic!("incorrect error"),
        }
    }

    #[test]
    fn json_credentials_missing_code() {
        let resp = r#"{
            "LastUpdated" : "2021-09-17T20:57:08Z",
            "Type" : "AWS-HMAC",
            "AccessKeyId" : "ASIARTEST",
            "SecretAccessKey" : "xjtest",
            "Token" : "IQote///test",
            "Expiration" : "2021-09-18T03:31:56Z"
        }"#;
        let parsed = parse_json_credentials(resp).expect("code not required");
        assert_eq!(
            parsed,
            JsonCredentials::RefreshableCredentials {
                access_key_id: "ASIARTEST".into(),
                secret_access_key: "xjtest".into(),
                session_token: "IQote///test".into(),
                expiration: UNIX_EPOCH + Duration::from_secs(1631935916),
            }
        )
    }

    #[test]
    fn json_credentials_required_session_token() {
        let resp = r#"{
            "LastUpdated" : "2021-09-17T20:57:08Z",
            "Type" : "AWS-HMAC",
            "AccessKeyId" : "ASIARTEST",
            "SecretAccessKey" : "xjtest",
            "Expiration" : "2021-09-18T03:31:56Z"
        }"#;
        let parsed = parse_json_credentials(resp).expect_err("token missing");
        assert_eq!(
            format!("{}", parsed),
            "Expected field `Token` in response but it was missing"
        );
    }

    #[test]
    fn json_credentials_missing_akid() {
        let resp = r#"{
            "Code": "Success",
            "LastUpdated" : "2021-09-17T20:57:08Z",
            "Type" : "AWS-HMAC",
            "SecretAccessKey" : "xjtest",
            "Token" : "IQote///test",
            "Expiration" : "2021-09-18T03:31:56Z"
        }"#;
        match parse_json_credentials(resp).expect_err("no code") {
            InvalidJsonCredentials::MissingField("AccessKeyId") => {} // ok
            resp => panic!("incorrect json_credentials response: {:?}", resp),
        }
    }

    #[test]
    fn json_credentials_error_response() {
        let response = r#"{
          "Code" : "AssumeRoleUnauthorizedAccess",
          "Message" : "EC2 cannot assume the role integration-test.",
          "LastUpdated" : "2021-09-17T20:46:56Z"
        }"#;
        let parsed = parse_json_credentials(response).expect("valid JSON");
        assert_eq!(
            parsed,
            JsonCredentials::Error {
                code: "AssumeRoleUnauthorizedAccess".into(),
                message: "EC2 cannot assume the role integration-test.".into(),
            }
        );
    }

    /// Validate the specific JSON response format sent by ECS
    #[test]
    fn json_credentials_ecs() {
        // identical, but extra `RoleArn` field is present
        let response = r#"{
            "RoleArn":"arn:aws:iam::123456789:role/ecs-task-role",
            "AccessKeyId":"ASIARTEST",
            "SecretAccessKey":"SECRETTEST",
            "Token":"tokenEaCXVzLXdlc3QtMiJGMEQCIHt47W18eF4dYfSlmKGiwuJnqmIS3LMXNYfODBCEhcnaAiAnuhGOpcdIDxin4QFzhtgaCR2MpcVqR8NFJdMgOt0/xyrnAwhhEAEaDDEzNDA5NTA2NTg1NiIM9M9GT+c5UfV/8r7PKsQDUa9xE9Eprz5N+jgxbFSD2aJR2iyXCcP9Q1cOh4fdZhyw2WNmq9XnIa2tkzrreiQ5R2t+kzergJHO1KRZPfesarfJ879aWJCSocsEKh7xXwwzTsVXrNo5eWkpwTh64q+Ksz15eoaBhtrvnGvPx6SmXv7SToi/DTHFafJlT/T9jITACZvZXSE9zfLka26Rna3rI4g0ugowha//j1f/c1XuKloqshpZvMKc561om9Y5fqBv1fRiS2KhetGTcmz3wUqNQAk8Dq9oINS7cCtdIO0atqCK69UaKeJ9uKY8mzY9dFWw2IrkpOoXmA9r955iU0NOz/95jVJiPZ/8aE8vb0t67gQfzBUCfky+mGSGWAfPRXQlFa5AEulCTHPd7IcTVCtasG033oKEKgB8QnTxvM2LaPlwaaHo7MHGYXeUKbn9NRKd8m1ShwmAlr4oKp1vQp6cPHDTsdTfPTzh/ZAjUPs+ljQbAwqXbPQdUUPpOk0vltY8k6Im9EA0pf80iUNoqrixpmPsR2hzI/ybUwdh+QhvCSBx+J8KHqF6X92u4qAVYIxLy/LGZKT9YC6Kr9Gywn+Ro+EK/xl3axHPzNpbjRDJnbW3HrMw5LmmiwY6pgGWgmD6IOq4QYUtu1uhaLQZyoI5o5PWn+d3kqqxifu8D0ykldB3lQGdlJ2rjKJjCdx8fce1SoXao9cc4hiwn39hUPuTqzVwv2zbzCKmNggIpXP6gqyRtUCakf6tI7ZwqTb2S8KF3t4ElIP8i4cPdNoI0JHSC+sT4LDPpUcX1CjGxfvo55mBHJedW3LXve8TRj4UckFXT1gLuTnzqPMrC5AHz4TAt+uv",
            "Expiration" : "2009-02-13T23:31:30Z"
        }"#;
        let parsed = parse_json_credentials(response).expect("valid JSON");
        use std::borrow::Cow;
        assert!(
            matches!(
                &parsed,
                JsonCredentials::RefreshableCredentials {
                    access_key_id: Cow::Borrowed("ASIARTEST"),
                    secret_access_key: Cow::Borrowed("SECRETTEST"),
                    session_token,
                    expiration
                } if session_token.starts_with("token") && *expiration == UNIX_EPOCH + Duration::from_secs(1234567890)
            ),
            "{:?}",
            parsed
        );
    }

    #[test]
    fn case_insensitive_code_parsing() {
        let response = r#"{
          "code" : "AssumeRoleUnauthorizedAccess",
          "message" : "EC2 cannot assume the role integration-test."
        }"#;
        let parsed = parse_json_credentials(response).expect("valid JSON");
        assert_eq!(
            parsed,
            JsonCredentials::Error {
                code: "AssumeRoleUnauthorizedAccess".into(),
                message: "EC2 cannot assume the role integration-test.".into(),
            }
        );
    }
}
