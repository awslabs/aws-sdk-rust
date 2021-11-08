vNext (Month Day, Year)
=======================

v0.0.23-alpha (November 3rd, 2021)
==================================
**New this week**
- The SDK is available on crates.io!
- :tada: Add support for AWS Glacier (smithy-rs#801)
- :tada: Add support for AWS Panorama
- :bug: Fix `native-tls` feature in `aws-config` (aws-sdk-rust#265, smithy-rs#803)
- Add example to aws-sig-auth for generating an IAM Token for RDS (smithy-rs#811, aws-sdk-rust#147)
- :bug: `hyper::Error(IncompleteMessage)` will now be retried (smithy-rs#815)
- :bug: Fix generated docs on unions like `dynamodb::AttributeValue`. (smithy-rs#826)

**Breaking Changes**
- `<operation>.make_operation(&config)` is now an `async` function for all operations. Code should be updated to call `.await`. This will only impact users using the low-level API. (smithy-rs#797)
- :bug: S3 request metadata signing now correctly trims headers fixing [problems like this](https://github.com/awslabs/aws-sdk-rust/issues/248) (smithy-rs#761)

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
# eg. S3:
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
- :warning: **Breaking Change:** Models are now in strict pascal case including acronyms (eg. `dynamodb::model::{SSESpecification => SseSpecification}`)
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
# eg. Cloudwatch Logs:
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
# eg. EC2:
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
# eg. SNS:
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

You can install the new release by updating your dependencies to `tag = "v0.0.5-alpha"`, eg.
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

You can install the new release by updating your dependencies to `tag = "v0.0.4-alpha"`, eg.
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
- The request type generated by the fluent builders, eg. `dynamodb.list_tables()` is now `Debug` (smithy-rs#377, @declanvk)

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
