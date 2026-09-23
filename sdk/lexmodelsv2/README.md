# aws-sdk-lexmodelsv2

This document provides detailed information about the Amazon Lex V2 API actions and their parameters.

For information about the IAM access control permissions you need to use this API, see [Identity-based policies for Amazon Lex V2](https://docs.aws.amazon.com/lexv2/latest/dg/security_iam_service-with-iam.html).

Amazon Lex V2 Model Building V2 operations let you build and manage bots.

If you use a custom HTTP client to call Amazon Lex Model Building V2 operations, you must set the "Content-Type" HTTP header to "application/x-amz-json-1.1". Otherwise, you receive an HTTP 404 - UnknownOperationException in the response.

Amazon Lex Model Building V2 operations return the responses with the "application/x-amz-json-1.1" content type.

You can use [Amazon Web Services SDKs](http://aws.amazon.com/tools/#sdk) to access Amazon Lex V2 APIs using your favorite programming language. The SDKs automatically perform useful tasks for you, such as:
  - Cryptographically sign your service requests
  - Retry requests
  - Handle error responses

The following resources provide additional information about the Amazon Lex V2 Model Building API.
  - _Amazon Web Services General Reference_
    - [Amazon Lex V2 Endpoints for each region](https://docs.aws.amazon.com/general/latest/gr/lex.html).

  - _Command Line Interface_
    - [Amazon Lex Model Building V2 CLI commands](https://docs.aws.amazon.com/cli/latest/reference/lexv2-models/index.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-lexmodelsv2` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-lexmodelsv2 = "1.125.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_lexmodelsv2 as lexmodelsv2;

#[::tokio::main]
async fn main() -> Result<(), lexmodelsv2::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_lexmodelsv2::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-lexmodelsv2/latest/aws_sdk_lexmodelsv2/client/struct.Client.html)
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

