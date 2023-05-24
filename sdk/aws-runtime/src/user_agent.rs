/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_http::user_agent::{ApiMetadata, AwsUserAgent};
use aws_smithy_runtime_api::client::interceptors::error::BoxError;
use aws_smithy_runtime_api::client::interceptors::{Interceptor, InterceptorContext};
use aws_smithy_runtime_api::config_bag::ConfigBag;
use aws_types::app_name::AppName;
use aws_types::os_shim_internal::Env;
use http::header::{InvalidHeaderValue, USER_AGENT};
use http::{HeaderName, HeaderValue};
use std::borrow::Cow;
use std::fmt;

#[allow(clippy::declare_interior_mutable_const)] // we will never mutate this
const X_AMZ_USER_AGENT: HeaderName = HeaderName::from_static("x-amz-user-agent");

#[derive(Debug)]
enum UserAgentInterceptorError {
    MissingApiMetadata,
    InvalidHeaderValue(InvalidHeaderValue),
}

impl std::error::Error for UserAgentInterceptorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidHeaderValue(source) => Some(source),
            Self::MissingApiMetadata => None,
        }
    }
}

impl fmt::Display for UserAgentInterceptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidHeaderValue(_) => "AwsUserAgent generated an invalid HTTP header value. This is a bug. Please file an issue.",
            Self::MissingApiMetadata => "The UserAgentInterceptor requires ApiMetadata to be set before the request is made. This is a bug. Please file an issue.",
        })
    }
}

impl From<InvalidHeaderValue> for UserAgentInterceptorError {
    fn from(err: InvalidHeaderValue) -> Self {
        UserAgentInterceptorError::InvalidHeaderValue(err)
    }
}

/// Generates and attaches the AWS SDK's user agent to a HTTP request
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct UserAgentInterceptor;

impl UserAgentInterceptor {
    /// Creates a new `UserAgentInterceptor`
    pub fn new() -> Self {
        UserAgentInterceptor
    }
}

fn header_values(
    ua: &AwsUserAgent,
) -> Result<(HeaderValue, HeaderValue), UserAgentInterceptorError> {
    // Pay attention to the extremely subtle difference between ua_header and aws_ua_header below...
    Ok((
        HeaderValue::try_from(ua.ua_header())?,
        HeaderValue::try_from(ua.aws_ua_header())?,
    ))
}

impl Interceptor for UserAgentInterceptor {
    fn modify_before_signing(
        &self,
        context: &mut InterceptorContext,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let api_metadata = cfg
            .get::<ApiMetadata>()
            .ok_or(UserAgentInterceptorError::MissingApiMetadata)?;

        // Allow for overriding the user agent by an earlier interceptor (so, for example,
        // tests can use `AwsUserAgent::for_tests()`) by attempting to grab one out of the
        // config bag before creating one.
        let ua: Cow<'_, AwsUserAgent> = cfg
            .get::<AwsUserAgent>()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| {
                let mut ua = AwsUserAgent::new_from_environment(Env::real(), api_metadata.clone());

                let maybe_app_name = cfg.get::<AppName>();
                if let Some(app_name) = maybe_app_name {
                    ua.set_app_name(app_name.clone());
                }
                Cow::Owned(ua)
            });

        let headers = context.request_mut()?.headers_mut();
        let (user_agent, x_amz_user_agent) = header_values(&ua)?;
        headers.append(USER_AGENT, user_agent);
        headers.append(X_AMZ_USER_AGENT, x_amz_user_agent);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_runtime_api::client::interceptors::{Interceptor, InterceptorContext};
    use aws_smithy_runtime_api::config_bag::ConfigBag;
    use aws_smithy_runtime_api::type_erasure::TypedBox;
    use aws_smithy_types::error::display::DisplayErrorContext;

    fn expect_header<'a>(context: &'a InterceptorContext, header_name: &str) -> &'a str {
        context
            .request()
            .unwrap()
            .headers()
            .get(header_name)
            .unwrap()
            .to_str()
            .unwrap()
    }

    #[test]
    fn test_overridden_ua() {
        let mut context = InterceptorContext::new(TypedBox::new("doesntmatter").erase());
        context.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());

        let mut config = ConfigBag::base();
        config.put(AwsUserAgent::for_tests());
        config.put(ApiMetadata::new("unused", "unused"));

        let interceptor = UserAgentInterceptor::new();
        interceptor
            .modify_before_signing(&mut context, &mut config)
            .unwrap();

        let header = expect_header(&context, "user-agent");
        assert_eq!(AwsUserAgent::for_tests().ua_header(), header);
        assert!(!header.contains("unused"));

        assert_eq!(
            AwsUserAgent::for_tests().aws_ua_header(),
            expect_header(&context, "x-amz-user-agent")
        );
    }

    #[test]
    fn test_default_ua() {
        let mut context = InterceptorContext::new(TypedBox::new("doesntmatter").erase());
        context.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());

        let api_metadata = ApiMetadata::new("some-service", "some-version");
        let mut config = ConfigBag::base();
        config.put(api_metadata.clone());

        let interceptor = UserAgentInterceptor::new();
        interceptor
            .modify_before_signing(&mut context, &mut config)
            .unwrap();

        let expected_ua = AwsUserAgent::new_from_environment(Env::real(), api_metadata);
        assert!(
            expected_ua.aws_ua_header().contains("some-service"),
            "precondition"
        );
        assert_eq!(
            expected_ua.ua_header(),
            expect_header(&context, "user-agent")
        );
        assert_eq!(
            expected_ua.aws_ua_header(),
            expect_header(&context, "x-amz-user-agent")
        );
    }

    #[test]
    fn test_app_name() {
        let mut context = InterceptorContext::new(TypedBox::new("doesntmatter").erase());
        context.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());

        let api_metadata = ApiMetadata::new("some-service", "some-version");
        let mut config = ConfigBag::base();
        config.put(api_metadata.clone());
        config.put(AppName::new("my_awesome_app").unwrap());

        let interceptor = UserAgentInterceptor::new();
        interceptor
            .modify_before_signing(&mut context, &mut config)
            .unwrap();

        let app_value = "app/my_awesome_app";
        let header = expect_header(&context, "user-agent");
        assert!(
            !header.contains(app_value),
            "expected `{header}` to not contain `{app_value}`"
        );

        let header = expect_header(&context, "x-amz-user-agent");
        assert!(
            header.contains(app_value),
            "expected `{header}` to contain `{app_value}`"
        );
    }

    #[test]
    fn test_api_metadata_missing() {
        let mut context = InterceptorContext::new(TypedBox::new("doesntmatter").erase());
        context.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());

        let mut config = ConfigBag::base();

        let interceptor = UserAgentInterceptor::new();
        let error = format!(
            "{}",
            DisplayErrorContext(
                &*interceptor
                    .modify_before_signing(&mut context, &mut config)
                    .expect_err("it should error")
            )
        );
        assert!(
            error.contains("This is a bug"),
            "`{error}` should contain message `This is a bug`"
        );
    }
}
