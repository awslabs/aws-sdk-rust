# aws-sdk-ssooidc

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Web Services Single Sign On OpenID Connect (OIDC) is a web service that enables a client (such as Amazon Web Services CLI or a native application) to register with Amazon Web Services SSO. The service also enables the client to fetch the user’s access token upon successful authentication and authorization with Amazon Web Services SSO.

__Considerations for Using This Guide__

Before you begin using this guide, we recommend that you first review the following important information about how the Amazon Web Services SSO OIDC service works.
  - The Amazon Web Services SSO OIDC service currently implements only the portions of the OAuth 2.0 Device Authorization Grant standard ([https://tools.ietf.org/html/rfc8628](https://tools.ietf.org/html/rfc8628)) that are necessary to enable single sign-on authentication with the AWS CLI. Support for other OIDC flows frequently needed for native applications, such as Authorization Code Flow (+ PKCE), will be addressed in future releases.
  - The service emits only OIDC access tokens, such that obtaining a new token (For example, token refresh) requires explicit user re-authentication.
  - The access tokens provided by this service grant access to all AWS account entitlements assigned to an Amazon Web Services SSO user, not just a particular application.
  - The documentation in this guide does not describe the mechanism to convert the access token into AWS Auth (“sigv4”) credentials for use with IAM-protected AWS service endpoints. For more information, see [GetRoleCredentials](https://docs.aws.amazon.com/singlesignon/latest/PortalAPIReference/API_GetRoleCredentials.html) in the _Amazon Web Services SSO Portal API Reference Guide_.

For general information about Amazon Web Services SSO, see [What is Amazon Web Services SSO?](https://docs.aws.amazon.com/singlesignon/latest/userguide/what-is.html) in the _Amazon Web Services SSO User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ssooidc` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.49.0"
aws-sdk-ssooidc = "0.19.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust
use aws_sdk_ssooidc as ssooidc;

#[tokio::main]
async fn main() -> Result<(), ssooidc::Error> {
    let config = aws_config::load_from_env().await;
    let client = ssooidc::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-ssooidc/latest/aws_sdk_ssooidc/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) – For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

