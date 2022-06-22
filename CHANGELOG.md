<!-- Do not manually edit this file, use `update-changelogs` -->
v0.14.0 (June 22nd, 2022)
=========================
**New this release:**
- 🐛 ([aws-sdk-rust#547](https://github.com/awslabs/aws-sdk-rust/issues/547), [smithy-rs#1458](https://github.com/awslabs/smithy-rs/issues/1458)) Fix bug in profile file credential provider where a missing `default` profile lead to an unintended error.
- ([smithy-rs#1421](https://github.com/awslabs/smithy-rs/issues/1421)) Add `Debug` implementation to several types in `aws-config`
- 🐛 ([aws-sdk-rust#558](https://github.com/awslabs/aws-sdk-rust/issues/558), [smithy-rs#1478](https://github.com/awslabs/smithy-rs/issues/1478)) Fix bug in retry policy where user configured timeouts were not retried. With this fix, setting
    [`with_call_attempt_timeout`](https://docs.rs/aws-smithy-types/0.43.0/aws_smithy_types/timeout/struct.Api.html#method.with_call_attempt_timeout)
    will lead to a retry when retries are enabled.


v0.13.0 (June 9th, 2022)
========================
**New this release:**
- 🎉 ([smithy-rs#1390](https://github.com/awslabs/smithy-rs/issues/1390)) Add method `ByteStream::into_async_read`. This makes it easy to convert `ByteStream`s into a struct implementing `tokio:io::AsyncRead`. Available on **crate feature** `rt-tokio` only.
- 🎉 ([smithy-rs#1356](https://github.com/awslabs/smithy-rs/issues/1356), @jszwedko) Add support for `credential_process` in AWS configs for fetching credentials from an external process.
- ([smithy-rs#1404](https://github.com/awslabs/smithy-rs/issues/1404), @petrosagg) Switch to [RustCrypto](https://github.com/RustCrypto)'s implementation of MD5.

**Contributors**
Thank you for your contributions! ❤
- @jszwedko ([smithy-rs#1356](https://github.com/awslabs/smithy-rs/issues/1356))
- @petrosagg ([smithy-rs#1404](https://github.com/awslabs/smithy-rs/issues/1404))

v0.12.0 (May 13th, 2022)
========================
**New this release:**
- ([smithy-rs#1352](https://github.com/awslabs/smithy-rs/issues/1352)) Log a debug event when a retry is going to be peformed


0.11.0 (April 28th, 2022)
=========================
**Breaking Changes:**
- ⚠ ([smithy-rs#1318](https://github.com/awslabs/smithy-rs/issues/1318)) Bump [MSRV](https://github.com/awslabs/aws-sdk-rust#supported-rust-versions-msrv) from 1.56.1 to 1.58.1 per our "two versions behind" policy.

**New this release:**
- 🐛 ([smithy-rs#1344](https://github.com/awslabs/smithy-rs/issues/1344), @ryansb) Suppress irrelevant `$HOME` expansion warning when running in a Lambda Extension

**Contributors**
Thank you for your contributions! ❤
- @ryansb ([smithy-rs#1344](https://github.com/awslabs/smithy-rs/issues/1344))

0.10.1 (April 14th, 2022)
=========================

**Breaking Changes:**
- ⚠ ([aws-sdk-rust#490](https://github.com/awslabs/aws-sdk-rust/issues/490)) Update all SDK and runtime crates to [edition 2021](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html)

**New this release:**
- ([smithy-rs#1262](https://github.com/awslabs/smithy-rs/issues/1262), @liubin) Fix link to Developer Guide in crate's README.md
- 🐛 ([aws-sdk-rust#1271](https://github.com/awslabs/aws-sdk-rust/issues/1271), @elrob) Treat blank environment variable credentials (`AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`) as missing instead of attempting to use them to sign requests.
- ([aws-sdk-rust#479](https://github.com/awslabs/aws-sdk-rust/issues/479), [smithy-rs#1296](https://github.com/awslabs/smithy-rs/issues/1296)) Add support for configuring the session length in [AssumeRoleProvider](https://docs.rs/aws-config/latest/aws_config/sts/struct.AssumeRoleProvider.html)
- ([smithy-rs#1296](https://github.com/awslabs/smithy-rs/issues/1296)) Add caching to [AssumeRoleProvider](https://docs.rs/aws-config/latest/aws_config/sts/struct.AssumeRoleProvider.html)
- ([smithy-rs#1300](https://github.com/awslabs/smithy-rs/issues/1300), @benesch) Add endpoint resolver to SdkConfig. This enables overriding the endpoint resolver for all services build from a single SdkConfig.

**Contributors**
Thank you for your contributions! ❤
- @benesch ([smithy-rs#1300](https://github.com/awslabs/smithy-rs/issues/1300))
- @elrob ([aws-sdk-rust#1271](https://github.com/awslabs/aws-sdk-rust/issues/1271))
- @liubin ([smithy-rs#1262](https://github.com/awslabs/smithy-rs/issues/1262))

0.9.0 (March 17, 2022)
======================
**Breaking Changes:**
- ⚠ ([aws-sdk-rust#406](https://github.com/awslabs/aws-sdk-rust/issues/406)) `aws_types::config::Config` has been renamed to `aws_types::sdk_config::SdkConfig`. This is to better differentiate it
    from service-specific configs like `aws_sdk_s3::Config`. If you were creating shared configs with
    `aws_config::load_from_env()`, then you don't have to do anything. If you were directly referring to a shared config,
    update your `use` statements and `struct` names.

    _Before:_
    ```rust
    use aws_types::config::Config;

    fn main() {
        let config = Config::builder()
        // config builder methods...
        .build()
        .await;
    }
    ```

    _After:_
    ```rust
    // We re-export this type from the root module so it's easier to reference
    use aws_types::SdkConfig;

    fn main() {
        let config = SdkConfig::builder()
        // config builder methods...
        .build()
        .await;
    }
    ```
- ⚠ ([smithy-rs#724](https://github.com/awslabs/smithy-rs/issues/724)) Timeout configuration has been refactored a bit. If you were setting timeouts through environment variables or an AWS
    profile, then you shouldn't need to change anything. Take note, however, that we don't currently support HTTP connect,
    read, write, or TLS negotiation timeouts. If you try to set any of those timeouts in your profile or environment, we'll
    log a warning explaining that those timeouts don't currently do anything.

    If you were using timeouts programmatically,
    you'll need to update your code. In previous versions, timeout configuration was stored in a single `TimeoutConfig`
    struct. In this new version, timeouts have been broken up into several different config structs that are then collected
    in a `timeout::Config` struct. As an example, to get the API per-attempt timeout in previous versions you would access
    it with `<your TimeoutConfig>.api_call_attempt_timeout()` and in this new version you would access it with
    `<your timeout::Config>.api.call_attempt_timeout()`. We also made some unimplemented timeouts inaccessible in order to
    avoid giving users the impression that setting them had an effect. We plan to re-introduce them once they're made
    functional in a future update.

**New this release:**
- 🎉 ([aws-sdk-rust#475](https://github.com/awslabs/aws-sdk-rust/issues/475), [aws-sdk-rust#473](https://github.com/awslabs/aws-sdk-rust/issues/473)) Enable presigning for S3 operations UploadPart and DeleteObject


0.8.0 (Februrary 24, 2022)
==========================
**Breaking Changes:**
- ⚠ ([smithy-rs#1216](https://github.com/awslabs/smithy-rs/issues/1216)) `aws-sigv4` no longer skips the `content-length` and `content-type` headers when signing with `SignatureLocation::QueryParams`

**New this release:**
- 🎉 ([smithy-rs#1220](https://github.com/awslabs/smithy-rs/issues/1220), [aws-sdk-rust#462](https://github.com/awslabs/aws-sdk-rust/issues/462)) Made it possible to change settings, such as load timeout, on the credential cache used by the `DefaultCredentialsChain`.
- 🐛 ([smithy-rs#1197](https://github.com/awslabs/smithy-rs/issues/1197)) Fixed a bug that caused clients to eventually stop retrying. The cross-request retry allowance wasn't being reimbursed upon receiving a successful response, so once this allowance reached zero, no further retries would ever be attempted.
- 🐛 ([smithy-rs#1217](https://github.com/awslabs/smithy-rs/issues/1217), [aws-sdk-rust#467](https://github.com/awslabs/aws-sdk-rust/issues/467)) `ClientBuilder` helpers `rustls()` and `native_tls()` now return `DynConnector` so that they once again work when constructing clients with custom middleware in the SDK.
- 🐛 ([smithy-rs#1216](https://github.com/awslabs/smithy-rs/issues/1216), [aws-sdk-rust#466](https://github.com/awslabs/aws-sdk-rust/issues/466)) Fixed a bug in S3 that prevented the `content-length` and `content-type` inputs from being included in a presigned request signature. With this fix, customers can generate presigned URLs that enforce `content-length` and `content-type` for requests to S3.


0.7.0 (February 18th, 2022)
===========================
**Breaking Changes:**
- ⚠ ([smithy-rs#1144](https://github.com/awslabs/smithy-rs/issues/1144)) The `aws_config::http_provider` module has been renamed to `aws_config::http_credential_provider` to better reflect its purpose.
- ⚠ ([smithy-rs#1144](https://github.com/awslabs/smithy-rs/issues/1144)) Some APIs required that timeout configuration be specified with an `aws_smithy_client::timeout::Settings` struct while
    others required an `aws_smithy_types::timeout::TimeoutConfig` struct. Both were equivalent. Now `aws_smithy_types::timeout::TimeoutConfig`
    is used everywhere and `aws_smithy_client::timeout::Settings` has been removed. Here's how to migrate code your code that
    depended on `timeout::Settings`:

    The old way:
    ```rust
    let timeout = timeout::Settings::new()
        .with_connect_timeout(Duration::from_secs(1))
        .with_read_timeout(Duration::from_secs(2));
    ```

    The new way:
    ```rust
    // This example is passing values, so they're wrapped in `Option::Some`. You can disable a timeout by passing `None`.
    let timeout = TimeoutConfig::new()
        .with_connect_timeout(Some(Duration::from_secs(1)))
        .with_read_timeout(Some(Duration::from_secs(2)));
    ```
- ⚠ ([smithy-rs#1144](https://github.com/awslabs/smithy-rs/issues/1144)) `MakeConnectorFn`, `HttpConnector`, and `HttpSettings` have been moved from `aws_config::provider_config` to
    `aws_smithy_client::http_connector`. This is in preparation for a later update that will change how connectors are
    created and configured.

    If you were using these structs/enums, you can migrate your old code by importing them from their new location.
- ⚠ ([smithy-rs#1144](https://github.com/awslabs/smithy-rs/issues/1144)) Along with moving `HttpConnector` to `aws_smithy_client`, the `HttpConnector::make_connector` method has been renamed to
    `HttpConnector::connector`.

    If you were using this method, you can migrate your old code by calling `connector` instead of `make_connector`.
- ⚠ ([smithy-rs#1085](https://github.com/awslabs/smithy-rs/issues/1085)) Moved the following re-exports into a `types` module for all services:
    - `aws_sdk_<service>::AggregatedBytes` -> `aws_sdk_<service>::types::AggregatedBytes`
    - `aws_sdk_<service>::Blob` -> `aws_sdk_<service>::types::Blob`
    - `aws_sdk_<service>::ByteStream` -> `aws_sdk_<service>::types::ByteStream`
    - `aws_sdk_<service>::DateTime` -> `aws_sdk_<service>::types::DateTime`
    - `aws_sdk_<service>::SdkError` -> `aws_sdk_<service>::types::SdkError`
- ⚠ ([smithy-rs#1085](https://github.com/awslabs/smithy-rs/issues/1085)) `AggregatedBytes` and `ByteStream` are now only re-exported if the service has streaming operations,
    and `Blob`/`DateTime` are only re-exported if the service uses them.
- ⚠ ([smithy-rs#1130](https://github.com/awslabs/smithy-rs/issues/1130)) MSRV increased from `1.54` to `1.56.1` per our 2-behind MSRV policy.
- ⚠ ([smithy-rs#1132](https://github.com/awslabs/smithy-rs/issues/1132)) Fluent clients for all services no longer have generics, and now use `DynConnector` and `DynMiddleware` to allow
    for connector/middleware customization. This should only break references to the client that specified generic types for it.

    If you customized the AWS client's connector or middleware with something like the following:
    ```rust
    let client = aws_sdk_s3::Client::with_config(
        aws_sdk_s3::client::Builder::new()
            .connector(my_custom_connector) // Connector customization
            .middleware(my_custom_middleware) // Middleware customization
            .default_async_sleep()
            .build(),
        config
    );
    ```
    Then you will need to wrap the custom connector or middleware in
    [`DynConnector`](https://docs.rs/aws-smithy-client/0.36.0/aws_smithy_client/erase/struct.DynConnector.html)
    and
    [`DynMiddleware`](https://docs.rs/aws-smithy-client/0.36.0/aws_smithy_client/erase/struct.DynMiddleware.html)
    respectively:
    ```rust
    let client = aws_sdk_s3::Client::with_config(
        aws_sdk_s3::client::Builder::new()
            .connector(DynConnector::new(my_custom_connector)) // Now with `DynConnector`
            .middleware(DynMiddleware::new(my_custom_middleware)) // Now with `DynMiddleware`
            .default_async_sleep()
            .build(),
        config
    );
    ```

    If you had functions that took a generic connector, such as the following:
    ```rust
    fn some_function<C, E>(conn: C) -> Result<()>
    where
        C: aws_smithy_client::bounds::SmithyConnector<Error = E> + Send + 'static,
        E: Into<aws_smithy_http::result::ConnectorError>
    {
        // ...
    }
    ```

    Then the generics and trait bounds will no longer be necessary:
    ```rust
    fn some_function(conn: DynConnector) -> Result<()> {
        // ...
    }
    ```

    Similarly, functions that took a generic middleware can replace the generic with `DynMiddleware` and
    remove their trait bounds.

**New this release:**
- 🐛 ([aws-sdk-rust#443](https://github.com/awslabs/aws-sdk-rust/issues/443)) The `ProfileFileRegionProvider` will now respect regions set in chained profiles
- ([smithy-rs#1144](https://github.com/awslabs/smithy-rs/issues/1144)) Several modules defined in the `aws_config` crate that used to be declared within another module's file have been moved to their own files. The moved modules are `sts`, `connector`, and `default_providers`. They still have the exact same import paths.
- 🐛 ([smithy-rs#1129](https://github.com/awslabs/smithy-rs/issues/1129)) Fix some docs links not working because they were escaped when they shouldn't have been
- ([smithy-rs#1085](https://github.com/awslabs/smithy-rs/issues/1085)) The `Client` and `Config` re-exports now have their documentation inlined in the service docs
- 🐛 ([smithy-rs#1180](https://github.com/awslabs/smithy-rs/issues/1180)) Fixed example showing how to use hardcoded credentials in `aws-types`


0.6.0 (January 26, 2022)
========================
**New this release:**
- ([aws-sdk-rust#423](https://github.com/awslabs/aws-sdk-rust/issues/423)) Added `impl Into<http::request::Builder> for PresignedRequest` and a conversion method for turning `PresignedRequest`s into `http::Request`s.
- ([smithy-rs#1087](https://github.com/awslabs/smithy-rs/issues/1087)) Convert several `info` spans to `debug` in aws-config
- ([smithy-rs#1118](https://github.com/awslabs/smithy-rs/issues/1118)) SDK examples now come from [`awsdocs/aws-doc-sdk-examples`](https://github.com/awsdocs/aws-doc-sdk-examples) rather than from `smithy-rs`


0.5.2 (January 20th, 2022)
==========================

**New this release:**
- 🐛 ([smithy-rs#1100](https://github.com/awslabs/smithy-rs/issues/1100)) _Internal:_ Update sync script to run gradle clean. This fixes an issue where codegen was not triggered when only properties changed.


v0.5.1 (January 19th, 2022)
===========================

**New this release:**
- 🐛 ([smithy-rs#1089](https://github.com/awslabs/smithy-rs/issues/1089)) Fix dev-dependency cycle between aws-sdk-sso and aws-config


0.5.0 (January 19, 2022)
========================
**New this release:**
- 🎉 ([aws-sdk-rust#348](https://github.com/awslabs/aws-sdk-rust/issues/348)) The docs for fluent builders now have easy links to their corresponding Input, Output, and Error structs
- 🎉 ([smithy-rs#1051](https://github.com/awslabs/smithy-rs/issues/1051), [aws-sdk-rust#4](https://github.com/awslabs/aws-sdk-rust/issues/4)) Add support for SSO credentials
- 🐛🎉 ([smithy-rs#1065](https://github.com/awslabs/smithy-rs/issues/1065), [aws-sdk-rust#398](https://github.com/awslabs/aws-sdk-rust/issues/398), @nmoutschen) Silence profile credential warnings in Lambda environment
- 🐛 ([aws-sdk-rust#405](https://github.com/awslabs/aws-sdk-rust/issues/405), [smithy-rs#1083](https://github.com/awslabs/smithy-rs/issues/1083)) Fixed paginator bug impacting EC2 describe VPCs (and others)

**Contributors**
Thank you for your contributions! ❤
- @nmoutschen ([aws-sdk-rust#398](https://github.com/awslabs/aws-sdk-rust/issues/398), [smithy-rs#1065](https://github.com/awslabs/smithy-rs/issues/1065))


v0.4.1 (January 10, 2022)
=========================
**New this release:**
- 🐛 (smithy-rs#1050, @nmoutschen) Fix typos for X-Ray trace ID environment variable in aws_http::recursion_detection
- 🐛 (smithy-rs#1054, aws-sdk-rust#391) Fix critical paginator bug where an empty outputToken lead to a never ending stream.

**Contributors**
Thank you for your contributions! ❤
- @nmoutschen (smithy-rs#1050)


0.4.0 (January 6th, 2022)
=========================
**Breaking Changes:**
- ⚠ (smithy-rs#990) Codegen will no longer produce builders and clients with methods that take `impl Into<T>` except for strings and boxed types.
- ⚠ (smithy-rs#961) The `meta`, `environment`, and `dns` Cargo feature flags were removed from `aws-config`.
    The code behind the `dns` flag is now enabled when `rt-tokio` is enabled. The code behind
    the `meta` and `environment` flags is always enabled now.
- ⚠ (smithy-rs#1003) `aws_http::AwsErrorRetryPolicy` was moved to `aws_http::retry::AwsErrorRetryPolicy`.
- ⚠ (smithy-rs#1017, smithy-rs#930) Simplify features in aws-config. All features have been removed from `aws-config` with the exception of: `rt-tokio`, `rustls` and `native-tls`. All other features are now included by default. If you depended on those features specifically, remove them from your features listing.

**New this release:**
- 🎉 (aws-sdk-rust#47, smithy-rs#1006) Add support for paginators! Paginated APIs now include `.into_paginator()` and (when supported) `.into_paginator().items()` to enable paginating responses automatically. The paginator API should be considered in preview and is subject to change pending customer feedback.
- (smithy-rs#712) We removed an example 'telephone-game' that was problematic for our CI.
    The APIs that that example demonstrated are also demonstrated by our Polly
    and TranscribeStreaming examples so please check those out if you miss it.
- 🐛 (aws-sdk-rust#357) Generated docs should no longer contain links that don't go anywhere
- (aws-sdk-rust#254, @jacco) Made fluent operation structs cloneable
- (smithy-rs#973) Debug implementation of Credentials will print `expiry` in a human readable way.
- 🐛 (smithy-rs#999, smithy-rs#143, aws-sdk-rust#344) Add Route53 customization to trim `/hostedzone/` prefix prior to serialization. This fixes a bug where round-tripping a hosted zone id resulted in an error.
- 🐛 (smithy-rs#998, aws-sdk-rust#359) Fix bug where ECS credential provider could not perform retries.
- (smithy-rs#1003) Add recursion detection middleware to the default stack
- (smithy-rs#1002, aws-sdk-rust#352) aws_types::Config is now `Clone`
- (smithy-rs#670, @jacco) Example for Config builder region function added
- (smithy-rs#1021, @kiiadi) Add function to `aws_config::profile::ProfileSet` that allows listing of loaded profiles by name.
- 🐛 (smithy-rs#1046, aws-sdk-rust#384) Fix IMDS credentials provider bug where the instance profile name was incorrectly cached.

**Contributors**
Thank you for your contributions! ❤
- @jacco (aws-sdk-rust#254, smithy-rs#670)
- @kiiadi (smithy-rs#1021)


v0.3.0 (December 15th, 2021)
============================
**Breaking Changes:**
- ⚠ (smithy-rs#930) If you directly depend on AWS or Smithy runtime crates _(e.g., AWS crates not named `aws-config` or prefixed with `aws-sdk-`)_,
    the formerly default features from those crates must now be explicitly set in your `Cargo.toml`.

    **Upgrade guide**

    | before                          | after                                                                                            |
    | ------------------------------- | ------------------------------------------------------------------------------------------------ |
    | `aws-smithy-async = "VERSION"`  | `aws-smithy-async = { version = "VERSION", features = ["rt-tokio"] }`                            |
    | `aws-smithy-client = "VERSION"` | `aws-smithy-client = { version = "VERSION", features = ["client-hyper", "rustls", "rt-tokio"] }` |
    | `aws-smithy-http = "VERSION"`   | `aws-smithy-http = { version = "VERSION", features = ["rt-tokio"] }`                             |
- ⚠ (smithy-rs#940) `aws_hyper::Client` which was just a re-export of `aws_smithy_types::Client` with generics set has been removed. If you used
    `aws_hyper::Client` or `aws_hyper::Client::https()` you can update your code to use `aws_smithy_client::Builder::https()`.
- ⚠ (smithy-rs#947) The features `aws-hyper/rustls` and `aws-hyper/native-tls` have been removed. If you were using these, use the identical features on `aws-smithy-client`.
- ⚠ (smithy-rs#959, smithy-rs#934) `aws-hyper::AwsMiddleware` is now generated into generated service clients directly. If you used `aws_hyper::Middleware`, use <service>::middleware::DefaultMiddleware` instead.

**New this release:**
- 🐛 (aws-sdk-rust#330) A bug that occurred when signing certain query strings has been fixed
- 🐛 (smithy-rs#949, @a-xp) Fix incorrect argument order in the builder for `LazyCachingCredentialsProvider`
- 🐛 (aws-sdk-rust#304) `aws-config` will now work as intended for users that want to use `native-tls` instead of `rustls`. Previously, it was
    difficult to ensure that `rustls` was not in use. Also, there is now an example of how to use `native-tls` and a test
    that ensures `rustls` is not in the dependency tree
- 🐛 (aws-sdk-rust#317, smithy-rs#907) Removed inaccurate log message when a client was used without a sleep implementation, and
    improved context and call to action in logged messages around missing sleep implementations.
- (smithy-rs#923) Use provided `sleep_impl` for retries instead of using Tokio directly.
- (smithy-rs#920) Fix typos in module documentation for generated crates
- 🐛 (aws-sdk-rust#301, smithy-rs#892) Avoid serializing repetitive `xmlns` attributes when serializing XML. This reduces the length of serialized requests and should improve compatibility with localstack.
- 🐛 (smithy-rs#953, aws-sdk-rust#331) Fixed a bug where certain characters caused a panic during URI encoding.

**Contributors**
Thank you for your contributions! ❤
- @a-xp (smithy-rs#949)


v0.2.0 (December 2nd, 2021)
===========================

- This release was a version bump to fix a version number conflict in crates.io

v0.1.0 (December 2nd, 2021)
===========================

**New this release**
- Add docs.rs metadata section to all crates to document all features
- [Added a new example showing how to set all currently supported timeouts](./examples/setting_timeouts/src/main.rs)
- Add a new check so that the SDK doesn't emit an irrelevant `$HOME` dir warning when running in a Lambda (aws-sdk-rust#307)
- :bug: Don't capture empty session tokens from the `AWS_SESSION_TOKEN` environment variable (aws-sdk-rust#316, smithy-rs#906)

v0.0.26-alpha (November 23rd, 2021)
===================================

**Breaking Changes**
- `RetryConfigBuilder::merge_with` has been renamed to `RetryConfigBuilder::take_unset_from`
- `Credentials::from_keys` is now behind a feature flag named `hardcoded-credentials` in `aws-types`.
  It is __NOT__ secure to hardcode credentials into your application, and the credentials
  providers that come with the AWS SDK should be preferred. (smithy-rs#875, smithy-rs#317)
- (aws-smithy-client): Extraneous `pub use SdkSuccess` removed from `aws_smithy_client::hyper_ext`. (smithy-rs#855)
- The `add_metadata` function was removed from `AwsUserAgent` in `aws-http`.
  Use `with_feature_metadata`, `with_config_metadata`, or `with_framework_metadata` now instead. (smithy-rs#865)
- Several breaking changes around `aws_smithy_types::Instant` were introduced by smithy-rs#849:
  - `aws_smithy_types::Instant` from was renamed to `DateTime` to avoid confusion with the standard library's monotonically nondecreasing `Instant` type.
  - `DateParseError` in `aws_smithy_types` has been renamed to `DateTimeParseError` to match the type that's being parsed.
  - The `chrono-conversions` feature and associated functions have been moved to the `aws-smithy-types-convert` crate.
    - Calls to `Instant::from_chrono` should be changed to:
      ```rust
      use aws_smithy_types::DateTime;
      use aws_smithy_types_convert::date_time::DateTimeExt;

      // For chrono::DateTime<Utc>
      let date_time = DateTime::from_chrono_utc(chrono_date_time);
      // For chrono::DateTime<FixedOffset>
      let date_time = DateTime::from_chrono_offset(chrono_date_time);
      ```
    - Calls to `instant.to_chrono()` should be changed to:
      ```rust
      use aws_smithy_types_convert::date_time::DateTimeExt;

      date_time.to_chrono_utc();
      ```
  - `Instant::from_system_time` and `Instant::to_system_time` have been changed to `From` trait implementations.
    - Calls to `from_system_time` should be changed to:
      ```rust
      DateTime::from(system_time);
      // or
      let date_time: DateTime = system_time.into();
      ```
    - Calls to `to_system_time` should be changed to:
      ```rust
      SystemTime::from(date_time);
      // or
      let system_time: SystemTime = date_time.into();
      ```
  - Several functions in `Instant`/`DateTime` were renamed:
    - `Instant::from_f64` -> `DateTime::from_secs_f64`
    - `Instant::from_fractional_seconds` -> `DateTime::from_fractional_secs`
    - `Instant::from_epoch_seconds` -> `DateTime::from_secs`
    - `Instant::from_epoch_millis` -> `DateTime::from_millis`
    - `Instant::epoch_fractional_seconds` -> `DateTime::as_secs_f64`
    - `Instant::has_nanos` -> `DateTime::has_subsec_nanos`
    - `Instant::epoch_seconds` -> `DateTime::secs`
    - `Instant::epoch_subsecond_nanos` -> `DateTime::subsec_nanos`
    - `Instant::to_epoch_millis` -> `DateTime::to_millis`
  - The `DateTime::fmt` method is now fallible and fails when a `DateTime`'s value is outside what can be represented by the desired date format.

**New this week**

- :warning: MSRV increased from 1.53.0 to 1.54.0 per our 3-behind MSRV policy.
- Conversions from `aws_smithy_types::DateTime` to `OffsetDateTime` from the `time` crate are now available from the `aws-smithy-types-convert` crate. (smithy-rs#849)
- Fixed links to Usage Examples (smithy-rs#862, @floric)
- Added missing features to user agent formatting, and made it possible to configure an app name for the user agent via service config. (smithy-rs#865)
- :bug: Relaxed profile name validation to allow `@` and other characters (smithy-rs#861, aws-sdk-rust#270)
- :bug: Fixed signing problem with S3 Control (smithy-rs#858, aws-sd-rust#291)
- :tada: Timeouts for requests are now configurable. You can set a timeout for each individual request attempt or for all attempts made for a request. (smithy-rs#831)
  - `SdkError` now includes a variant `TimeoutError` for when a request times out  (smithy-rs#885)
- Improve docs on `aws-smithy-client` (smithy-rs#855)
- Fix http-body dependency version (smithy-rs#883, aws-sdk-rust#305)

**Contributions**

Thank you for your contributions! :heart:

- @floric (smithy-rs#862)

v0.0.25-alpha (November 11th, 2021)
===================================

No changes since last release except for version bumping since older versions
of the AWS SDK were failing to compile with the `0.27.0-alpha.2` version chosen
for some of the supporting crates.

v0.0.24-alpha (November 9th, 2021)
==================================
**Breaking Changes**
- Members named `builder` on model structs were renamed to `builder_value` so that their accessors don't conflict with the existing `builder()` methods (smithy-rs#842)

**New this week**
- Fix epoch seconds date-time parsing bug in `aws-smithy-types` (smithy-rs#834)
- Omit trailing zeros from fraction when formatting HTTP dates in `aws-smithy-types` (smithy-rs#834)
- Moved examples into repository root (aws-sdk-rust#181, smithy-rs#843)
- Model structs now have accessor methods for their members. We recommend updating code to use accessors instead of public fields. A future release will deprecate the public fields before they are made private. (smithy-rs#842)
- :bug: Fix bug that caused signing to fail for requests where the body length was <=9. (smithy-rs#845)

v0.0.23-alpha (November 3rd, 2021)
==================================
**New this week**
- :tada: Add support for AWS Glacier (smithy-rs#801)
- :tada: Add support for AWS Panorama
- :bug: Fix `native-tls` feature in `aws-config` (aws-sdk-rust#265, smithy-rs#803)
- Add example to aws-sig-auth for generating an IAM Token for RDS (smithy-rs#811, aws-sdk-rust#147)
- :bug: `hyper::Error(IncompleteMessage)` will now be retried (smithy-rs#815)
- :bug: S3 request metadata signing now correctly trims headers fixing [problems like this](https://github.com/awslabs/aws-sdk-rust/issues/248) (smithy-rs#761)
- All unions (eg. `dynamodb::model::AttributeValue`) now include an additional `Unknown` variant. These support cases where a new union variant has been added on the server but the client has not been updated.
- :bug: Fix generated docs on unions like `dynamodb::AttributeValue`. (smithy-rs#826)

**Breaking Changes**
- `<operation>.make_operation(&config)` is now an `async` function for all operations. Code should be updated to call `.await`. This will only impact users using the low-level API. (smithy-rs#797)

v0.0.22-alpha (October 20th, 2021)
==================================

**Breaking Changes**

- `CredentialsError` variants became non-exhaustive. This makes them impossible to construct directly outside of the `aws_types` crate. In order to construct credentials errors, new methods have been added for each variant. Instead of `CredentialsError::Unhandled(...)`, you should instead use `CredentialsError::unhandled`. Matching methods exist for all variants. (#781)
- The default credentials chain now returns `CredentialsError::CredentialsNotLoaded` instead of `ProviderError` when no credentials providers are configured.
- :warning: All Smithy runtime crates have been renamed to have an `aws-` prefix. This may require code changes:
  - _Cargo.toml_ changes:
    - `smithy-async` -> `aws-smithy-async`
    - `smithy-client` -> `aws-smithy-client`
    - `smithy-eventstream` -> `aws-smithy-eventstream`
    - `smithy-http` -> `aws-smithy-http`
    - `smithy-http-tower` -> `aws-smithy-http-tower`
    - `smithy-json` -> `aws-smithy-json`
    - `smithy-protocol-test` -> `aws-smithy-protocol-test`
    - `smithy-query` -> `aws-smithy-query`
    - `smithy-types` -> `aws-smithy-types`
    - `smithy-xml` -> `aws-smithy-xml`
  - Rust `use` statement changes:
    - `smithy_async` -> `aws_smithy_async`
    - `smithy_client` -> `aws_smithy_client`
    - `smithy_eventstream` -> `aws_smithy_eventstream`
    - `smithy_http` -> `aws_smithy_http`
    - `smithy_http_tower` -> `aws_smithy_http_tower`
    - `smithy_json` -> `aws_smithy_json`
    - `smithy_protocol_test` -> `aws_smithy_protocol_test`
    - `smithy_query` -> `aws_smithy_query`
    - `smithy_types` -> `aws_smithy_types`
    - `smithy_xml` -> `aws_smithy_xml`

**New this week**

- Moved the contents of `aws-auth` into the `aws-http` runtime crate (smithy-rs#783)
- Fix instances where docs were missing in generated services and add `#[warn_missing_docs]` (smithy-rs#779)
- Add tracing output for resolved AWS endpoint (smithy-rs#784)
- Update AWS service models (smithy-rs#790)
- Add support for the following Glacier customizations:
  - Set the ApiVersion header (smithy-rs#138, #787)

v0.0.21-alpha (October 15th, 2021)
==================================

**New this week**

- Prepare crate manifests for publishing to crates.io (smithy-rs#755)
- Add support for IAM Roles for tasks credential provider (smithy-rs#765, aws-sdk-rust#123)
- All service crates now have generated README files (smithy-rs#766)
- Update AWS service models (smithy-rs#772)
- :tada: Add support for Amazon Managed Grafana (smithy-rs#772)

v0.0.20-alpha (October 7, 2021)
===============================

**Breaking changes**

- :warning: MSRV increased from 1.52.1 to 1.53.0 per our 3-behind MSRV policy.
- `SmithyConnector` and `DynConnector` now return `ConnectorError` instead of `Box<dyn Error>`. If you have written a custom connector, it will need to be updated to return the new error type. (#744)
- The `DispatchError` variant of `SdkError` now contains `ConnectorError` instead of `Box<dyn Error>` (#744).

**New This Week**

- :tada: Make retry behavior configurable
    - With env vars `AWS_MAX_ATTEMPTS` and `AWS_RETRY_MODE`
    - With `~/.aws/config` settings `max_attempts` and `retry_mode`
    - By calling the `with_retry_config` method on a `Config` and passing in a `RetryConfig`
    - Only the `Standard` retry mode is currently implemented. `Adaptive` retry mode will be implemented at a later
      date.
    - For more info, see the AWS Reference pages on configuring these settings:
        - [Setting global max attempts](https://docs.aws.amazon.com/sdkref/latest/guide/setting-global-max_attempts.html)
        - [Setting global retry mode](https://docs.aws.amazon.com/sdkref/latest/guide/setting-global-retry_mode.html)
- :tada: Add presigned request support and examples for S3 GetObject and PutObject (smithy-rs#731, aws-sdk-rust#139)
- :tada: Add presigned request support and example for Polly SynthesizeSpeech (smithy-rs#735, aws-sdk-rust#139)
- Add connect & HTTP read timeouts to IMDS, defaulting to 1 second
- IO and timeout errors from Hyper can now be retried (#744)
- :bug: Fix error when receiving `Cont` event from S3 SelectObjectContent (smithy-rs#736)
- :bug: Fix bug in event stream receiver that could cause the last events in the response stream to be lost when using S3 SelectObjectContent (smithy-rs#736)
- Updated EC2 code examples to include readme; refactored operations from main into separate functions.
- Updated Transcribe code example to take an audio file as a command-line option and added readme.
- Refactored API Gateway code example by moving operation out of main and into a separate function; added readme.
- Updated Auto Scaling code example to move operation from main to separate function; added readme.
- Updated AWS Config code examples to include a readme; added command-line options; added DeleteConfigurationRecorder, DeleteDeliveryChannel, ListConfigurationRecorders, ListDeliveryChannels, ListResources, ShowResourceHistory, and EnableConfig code examples.
- :tada: Add support for 6 new AWS services:
    - Wisdom
    - VoiceId
    - Account
    - KafkaConnect
    - OpenSearch
    - CloudControl

v0.0.19-alpha (September 24th, 2021)
====================================

**New This Week**

- :tada: IMDS support in the default credential provider chain (aws-sdk-rust#97)
- :tada: Add `sts::AssumeRoleProvider` to `aws-config`. This enables customers to invoke STS directly,
  instead of using it via `~/.aws/config`. (smithy-rs#703, aws-sdk-rust#3)
- Add IMDS client to `aws-config` (smithy-rs#701)
- Add IMDS credential provider to `aws-config` (smithy-rs#709)
- Add IMDS region provider to `aws-config` (smithy-rs#715, aws-sdk-rust#97)
- Update event stream `Receiver`s to be `Send` (aws-sdk-rust#224)
- Add query param signing to the `aws-sigv4` crate (smithy-rs#707)
- :bug: Update event stream `Receiver`s to be `Send` (smithy-rs#702, #aws-sdk-rust#224)
- :bug: Fix panic when signing non-ASCII header values (smithy-rs#708, aws-sdk-rust#226)
- Add an example that uses Polly, Transcribe, and S3 called [telephone-game](sdk/examples/telephone-game/src/main.rs)

**Contributions**

Thank you for your contributions! :heart:

- @jonhoo (smithy-rs#703)

v0.0.18-alpha (September 14th, 2021)
=======================

- :tada: Add support for `OpenSearch` service & bring in other model updates (#todo)
- Cleanup docs in `aws-config`

**New This Week**
- :bug: Fixes issue where `Content-Length` header could be duplicated leading to signing failure (aws-sdk-rust#220, smithy-rs#697)

- Updated AutoScaling code examples to use asynchronous config; added readme file; tested on 0.0.17 bits

v0.0.17-alpha (September 2nd, 2021)
===================================

This release adds support for three commonly requested features:
- More powerful credential chain
- Support for constructing multiple clients from the same configuration
- Support for Transcribe streaming and S3 Select

In addition, this overhauls client configuration which lead to a number of breaking changes. Detailed changes are inline.

Current Credential Provider Support:
- [x] Environment variables
- [x] Web Identity Token Credentials
- [ ] Profile file support (partial)
  - [ ] Credentials
    - [ ] SSO
    - [ ] ECS Credential source
    - [ ] IMDS credential source
    - [x] Assume role from source profile
    - [x] Static credentials source profile
    - [x] WebTokenIdentity provider
  - [x] Region
- [ ] IMDS
- [ ] ECS

Upgrade Guide
-------------

### If you use `<sdk>::Client::from_env`

`from_env` loaded region & credentials from environment variables _only_. Default sources have been removed from the generated
SDK clients and moved to the `aws-config` package. Note that the `aws-config` package default chain adds support for
profile file and web identity token profiles.

1. Add a dependency on `aws-config`:
     ```toml
     [dependencies]
     aws-config = { git = "https://github.com/awslabs/aws-sdk-rust", tag = "v0.0.17-alpha" }
     ```
2. Update your client creation code:
   ```rust
   // `shared_config` can be used to construct multiple different service clients!
   let shared_config = aws_config::load_from_env().await;
   // before: <service>::Client::from_env();
   let client = <service>::Client::new(&shared_config)
   ```

### If you used `<client>::Config::builder()`

`Config::build()` has been modified to _not_ fallback to a default provider. Instead, use `aws-config` to load and modify
the default chain. Note that when you switch to `aws-config`, support for profile files and web identity tokens will be added.

1. Add a dependency on `aws-config`:
     ```toml
     [dependencies]
     aws-config = { git = "https://github.com/awslabs/aws-sdk-rust", tag = "v0.0.17-alpha" }
     ```

2. Update your client creation code:

   ```rust
   fn before() {
     let region = aws_types::region::ChainProvider::first_try(<1 provider>).or_default_provider();
     let config = <service>::Config::builder().region(region).build();
     let client = <service>::Client::from_conf(&config);
   }

   async fn after() {
     use aws_config::meta::region::RegionProviderChain;
     let region_provider = RegionProviderChain::first_try(<1 provider>).or_default_provider();
     // `shared_config` can be used to construct multiple different service clients!
     let shared_config = aws_config::from_env().region(region_provider).load().await;
     let client = <service>::Client::new(&shared_config)
   }
   ```

### If you used `aws-auth-providers`
All credential providers that were in `aws-auth-providers` have been moved to `aws-config`. Unless you have a specific use case
for a specific credential provider, you should use the default provider chain:

```rust
 let shared_config = aws_config::load_from_env().await;
 let client = <service>::Client::new(&shared_config);
```

### If you maintain your own credential provider

`AsyncProvideCredentials` has been renamed to `ProvideCredentials`. The trait has been moved from `aws-auth` to `aws-types`.
The original `ProvideCredentials` trait has been removed. The return type has been changed to by a custom future.

For synchronous use cases:
```rust
use aws_types::credentials::{ProvideCredentials, future};

#[derive(Debug)]
struct CustomCreds;
impl ProvideCredentials for CustomCreds {
  fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
            Self: 'a,
  {
    // if your credentials are synchronous, use `::ready`
    // if your credentials are loaded asynchronously, use `::new`
    future::ProvideCredentials::ready(todo!()) // your credentials go here
  }
}
```

For asynchronous use cases:
```rust
use aws_types::credentials::{ProvideCredentials, future, Result};

#[derive(Debug)]
struct CustomAsyncCreds;
impl CustomAsyncCreds {
  async fn load_credentials(&self) -> Result {
    Ok(Credentials::from_keys("my creds...", "secret", None))
  }
}

impl ProvideCredentials for CustomCreds {
  fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
            Self: 'a,
  {
    future::ProvideCredentials::new(self.load_credentials())
  }
}
```

Changes
-------

**Breaking Changes**

- Credential providers from `aws-auth-providers` have been moved to `aws-config` (smithy-rs#678)
- `AsyncProvideCredentials` has been renamed to `ProvideCredentials`. The original non-async provide credentials has been
  removed. See the migration guide above.
- `<sevicename>::from_env()` has been removed (#675). A drop-in replacement is available:
  1. Add a dependency on `aws-config`:
     ```toml
     [dependencies]
     aws-config = { git = "https://github.com/awslabs/aws-sdk-rust", tag = "v0.0.17-alpha" }
     ```
  2. Update your client creation code:
     ```rust
     let client = <service>>::Client::new(&aws_config::load_from_env().await)
     ```

- `ProvideRegion` has been moved to `aws_config::meta::region::ProvideRegion`. (smithy-rs#675)
- `aws_types::region::ChainProvider` has been moved to `aws_config::meta::region::RegionProviderChain` (smithy-rs#675).
- `ProvideRegion` is now asynchronous. Code that called `provider.region()` must be changed to `provider.region().await`.
- `<awsservice>::Config::builder()` will **not** load a default region. To preserve previous behavior:
  1. Add a dependency on `aws-config`:
     ```toml
     [dependencies]
     aws-config = { git = "https://github.com/awslabs/aws-sdk-rust", tag = "v0.0.17-alpha" }
     ```
  2. ```rust
     let shared_config = aws_config::load_from_env().await;
     let config = <service>::config::Builder::from(&shared_config).<other builder modifications>.build();
     ```

**New this week**

- :tada: Add profile file provider for region (smithy-rs#594, smithy-rs#682)
- :tada: Add support for shared configuration between multiple services (smithy-rs#673)
- :tada: Add support for Transcribe `StartStreamTranscription` and S3 `SelectObjectContent` operations (smithy-rs#667)
- :tada: Add support for new MemoryDB service (smithy-rs#677)
- Improve documentation on collection-aware builders (smithy-rs#664)
- Update AWS SDK models (smithy-rs#677)
- :bug: Fix sigv4 signing when request ALPN negotiates to HTTP/2. (smithy-rs#674)
- :bug: Fix integer size on S3 `Size` (smithy-rs#679, #209)
- :bug: Fix MediaLive response parsing issue (smithy-rs#683, #212)


v0.0.16-alpha (August 19th, 2021)
=================================

**New This Week**

- :tada: Add Chime Identity, Chime Messaging, and Snow Device Management support (smithy-rs#657)
- :tada: Add profile file credential provider implementation. This implementation currently does not support credential sources for assume role providers other than environment variables. (smithy-rs#640)
- :tada: Add support for WebIdentityToken providers via profile & environment variables. (smithy-rs#654)
- :bug: Fix name collision that occurred when a model had both a union and a structure named `Result` (smithy-rs#643)
- :bug: Fix STS Assume Role with WebIdentity & Assume role with SAML to support clients with no credentials provided (smithy-rs#652)
- Update AWS SDK models (smithy-rs#657)
- Add initial implementation of a default provider chain. (smithy-rs#650)

v0.0.15-alpha (August 11th, 2021)
=================================

This release primarily contains internal changes to runtime components & updates to AWS models.

**Breaking changes**

- (smithy-rs#635) The `config()`, `config_mut()`, `request()`, and `request_mut()` methods on `operation::Request` have been renamed to `properties()`, `properties_mut()`, `http()`, and `http_mut()` respectively.
- (smithy-rs#635) The `Response` type on Tower middleware has been changed from `http::Response<SdkBody>` to `operation::Response`. The HTTP response is still available from the `operation::Response` using its `http()` and `http_mut()` methods.
- (smithy-rs#635) The `ParseHttpResponse` trait's `parse_unloaded()` method now takes an `operation::Response` rather than an `http::Response<SdkBody>`.
- (smithy-rs#626) `ParseHttpResponse` no longer has a generic argument for the body type, but instead, always uses `SdkBody`. This may cause compilation failures for you if you are using Smithy generated types to parse JSON or XML without using a client to request data from a service. The fix should be as simple as removing `<SdkBody>` in the example below:

  Before:
  ```rust
  let output = <Query as ParseHttpResponse<SdkBody>>::parse_loaded(&parser, &response).unwrap();
  ```

  After:
  ```rust
  let output = <Query as ParseHttpResponse>::parse_loaded(&parser, &response).unwrap();
  ```

**New This Week**

- The closure passed to `async_provide_credentials_fn` can now borrow values (smithy-rs#637)
- Bring in the latest AWS models (smithy-rs#630)

v0.0.14-alpha (July 28th, 2021)
===============================

IoT Data Plane is now available! If you discover it isn't functioning as expected, please let us know!

This week also sees the addition of a robust async caching credentials provider. Take a look at the [STS example](https://github.com/awslabs/smithy-rs/blob/7fa4af4a9367aeca6d55e26fc4d4ba93093b90c4/aws/sdk/examples/sts/src/bin/credentials-provider.rs) to see how to use it.

To upgrade to the new release, update `tag` to `v0.0.14-alpha`:
```
[dependencies]
# e.g. S3:
aws-sdk-s3 = { git = "https://github.com/awslabs/aws-sdk-rust", tag = "v0.0.14-alpha" }
```

**New This Week**

- :tada: Add IoT Data Plane (smithy-rs#624)
- :tada: Add LazyCachingCredentialsProvider to aws-auth for use with expiring credentials, such as STS AssumeRole. Update STS example to use this new provider (smithy-rs#578, smithy-rs#595)
- :bug: Correctly encode HTTP Checksums using base64 instead of hex. Fixes #164. (smithy-rs#615)
- Overhaul serialization/deserialization of numeric/boolean types. This resolves issues around serialization of NaN/Infinity and should also reduce the number of allocations required during serialization. (smithy-rs#618)
- Update SQS example to clarify usage of FIFO vs. standard queues (#162, @trevorrobertsjr)

**Contributions**

Thank you for your contributions! :heart:

- @trevorrobertsjr (#622)


v0.0.13-alpha (July 28th, 2021)
===============================

:tada: This week's release includes most of the remaining AWS services (269 in total!).

**Breaking changes**
- `test-util` has been made an optional dependency and has moved from
  aws-hyper to smithy-http. If you were relying on `aws_hyper::TestConnection`, add `smithy-client` as a dependency
  and enable the optional `test-util` feature. This prunes some unnecessary dependencies on `roxmltree` and `serde_json`
  for most users. (smithy-rs#608)

**New This Week**
- :tada: Release all but four remaining AWS services! Glacier, IoT Data Plane, Timestream DB and Transcribe Streaming will be available in a future release. If you discover that a service isn't functioning as expected please let us know! (smithy-rs#607)
- :bug: Bugfix: Fix parsing bug where parsing XML incorrectly stripped whitespace (smithy-rs#590, #153)
- We now run some tests on Windows (smithy-rs#594)
- :bug: Bugfix: Constrain RFC-3339 timestamp formatting to microsecond precision (smithy-rs#596, #152)


v0.0.12-alpha (July 19th, 2021)
===============================

This week we've added Autoscaling and fixed an S3 bug.

To update to the new release, change your tag to v0.0.12-alpha.

**New this Week**
- :tada: Add support for Autoscaling (#576, #582)
- `AsyncProvideCredentials` now introduces an additional lifetime parameter, simplifying bridging it with `#[async_trait]` interfaces
- Fix S3 bug when content type was set explicitly (aws-sdk-rust#131, #566, @eagletmt)

**Contributions**
Thank you for your contributions! :heart:
- @eagletmt (#566)


v0.0.11-alpha (July 6th, 2021)
==============================

This week, we've added AWS Config, EBS, Cognito, and Snowball. Projects that are implementing the `ProvideCredentials` trait will need to update their imports and should consider using the new `async_provide_credentials_fn` for async credential use-cases.

To update to the new release, change your tag to `v0.0.11-alpha`.

**New this Week**
- :warning: **Breaking Change:** `ProvideCredentials` and `CredentialError` were both moved into `aws_auth::provider` when they were previously in `aws_auth` (#572)
- :tada: Add support for AWS Config (#570)
- :tada: Add support for EBS (#567)
- :tada: Add support for Cognito (#573)
- :tada: Add support for Snowball (#579, @landonxjames)
- Make it possible to asynchronously provide credentials with `async_provide_credentials_fn` (#572, #577)
- Improve RDS, QLDB, Polly, and KMS examples (#561, #560, #558, #556, #550)
- Update AWS SDK models (#575)
- :bug: Bugfix: Fill in message from error response even when it doesn't match the modeled case format (#565)

**Contributions**

Thank you for your contributions! :heart:

- landonxjames (#579)


v0.0.10-alpha (June 29th, 2021)
===============================

This week, we've added EKS, ECR and Cloudwatch. The JSON deserialization implementation has been replaced, please be
on the lookout for potential issues and compile time improvements.

To update to the new release, change your tag to `v0.0.10-alpha`.

**New this Week**
- :tada: Add support for ECR (smithy-rs#557)
- :tada: Add support for Cloudwatch (smithy-rs#554)
- :tada: Add support for EKS (smithy-rs#553)
- :warning: **Breaking Change:** httpLabel no longer causes fields to be non-optional. You may need to adapt code that uses models. (#537)
- :warning: **Breaking Change:** `Exception` is **not** renamed to `Error`. Code may need to be updated to replace `Error` with `Exception` when naming error shapes.
- :warning: **Breaking Change:** Models are now in strict pascal case including acronyms (e.g. `dynamodb::model::{SSESpecification => SseSpecification}`)
- Add more SES examples, and improve examples for Batch.
- Improved error handling ergonomics: Errors now provide `is_<variantname>()` methods to simplify error handling
- :bug: Bugfix: Fix bug in `create_multipart_upload`: #127 (smithy-rs#531, @eagletmt)

**Contributors**

Thank you for your contributions! :heart:

- @eagletmt (#531)


v0.0.9-alpha (June 22th, 2021)
==============================

This week, we've added CloudWatch Logs support and fixed several bugs in the generated S3 clients.
There are breaking changes on builders and unions this week.

To upgrade to the new release, update `tag` to `v0.0.9-alpha`:

```toml
[dependencies]
# e.g. Cloudwatch Logs:
aws-sdk-cloudwatchlogs = { git = "https://github.com/awslabs/aws-sdk-rust", tag = "v0.0.9-alpha" }
```

**New this Week**
- :tada: Add support for CloudWatch Logs (smithy-rs#526)
- :warning: **Breaking Change:** The `set_*` functions on generated Builders now always take an `Option` (smithy-rs#506)
- :warning: **Breaking Change:** The `as_*` functions on unions now return `Result` rather than `Option` to clearly indicate what the actual value is (smithy-rs#527)
- Add more S3 examples, and improve SNS, SQS, and SageMaker examples. Improve example doc comments (smithy-rs#490, smithy-rs#508, smithy-rs#509, smithy-rs#510, smithy-rs#511, smithy-rs#512, smithy-rs#513, smithy-rs#524)
- Combine individual example packages into per-service example packages with multiple binaries (smithy-rs#481, smithy-rs#490)
- :bug: Bugfix: Show response body in trace logs for calls that don't return a stream (smithy-rs#514)
- :bug: Bugfix: Correctly parse S3's GetBucketLocation response (smithy-rs#516)
- :bug: Bugfix: Fix S3 ListObjectsV2 for prefixes containing tilde characters (smithy-rs#519)
- :bug: Bugfix: Fix S3 PutBucketLifecycle operation by adding support for the `@httpChecksumRequired` Smithy trait (smithy-rs#523)
- :bug: Bugfix: Correctly parse `x-amz-expiration` header on S3 GetObject responses (smithy-rs#525, @eagletmt)

**Contributions**

Thank you for your contributions! :heart:

- @eagletmt (smithy-rs#525)
- @zekisherif (smithy-rs#515)


v0.0.8-alpha (June 15th, 2021)
==============================

This week, we've added CloudFormation, SageMaker, EC2, and SES. More details below.

To upgrade to the new release, update `tag` to `v0.0.8-alpha`:

```toml
[dependencies]
# e.g. EC2:
aws-sdk-ec2 = { git = "https://github.com/awslabs/aws-sdk-rust", tag = "v0.0.8-alpha" }
```

**New this Week**
- :tada: Add support for CloudFormation (smithy-rs#500, @alistaim)
- :tada: Add support for SageMaker (smithy-rs#473, @alistaim)
- :tada: Add support for EC2 (smithy-rs#495)
- :tada: Add support for SES (smithy-rs#499)
- Add support for the EC2 Query protocol (smithy-rs#475)
- Refactor smithy/hyper connectors to enable client-specified middleware (smithy-rs#496, @jonhoo)
- :bug: Bugfix: RFC-3339 timestamp formatting is no longer truncating zeros off of the number of seconds (smithy-rs#479, smithy-rs#489)

Contributors:
- @Doug-AWS
- @jdisanti
- @rcoh
- @alistaim
- @jonhoo

Thanks!!


v0.0.7-alpha (June 8th, 2021)
=============================

This week we’ve added MediaLive, MediaPackage, SNS, Batch, STS, RDS, RDSData, Route53, and IAM. More details below.

To upgrade to the new release, update `tag` to `v0.0.7-alpha`:
```toml
[dependencies]
# e.g. SNS:
aws-sdk-sns = { git = "https://github.com/awslabs/aws-sdk-rust", tag = "v0.0.7-alpha" }
```

**New this Week**
- **Breaking change**: Some string enums have changed case:`DynamoDB::{SSEStatus => SseStatus. SSEType => SseType}`
- :tada: Add support for MediaLive and MediaPackage (#449, @alastaim)
- :tada: Add support for SNS (smithy-rs#450)
- :tada: Add support for Batch (smithy-rs#452)
- :tada: Add support for STS. **Note:** This does not include support for an STS-based credential provider although an example is provided. (smithy-rs#453)
- :tada: Add support for RDS (smithy-rs#455) and RDS-Data (smithy-rs#470). (@LMJW)
- :tada: Add support for Route53 (smithy-rs#457, @alistaim)
- Support AWS Endpoints & Regions. With this update, regions like `iam-fips` and `cn-north-1` will now resolve to the correct endpoint. Please report any issues with endpoint resolution. (smithy-rs#468)
- :bug: Primitive numerics and booleans are now filtered from serialization when they are 0 and not marked as required. This resolves issues where maxResults needed to be set even though it is optional & fixes errors during deserialization. (smithy-rs#451)
- :bug: S3 Head Object returned the wrong error when the object did not exist (smithy-rs#460, fixes smithy-rs#456)


Contributors:
- @rcoh
- @jdisanti
- @alistaim
- @LMJW

Thanks!


v0.0.6-alpha (June 1st, 2021)
=============================

**New this week:**

- :tada: Add support for SQS. SQS is our first service to use the awsQuery protocol. Please report any issues you may encounter.
- :tada: Add support for ECS.
- **Breaking Change**: Refactored `smithy_types::Error` to be more flexible. Internal fields of `Error` are now private and can now be accessed accessor functions. (smithy-rs#426)
- **Breaking change**: Smithy Enums do not implement `serde::Serialize`
- `ByteStream::from_path` now accepts `implications AsRef<Path>` (@LMJW)
- Add support for S3 extended request id (smithy-rs#429)
- Add support for the awsQuery protocol. smithy-rs can now add support for all services except EC2.
- **Bugfix**: Timestamps that fell precisely on minute boundaries were not properly formatted (smithy-rs#435)
- Improve documentation for `ByteStream` & add `pub use ByteStream` to generated crates (smithy-rs#443)
- Add support for `EndpointPrefix` needed for [`s3::WriteGetObjectResponse`](https://awslabs.github.io/aws-sdk-rust/aws_sdk_s3/operation/struct.WriteGetObjectResponse.html) (smithy-rs#420)

Contributors:
- @jdisanti
- @rcoh
- @LMJW

Thanks!


v0.0.5-alpha (May 25th, 2021)
=============================

You can install the new release by updating your dependencies to `tag = "v0.0.5-alpha"`, e.g.
```toml
[dependencies]
aws-sdk-s3 = { git = "https://github.com/awslabs/aws-sdk-rust", tag = "v0.0.5-alpha" }
```

## New This Week
- :tada: Add S3 support.  S3 is the first protocol to use our new XML serializers which increases the likelihood of undiscovered issues. In addition, virtual addressing, dualstack and transfer acceleration are not currently supported.  Please try it out and let us know if you run into any problems! (smithy-rs#398) :tada:
- :tada: Add support for SSM. SSM was prioritized based on your votes—Please keep voting for the services and feature most important to you! (smithy-rs#393) :tada:
- Add request/response tracing. These can be enabled via tracing subscriber by setting: `RUST_LOG='smithy_http_tower::dispatch=trace,smithy_http::middleware=trace'` (smithy-rs#397)
- Bugfix: Generated service docs were missing at the module level (smithy-rs#404)
- `ByteStream` can now be created from `Path` and `File` via `ByteStream::from_path` (smithy-rs#412)
- Example code now uses `write_all_buf` (#408, @lmjw)
- The `Authorization` and `x-amz-security-token` headers are now marked as sensitive and will be omitted from logs even when full request/response tracing is enabled

And more: See the corresponding [smithy-rs release](https://github.com/awslabs/smithy-rs/releases/tag/v0.10).

Contributors:
- @rcoh
- @jdisanti
- @LMJW

Thanks!


v0.0.4-alpha (May 18th, 2021)
=============================

You can install the new release by updating your dependencies to `tag = "v0.0.4-alpha"`, e.g.
```toml
[dependencies]
aws-sdk-lambda = { git = "https://github.com/awslabs/aws-sdk-rust", tag = "v0.0.4-alpha" }
```

**New this week**:

- :tada: Add support for AWS Lambda (smithy-rs#361, @richardhboyd) :tada:
- Generate docs automatically and host on GitHub Pages: https://awslabs.github.io/aws-sdk-rust/ (#81)
- Add support for streaming request bodies. This is technically a **breaking change** but no currently generated AWS services expose this type. (smithy-rs#359)
- Types represented by the Smithy `Set` type now generate `Vec<T>` in all cases. This is also technically breaking but not currently exposed. (smithy-rs#270)
- Bugfix: The `.message()`field of errors will now look for both `message` and `Message` in the model (smithy-rs#374)
- Add support for the `AWS_REGION` environment variable. (smithy-rs#362)
- The request type generated by the fluent builders, e.g. `dynamodb.list_tables()` is now `Debug` (smithy-rs#377, @declanvk)

And more: See the corresponding [smithy-rs release](https://github.com/awslabs/smithy-rs/releases/tag/v0.9).

Contributors:
- @richardhboyd
- @declanvk
- @jdisanti2019
- @rcoh

Thanks!


v0.0.3-alpha (May 6th, 2021)
============================

**New this week:**

- Fix stack overflow in `SdkBody` Debug implementation
- Upgrade to Smithy 1.7. This adds support for several new API Gateway endpoints
- Add support for streaming response bodies. This is currently only used in Polly
- Added code examples for Kinesis

More details in smithy-rs: https://github.com/awslabs/smithy-rs/releases/tag/v0.8
