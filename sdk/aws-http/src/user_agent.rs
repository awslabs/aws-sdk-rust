/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::build_metadata::{OsFamily, BUILD_METADATA};
use http::header::{HeaderName, InvalidHeaderValue, USER_AGENT};
use http::HeaderValue;
use smithy_http::middleware::MapRequest;
use smithy_http::operation::Request;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// AWS User Agent
///
/// Ths struct should be inserted into the [`PropertyBag`](smithy_http::operation::Request::config)
/// during operation construction. [`UserAgentStage`](UserAgentStage) reads `AwsUserAgent`
/// from the property bag and sets the `User-Agent` and `x-amz-user-agent` headers.
pub struct AwsUserAgent {
    sdk_metadata: SdkMetadata,
    api_metadata: ApiMetadata,
    os_metadata: OsMetadata,
    language_metadata: LanguageMetadata,
    exec_env_metadata: Option<ExecEnvMetadata>,
}

impl AwsUserAgent {
    /// Load a User Agent configuration from the environment
    ///
    /// This utilizes [`BUILD_METADATA`](static@aws_types::build_metadata::BUILD_METADATA) from `aws_types`
    /// to capture the Rust version & target platform. `ApiMetadata` provides
    /// the version & name of the specific service.
    pub fn new_from_environment(api_metadata: ApiMetadata) -> Self {
        let build_metadata = &BUILD_METADATA;
        let sdk_metadata = SdkMetadata {
            name: "rust",
            version: build_metadata.core_pkg_version,
        };
        let os_metadata = OsMetadata {
            os_family: &build_metadata.os_family,
            version: None,
        };
        AwsUserAgent {
            sdk_metadata,
            api_metadata,
            os_metadata,
            language_metadata: LanguageMetadata {
                lang: "rust",
                version: BUILD_METADATA.rust_version,
                extras: vec![],
            },
            exec_env_metadata: None,
        }
    }

    /// For test purposes, construct an environment-independent User Agent
    ///
    /// Without this, running CI on a different platform would produce different user agent strings
    pub fn for_tests() -> Self {
        Self {
            sdk_metadata: SdkMetadata {
                name: "rust",
                version: "0.123.test",
            },
            api_metadata: ApiMetadata {
                service_id: "test-service".into(),
                version: "0.123",
            },
            os_metadata: OsMetadata {
                os_family: &OsFamily::Windows,
                version: Some("XPSP3".to_string()),
            },
            language_metadata: LanguageMetadata {
                lang: "rust",
                version: "1.50.0",
                extras: vec![],
            },
            exec_env_metadata: None,
        }
    }

    /// Generate a new-style user agent style header
    ///
    /// This header should be set at `x-amz-user-agent`
    pub fn aws_ua_header(&self) -> String {
        /*
        ABNF for the user agent:
        ua-string =
                        sdk-metadata RWS
                       [api-metadata RWS]
                       os-metadata RWS
                       language-metadata RWS
                       [env-metadata RWS]
                       *(feat-metadata RWS)
                       *(config-metadata RWS)
                       *(framework-metadata RWS)
                       [appId]
        */
        let mut ua_value = String::new();
        use std::fmt::Write;
        // unwrap calls should never fail because string formatting will always succeed.
        write!(ua_value, "{} ", &self.sdk_metadata).unwrap();
        write!(ua_value, "{} ", &self.api_metadata).unwrap();
        write!(ua_value, "{} ", &self.os_metadata).unwrap();
        write!(ua_value, "{} ", &self.language_metadata).unwrap();
        if let Some(ref env_meta) = self.exec_env_metadata {
            write!(ua_value, "{} ", env_meta).unwrap();
        }
        // TODO: feature metadata
        // TODO: config metadata
        // TODO: framework metadata
        // TODO: appId
        if ua_value.ends_with(' ') {
            ua_value.truncate(ua_value.len() - 1);
        }
        ua_value
    }

    /// Generate an old-style User-Agent header for backward compatibility
    ///
    /// This header is intended to be set at `User-Agent`
    pub fn ua_header(&self) -> String {
        let mut ua_value = String::new();
        use std::fmt::Write;
        write!(ua_value, "{} ", &self.sdk_metadata).unwrap();
        write!(ua_value, "{} ", &self.os_metadata).unwrap();
        write!(ua_value, "{}", &self.language_metadata).unwrap();
        ua_value
    }
}

pub struct SdkMetadata {
    name: &'static str,
    version: &'static str,
}

impl Display for SdkMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "aws-sdk-{}/{}", self.name, self.version)
    }
}

#[derive(Clone)]
pub struct ApiMetadata {
    service_id: Cow<'static, str>,
    version: &'static str,
}

impl ApiMetadata {
    pub const fn new(service_id: &'static str, version: &'static str) -> Self {
        Self {
            service_id: Cow::Borrowed(service_id),
            version,
        }
    }
}

impl Display for ApiMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "api/{}/{}", self.service_id, self.version)
    }
}

struct AdditionalMetadata {
    key: String,
    value: String,
}

struct OsMetadata {
    os_family: &'static OsFamily,
    version: Option<String>,
}

impl Display for OsMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let os_family = match self.os_family {
            OsFamily::Windows => "windows",
            OsFamily::Linux => "linux",
            OsFamily::Macos => "macos",
            OsFamily::Android => "android",
            OsFamily::Ios => "ios",
            OsFamily::Other => "other",
        };
        write!(f, "os/{}", os_family)?;
        if let Some(ref version) = self.version {
            write!(f, "/{}", version)?;
        }
        Ok(())
    }
}
struct LanguageMetadata {
    lang: &'static str,
    version: &'static str,
    extras: Vec<AdditionalMetadata>,
}
impl Display for LanguageMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // language-metadata    = "lang/" language "/" version *(RWS additional-metadata)
        write!(f, "lang/{}/{}", self.lang, self.version)?;
        for extra in &self.extras {
            write!(f, " md/{}/{}", &extra.key, &extra.value)?;
        }
        Ok(())
    }
}
struct ExecEnvMetadata {
    name: String,
}
impl Display for ExecEnvMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "exec-env/{}", &self.name)
    }
}

#[non_exhaustive]
#[derive(Default, Clone)]
pub struct UserAgentStage;

impl UserAgentStage {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Error)]
pub enum UserAgentStageError {
    #[error("User agent missing from property bag")]
    UserAgentMissing,
    #[error("Provided user agent header was invalid")]
    InvalidUAHeader(#[from] InvalidHeaderValue),
}

lazy_static::lazy_static! {
    static ref X_AMZ_USER_AGENT: HeaderName = HeaderName::from_static("x-amz-user-agent");
}

impl MapRequest for UserAgentStage {
    type Error = UserAgentStageError;

    fn apply(&self, request: Request) -> Result<Request, Self::Error> {
        request.augment(|mut req, conf| {
            let ua = conf
                .get::<AwsUserAgent>()
                .ok_or(UserAgentStageError::UserAgentMissing)?;
            // TODO: consider optimizing by caching the user agent headers to avoid the alloc on every request
            req.headers_mut()
                .append(USER_AGENT, HeaderValue::try_from(ua.ua_header())?);
            req.headers_mut().append(
                X_AMZ_USER_AGENT.clone(),
                HeaderValue::try_from(ua.aws_ua_header())?,
            );

            Ok(req)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::user_agent::X_AMZ_USER_AGENT;
    use crate::user_agent::{ApiMetadata, AwsUserAgent, UserAgentStage};
    use aws_types::build_metadata::OsFamily;
    use http::header::USER_AGENT;
    use smithy_http::body::SdkBody;
    use smithy_http::middleware::MapRequest;
    use smithy_http::operation;

    #[test]
    fn generate_a_valid_ua() {
        let api_metadata = ApiMetadata {
            service_id: "dynamodb".into(),
            version: "123",
        };
        let mut ua = AwsUserAgent::new_from_environment(api_metadata);
        // hard code some variable things for a deterministic test
        ua.sdk_metadata.version = "0.1";
        ua.language_metadata.version = "1.50.0";
        ua.os_metadata.os_family = &OsFamily::Macos;
        ua.os_metadata.version = Some("1.15".to_string());
        assert_eq!(
            ua.aws_ua_header(),
            "aws-sdk-rust/0.1 api/dynamodb/123 os/macos/1.15 lang/rust/1.50.0"
        );
        assert_eq!(
            ua.ua_header(),
            "aws-sdk-rust/0.1 os/macos/1.15 lang/rust/1.50.0"
        );
    }

    #[test]
    fn ua_stage_adds_headers() {
        let stage = UserAgentStage::new();
        let req = operation::Request::new(http::Request::new(SdkBody::from("some body")));
        stage
            .apply(req)
            .expect_err("adding UA should fail without a UA set");
        let mut req = operation::Request::new(http::Request::new(SdkBody::from("some body")));
        req.config_mut()
            .insert(AwsUserAgent::new_from_environment(ApiMetadata {
                service_id: "dynamodb".into(),
                version: "0.123",
            }));
        let req = stage.apply(req).expect("setting user agent should succeed");
        let (req, _) = req.into_parts();
        req.headers()
            .get(USER_AGENT)
            .expect("UA header should be set");
        req.headers()
            .get(&*X_AMZ_USER_AGENT)
            .expect("UA header should be set");
    }
}

/*
Appendix: User Agent ABNF
sdk-ua-header        = "x-amz-user-agent:" OWS ua-string OWS
ua-pair              = ua-name ["/" ua-value]
ua-name              = token
ua-value             = token
version              = token
name                 = token
service-id           = token
sdk-name             = java / ruby / php / dotnet / python / cli / kotlin / rust / js / cpp / go / go-v2
os-family            = windows / linux / macos / android / ios / other
config               = retry-mode
additional-metadata  = "md/" ua-pair
sdk-metadata         = "aws-sdk-" sdk-name "/" version
api-metadata         = "api/" service-id "/" version
os-metadata          = "os/" os-family ["/" version]
language-metadata    = "lang/" language "/" version *(RWS additional-metadata)
env-metadata         = "exec-env/" name
feat-metadata        = "ft/" name ["/" version] *(RWS additional-metadata)
config-metadata      = "cfg/" config "/" name
framework-metadata   = "lib/" name ["/" version] *(RWS additional-metadata)
appId                = "app/" name
ua-string            = sdk-metadata RWS
                       [api-metadata RWS]
                       os-metadata RWS
                       language-metadata RWS
                       [env-metadata RWS]
                       *(feat-metadata RWS)
                       *(config-metadata RWS)
                       *(framework-metadata RWS)
                       [appId]

# New metadata field might be added in the future and they must follow this format
prefix               = token
metadata             = prefix "/" ua-pair

# token, RWS and OWS are defined in [RFC 7230](https://tools.ietf.org/html/rfc7230)
OWS            = *( SP / HTAB )
               ; optional whitespace
RWS            = 1*( SP / HTAB )
               ; required whitespace
token          = 1*tchar
tchar          = "!" / "#" / "$" / "%" / "&" / "'" / "*" / "+" / "-" / "." /
                 "^" / "_" / "`" / "|" / "~" / DIGIT / ALPHA
*/
