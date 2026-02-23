# aws-sdk-wickr

Welcome to the _Amazon Web Services Wickr API Reference_.

The Amazon Web Services Wickr application programming interface (API) is designed for administrators to perform key tasks, such as creating and managing Amazon Web Services Wickr, networks, users, security groups, bots and more. This guide provides detailed information about the Amazon Web Services Wickr API, including operations, types, inputs and outputs, and error codes. You can use an Amazon Web Services SDK, the Amazon Web Services Command Line Interface (Amazon Web Services CLI, or the REST API to make API calls for Amazon Web Services Wickr.

_Using Amazon Web Services SDK_

The SDK clients authenticate your requests by using access keys that you provide. For more information, see [Authentication and access using Amazon Web Services SDKs and tools](https://docs.aws.amazon.com/sdkref/latest/guide/access.html) in the _Amazon Web Services SDKs and Tools Reference Guide_.

_Using Amazon Web Services CLI_

Use your access keys with the Amazon Web Services CLI to make API calls. For more information about setting up the Amazon Web Services CLI, see [Getting started with the Amazon Web Services CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-getting-started.html) in the _Amazon Web Services Command Line Interface User Guide for Version 2_.

_Using REST APIs_

If you use REST to make API calls, you must authenticate your request by providing a signature. Amazon Web Services Wickr supports Signature Version 4. For more information, see [Amazon Web Services Signature Version 4 for API requests](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv.html) in the _Amazon Web Services Identity and Access Management User Guide_.

Access and permissions to the APIs can be controlled by Amazon Web Services Identity and Access Management. The managed policy [Amazon Web ServicesWickrFullAccess](https://docs.aws.amazon.com/wickr/latest/adminguide/security-iam-awsmanpol.html#security-iam-awsmanpol-AWSWickrFullAccess) grants full administrative permission to the Amazon Web Services Wickr service APIs. For more information on restricting access to specific operations, see [Identity and access management for Amazon Web Services Wickr](https://docs.aws.amazon.com/wickr/latest/adminguide/security-iam.html) in the _Amazon Web Services Wickr Administration Guide_.

_Types of Errors_:

The Amazon Web Services Wickr APIs provide an HTTP interface. HTTP defines ranges of HTTP Status Codes for different types of error responses.
  1. Client errors are indicated by HTTP Status Code class of 4xx
  1. Service errors are indicated by HTTP Status Code class of 5xx

In this reference guide, the documentation for each API has an Errors section that includes a brief discussion about HTTP status codes. We recommend looking there as part of your investigation when you get an error.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-wickr` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-wickr = "1.5.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_wickr as wickr;

#[::tokio::main]
async fn main() -> Result<(), wickr::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_wickr::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-wickr/latest/aws_sdk_wickr/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1)

## License

This project is licensed under the Apache-2.0 License.

