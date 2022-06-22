# aws-sdk-ssooidc

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

AWS Single Sign-On (SSO) OpenID Connect (OIDC) is a web service that enables a client (such as AWS CLI or a native application) to register with AWS SSO. The service also enables the client to fetch the user’s access token upon successful authentication and authorization with AWS SSO. This service conforms with the OAuth 2.0 based implementation of the device authorization grant standard ([https://tools.ietf.org/html/rfc8628](https://tools.ietf.org/html/rfc8628)).

For general information about AWS SSO, see [What is AWS Single Sign-On?](https://docs.aws.amazon.com/singlesignon/latest/userguide/what-is.html) in the _AWS SSO User Guide_.

This API reference guide describes the AWS SSO OIDC operations that you can call programatically and includes detailed information on data types and errors.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ssooidc` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-ssooidc = "0.14.0"
tokio = { version = "1", features = ["full"] }
```

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

