/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::middleware::MapRequest;
use aws_smithy_http::operation::Request;
use aws_smithy_types::config_bag::{Storable, StoreReplace};
use aws_types::app_name::AppName;
use aws_types::build_metadata::{OsFamily, BUILD_METADATA};
use aws_types::os_shim_internal::Env;
use http::header::{HeaderName, InvalidHeaderValue, USER_AGENT};
use http::HeaderValue;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;

/// AWS User Agent
///
/// Ths struct should be inserted into the [`PropertyBag`](aws_smithy_http::operation::Request::properties)
/// during operation construction. [`UserAgentStage`](UserAgentStage) reads `AwsUserAgent`
/// from the property bag and sets the `User-Agent` and `x-amz-user-agent` headers.
#[derive(Clone, Debug)]
pub struct AwsUserAgent {
    sdk_metadata: SdkMetadata,
    api_metadata: ApiMetadata,
    os_metadata: OsMetadata,
    language_metadata: LanguageMetadata,
    exec_env_metadata: Option<ExecEnvMetadata>,
    feature_metadata: Vec<FeatureMetadata>,
    config_metadata: Vec<ConfigMetadata>,
    framework_metadata: Vec<FrameworkMetadata>,
    app_name: Option<AppName>,
}

impl AwsUserAgent {
    /// Load a User Agent configuration from the environment
    ///
    /// This utilizes [`BUILD_METADATA`](const@aws_types::build_metadata::BUILD_METADATA) from `aws_types`
    /// to capture the Rust version & target platform. `ApiMetadata` provides
    /// the version & name of the specific service.
    pub fn new_from_environment(env: Env, api_metadata: ApiMetadata) -> Self {
        let build_metadata = &BUILD_METADATA;
        let sdk_metadata = SdkMetadata {
            name: "rust",
            version: build_metadata.core_pkg_version,
        };
        let os_metadata = OsMetadata {
            os_family: &build_metadata.os_family,
            version: None,
        };
        let exec_env_metadata = env
            .get("AWS_EXECUTION_ENV")
            .ok()
            .map(|name| ExecEnvMetadata { name });
        AwsUserAgent {
            sdk_metadata,
            api_metadata,
            os_metadata,
            language_metadata: LanguageMetadata {
                lang: "rust",
                version: BUILD_METADATA.rust_version,
                extras: Default::default(),
            },
            exec_env_metadata,
            feature_metadata: Default::default(),
            config_metadata: Default::default(),
            framework_metadata: Default::default(),
            app_name: Default::default(),
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
                extras: Default::default(),
            },
            exec_env_metadata: None,
            feature_metadata: Vec::new(),
            config_metadata: Vec::new(),
            framework_metadata: Vec::new(),
            app_name: None,
        }
    }

    #[doc(hidden)]
    /// Adds feature metadata to the user agent.
    pub fn with_feature_metadata(mut self, metadata: FeatureMetadata) -> Self {
        self.feature_metadata.push(metadata);
        self
    }

    #[doc(hidden)]
    /// Adds feature metadata to the user agent.
    pub fn add_feature_metadata(&mut self, metadata: FeatureMetadata) -> &mut Self {
        self.feature_metadata.push(metadata);
        self
    }

    #[doc(hidden)]
    /// Adds config metadata to the user agent.
    pub fn with_config_metadata(mut self, metadata: ConfigMetadata) -> Self {
        self.config_metadata.push(metadata);
        self
    }

    #[doc(hidden)]
    /// Adds config metadata to the user agent.
    pub fn add_config_metadata(&mut self, metadata: ConfigMetadata) -> &mut Self {
        self.config_metadata.push(metadata);
        self
    }

    #[doc(hidden)]
    /// Adds framework metadata to the user agent.
    pub fn with_framework_metadata(mut self, metadata: FrameworkMetadata) -> Self {
        self.framework_metadata.push(metadata);
        self
    }

    #[doc(hidden)]
    /// Adds framework metadata to the user agent.
    pub fn add_framework_metadata(&mut self, metadata: FrameworkMetadata) -> &mut Self {
        self.framework_metadata.push(metadata);
        self
    }

    /// Sets the app name for the user agent.
    pub fn with_app_name(mut self, app_name: AppName) -> Self {
        self.app_name = Some(app_name);
        self
    }

    /// Sets the app name for the user agent.
    pub fn set_app_name(&mut self, app_name: AppName) -> &mut Self {
        self.app_name = Some(app_name);
        self
    }

    /// Generate a new-style user agent style header
    ///
    /// This header should be set at `x-amz-user-agent`
    pub fn aws_ua_header(&self) -> String {
        /*
        ABNF for the user agent (see the bottom of the file for complete ABNF):
        ua-string = sdk-metadata RWS
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
        for feature in &self.feature_metadata {
            write!(ua_value, "{} ", feature).unwrap();
        }
        for config in &self.config_metadata {
            write!(ua_value, "{} ", config).unwrap();
        }
        for framework in &self.framework_metadata {
            write!(ua_value, "{} ", framework).unwrap();
        }
        if let Some(app_name) = &self.app_name {
            write!(ua_value, "app/{}", app_name).unwrap();
        }
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

impl Storable for AwsUserAgent {
    type Storer = StoreReplace<Self>;
}

#[derive(Clone, Copy, Debug)]
struct SdkMetadata {
    name: &'static str,
    version: &'static str,
}

impl fmt::Display for SdkMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "aws-sdk-{}/{}", self.name, self.version)
    }
}

/// Metadata about the client that's making the call.
#[derive(Clone, Debug)]
pub struct ApiMetadata {
    service_id: Cow<'static, str>,
    version: &'static str,
}

impl ApiMetadata {
    /// Creates new `ApiMetadata`.
    pub const fn new(service_id: &'static str, version: &'static str) -> Self {
        Self {
            service_id: Cow::Borrowed(service_id),
            version,
        }
    }
}

impl fmt::Display for ApiMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "api/{}/{}", self.service_id, self.version)
    }
}

impl Storable for ApiMetadata {
    type Storer = StoreReplace<Self>;
}

/// Error for when an user agent metadata doesn't meet character requirements.
///
/// Metadata may only have alphanumeric characters and any of these characters:
/// ```text
/// !#$%&'*+-.^_`|~
/// ```
/// Spaces are not allowed.
#[derive(Debug)]
#[non_exhaustive]
pub struct InvalidMetadataValue;

impl Error for InvalidMetadataValue {}

impl fmt::Display for InvalidMetadataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "User agent metadata can only have alphanumeric characters, or any of \
             '!' |  '#' |  '$' |  '%' |  '&' |  '\\'' |  '*' |  '+' |  '-' | \
             '.' |  '^' |  '_' |  '`' |  '|' |  '~'"
        )
    }
}

fn validate_metadata(value: Cow<'static, str>) -> Result<Cow<'static, str>, InvalidMetadataValue> {
    fn valid_character(c: char) -> bool {
        match c {
            _ if c.is_ascii_alphanumeric() => true,
            '!' | '#' | '$' | '%' | '&' | '\'' | '*' | '+' | '-' | '.' | '^' | '_' | '`' | '|'
            | '~' => true,
            _ => false,
        }
    }
    if !value.chars().all(valid_character) {
        return Err(InvalidMetadataValue);
    }
    Ok(value)
}

#[doc(hidden)]
/// Additional metadata that can be bundled with framework or feature metadata.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct AdditionalMetadata {
    value: Cow<'static, str>,
}

impl AdditionalMetadata {
    /// Creates `AdditionalMetadata`.
    ///
    /// This will result in `InvalidMetadataValue` if the given value isn't alphanumeric or
    /// has characters other than the following:
    /// ```text
    /// !#$%&'*+-.^_`|~
    /// ```
    pub fn new(value: impl Into<Cow<'static, str>>) -> Result<Self, InvalidMetadataValue> {
        Ok(Self {
            value: validate_metadata(value.into())?,
        })
    }
}

impl fmt::Display for AdditionalMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // additional-metadata = "md/" ua-pair
        write!(f, "md/{}", self.value)
    }
}

#[derive(Clone, Debug, Default)]
struct AdditionalMetadataList(Vec<AdditionalMetadata>);

impl AdditionalMetadataList {
    fn push(&mut self, metadata: AdditionalMetadata) {
        self.0.push(metadata);
    }
}

impl fmt::Display for AdditionalMetadataList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for metadata in &self.0 {
            write!(f, " {}", metadata)?;
        }
        Ok(())
    }
}

#[doc(hidden)]
/// Metadata about a feature that is being used in the SDK.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct FeatureMetadata {
    name: Cow<'static, str>,
    version: Option<Cow<'static, str>>,
    additional: AdditionalMetadataList,
}

impl FeatureMetadata {
    /// Creates `FeatureMetadata`.
    ///
    /// This will result in `InvalidMetadataValue` if the given value isn't alphanumeric or
    /// has characters other than the following:
    /// ```text
    /// !#$%&'*+-.^_`|~
    /// ```
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        version: Option<Cow<'static, str>>,
    ) -> Result<Self, InvalidMetadataValue> {
        Ok(Self {
            name: validate_metadata(name.into())?,
            version: version.map(validate_metadata).transpose()?,
            additional: Default::default(),
        })
    }

    /// Bundles additional arbitrary metadata with this feature metadata.
    pub fn with_additional(mut self, metadata: AdditionalMetadata) -> Self {
        self.additional.push(metadata);
        self
    }
}

impl fmt::Display for FeatureMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // feat-metadata = "ft/" name ["/" version] *(RWS additional-metadata)
        if let Some(version) = &self.version {
            write!(f, "ft/{}/{}{}", self.name, version, self.additional)
        } else {
            write!(f, "ft/{}{}", self.name, self.additional)
        }
    }
}

#[doc(hidden)]
/// Metadata about a config value that is being used in the SDK.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ConfigMetadata {
    config: Cow<'static, str>,
    value: Option<Cow<'static, str>>,
}

impl ConfigMetadata {
    /// Creates `ConfigMetadata`.
    ///
    /// This will result in `InvalidMetadataValue` if the given value isn't alphanumeric or
    /// has characters other than the following:
    /// ```text
    /// !#$%&'*+-.^_`|~
    /// ```
    pub fn new(
        config: impl Into<Cow<'static, str>>,
        value: Option<Cow<'static, str>>,
    ) -> Result<Self, InvalidMetadataValue> {
        Ok(Self {
            config: validate_metadata(config.into())?,
            value: value.map(validate_metadata).transpose()?,
        })
    }
}

impl fmt::Display for ConfigMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // config-metadata = "cfg/" config ["/" value]
        if let Some(value) = &self.value {
            write!(f, "cfg/{}/{}", self.config, value)
        } else {
            write!(f, "cfg/{}", self.config)
        }
    }
}

#[doc(hidden)]
/// Metadata about a software framework that is being used with the SDK.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct FrameworkMetadata {
    name: Cow<'static, str>,
    version: Option<Cow<'static, str>>,
    additional: AdditionalMetadataList,
}

impl FrameworkMetadata {
    /// Creates `FrameworkMetadata`.
    ///
    /// This will result in `InvalidMetadataValue` if the given value isn't alphanumeric or
    /// has characters other than the following:
    /// ```text
    /// !#$%&'*+-.^_`|~
    /// ```
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        version: Option<Cow<'static, str>>,
    ) -> Result<Self, InvalidMetadataValue> {
        Ok(Self {
            name: validate_metadata(name.into())?,
            version: version.map(validate_metadata).transpose()?,
            additional: Default::default(),
        })
    }

    /// Bundles additional arbitrary metadata with this framework metadata.
    pub fn with_additional(mut self, metadata: AdditionalMetadata) -> Self {
        self.additional.push(metadata);
        self
    }
}

impl fmt::Display for FrameworkMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // framework-metadata = "lib/" name ["/" version] *(RWS additional-metadata)
        if let Some(version) = &self.version {
            write!(f, "lib/{}/{}{}", self.name, version, self.additional)
        } else {
            write!(f, "lib/{}{}", self.name, self.additional)
        }
    }
}

#[derive(Clone, Debug)]
struct OsMetadata {
    os_family: &'static OsFamily,
    version: Option<String>,
}

impl fmt::Display for OsMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[derive(Clone, Debug)]
struct LanguageMetadata {
    lang: &'static str,
    version: &'static str,
    extras: AdditionalMetadataList,
}
impl fmt::Display for LanguageMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // language-metadata = "lang/" language "/" version *(RWS additional-metadata)
        write!(f, "lang/{}/{}{}", self.lang, self.version, self.extras)
    }
}

#[derive(Clone, Debug)]
struct ExecEnvMetadata {
    name: String,
}
impl fmt::Display for ExecEnvMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exec-env/{}", &self.name)
    }
}

// TODO(enableNewSmithyRuntimeCleanup): Delete the user agent Tower middleware and consider moving all the remaining code into aws-runtime

/// User agent middleware
#[non_exhaustive]
#[derive(Default, Clone, Debug)]
pub struct UserAgentStage;

impl UserAgentStage {
    /// Creates a new `UserAgentStage`
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
enum UserAgentStageErrorKind {
    /// There was no [`AwsUserAgent`] in the property bag.
    UserAgentMissing,
    /// The formatted user agent string is not a valid HTTP header value. This indicates a bug.
    InvalidHeader(InvalidHeaderValue),
}

/// Failures that can arise from the user agent middleware
#[derive(Debug)]
pub struct UserAgentStageError {
    kind: UserAgentStageErrorKind,
}

impl UserAgentStageError {
    // `pub(crate)` method instead of implementing `From<InvalidHeaderValue>` so that we
    // don't have to expose `InvalidHeaderValue` in public API.
    pub(crate) fn from_invalid_header(value: InvalidHeaderValue) -> Self {
        Self {
            kind: UserAgentStageErrorKind::InvalidHeader(value),
        }
    }
}

impl Error for UserAgentStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use UserAgentStageErrorKind::*;
        match &self.kind {
            InvalidHeader(source) => Some(source as _),
            UserAgentMissing => None,
        }
    }
}

impl fmt::Display for UserAgentStageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UserAgentStageErrorKind::*;
        match self.kind {
            UserAgentMissing => write!(f, "user agent missing from property bag"),
            InvalidHeader(_) => {
                write!(f, "provided user agent header was invalid (this is a bug)")
            }
        }
    }
}

impl From<UserAgentStageErrorKind> for UserAgentStageError {
    fn from(kind: UserAgentStageErrorKind) -> Self {
        Self { kind }
    }
}

#[allow(clippy::declare_interior_mutable_const)] // we will never mutate this
const X_AMZ_USER_AGENT: HeaderName = HeaderName::from_static("x-amz-user-agent");

impl MapRequest for UserAgentStage {
    type Error = UserAgentStageError;

    fn name(&self) -> &'static str {
        "generate_user_agent"
    }

    fn apply(&self, request: Request) -> Result<Request, Self::Error> {
        request.augment(|mut req, conf| {
            let ua = conf
                .get::<AwsUserAgent>()
                .ok_or(UserAgentStageErrorKind::UserAgentMissing)?;
            req.headers_mut().append(
                USER_AGENT,
                HeaderValue::try_from(ua.ua_header())
                    .map_err(UserAgentStageError::from_invalid_header)?,
            );
            req.headers_mut().append(
                X_AMZ_USER_AGENT,
                HeaderValue::try_from(ua.aws_ua_header())
                    .map_err(UserAgentStageError::from_invalid_header)?,
            );
            Ok(req)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::user_agent::{
        AdditionalMetadata, ApiMetadata, AwsUserAgent, ConfigMetadata, FrameworkMetadata,
        UserAgentStage,
    };
    use crate::user_agent::{FeatureMetadata, X_AMZ_USER_AGENT};
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::middleware::MapRequest;
    use aws_smithy_http::operation;
    use aws_types::app_name::AppName;
    use aws_types::build_metadata::OsFamily;
    use aws_types::os_shim_internal::Env;
    use http::header::USER_AGENT;
    use std::borrow::Cow;

    fn make_deterministic(ua: &mut AwsUserAgent) {
        // hard code some variable things for a deterministic test
        ua.sdk_metadata.version = "0.1";
        ua.language_metadata.version = "1.50.0";
        ua.os_metadata.os_family = &OsFamily::Macos;
        ua.os_metadata.version = Some("1.15".to_string());
    }

    #[test]
    fn generate_a_valid_ua() {
        let api_metadata = ApiMetadata {
            service_id: "dynamodb".into(),
            version: "123",
        };
        let mut ua = AwsUserAgent::new_from_environment(Env::from_slice(&[]), api_metadata);
        make_deterministic(&mut ua);
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
    fn generate_a_valid_ua_with_execution_env() {
        let api_metadata = ApiMetadata {
            service_id: "dynamodb".into(),
            version: "123",
        };
        let mut ua = AwsUserAgent::new_from_environment(
            Env::from_slice(&[("AWS_EXECUTION_ENV", "lambda")]),
            api_metadata,
        );
        make_deterministic(&mut ua);
        assert_eq!(
            ua.aws_ua_header(),
            "aws-sdk-rust/0.1 api/dynamodb/123 os/macos/1.15 lang/rust/1.50.0 exec-env/lambda"
        );
        assert_eq!(
            ua.ua_header(),
            "aws-sdk-rust/0.1 os/macos/1.15 lang/rust/1.50.0"
        );
    }

    #[test]
    fn generate_a_valid_ua_with_features() {
        let api_metadata = ApiMetadata {
            service_id: "dynamodb".into(),
            version: "123",
        };
        let mut ua = AwsUserAgent::new_from_environment(Env::from_slice(&[]), api_metadata)
            .with_feature_metadata(
                FeatureMetadata::new("test-feature", Some(Cow::Borrowed("1.0"))).unwrap(),
            )
            .with_feature_metadata(
                FeatureMetadata::new("other-feature", None)
                    .unwrap()
                    .with_additional(AdditionalMetadata::new("asdf").unwrap()),
            );
        make_deterministic(&mut ua);
        assert_eq!(
            ua.aws_ua_header(),
            "aws-sdk-rust/0.1 api/dynamodb/123 os/macos/1.15 lang/rust/1.50.0 ft/test-feature/1.0 ft/other-feature md/asdf"
        );
        assert_eq!(
            ua.ua_header(),
            "aws-sdk-rust/0.1 os/macos/1.15 lang/rust/1.50.0"
        );
    }

    #[test]
    fn generate_a_valid_ua_with_config() {
        let api_metadata = ApiMetadata {
            service_id: "dynamodb".into(),
            version: "123",
        };
        let mut ua = AwsUserAgent::new_from_environment(Env::from_slice(&[]), api_metadata)
            .with_config_metadata(
                ConfigMetadata::new("some-config", Some(Cow::Borrowed("5"))).unwrap(),
            )
            .with_config_metadata(ConfigMetadata::new("other-config", None).unwrap());
        make_deterministic(&mut ua);
        assert_eq!(
            ua.aws_ua_header(),
            "aws-sdk-rust/0.1 api/dynamodb/123 os/macos/1.15 lang/rust/1.50.0 cfg/some-config/5 cfg/other-config"
        );
        assert_eq!(
            ua.ua_header(),
            "aws-sdk-rust/0.1 os/macos/1.15 lang/rust/1.50.0"
        );
    }

    #[test]
    fn generate_a_valid_ua_with_frameworks() {
        let api_metadata = ApiMetadata {
            service_id: "dynamodb".into(),
            version: "123",
        };
        let mut ua = AwsUserAgent::new_from_environment(Env::from_slice(&[]), api_metadata)
            .with_framework_metadata(
                FrameworkMetadata::new("some-framework", Some(Cow::Borrowed("1.3")))
                    .unwrap()
                    .with_additional(AdditionalMetadata::new("something").unwrap()),
            )
            .with_framework_metadata(FrameworkMetadata::new("other", None).unwrap());
        make_deterministic(&mut ua);
        assert_eq!(
            ua.aws_ua_header(),
            "aws-sdk-rust/0.1 api/dynamodb/123 os/macos/1.15 lang/rust/1.50.0 lib/some-framework/1.3 md/something lib/other"
        );
        assert_eq!(
            ua.ua_header(),
            "aws-sdk-rust/0.1 os/macos/1.15 lang/rust/1.50.0"
        );
    }

    #[test]
    fn generate_a_valid_ua_with_app_name() {
        let api_metadata = ApiMetadata {
            service_id: "dynamodb".into(),
            version: "123",
        };
        let mut ua = AwsUserAgent::new_from_environment(Env::from_slice(&[]), api_metadata)
            .with_app_name(AppName::new("my_app").unwrap());
        make_deterministic(&mut ua);
        assert_eq!(
            ua.aws_ua_header(),
            "aws-sdk-rust/0.1 api/dynamodb/123 os/macos/1.15 lang/rust/1.50.0 app/my_app"
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
        req.properties_mut()
            .insert(AwsUserAgent::new_from_environment(
                Env::from_slice(&[]),
                ApiMetadata {
                    service_id: "dynamodb".into(),
                    version: "0.123",
                },
            ));
        let req = stage.apply(req).expect("setting user agent should succeed");
        let (req, _) = req.into_parts();
        req.headers()
            .get(USER_AGENT)
            .expect("UA header should be set");
        req.headers()
            .get(X_AMZ_USER_AGENT)
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
config-metadata      = "cfg/" config ["/" value]
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
