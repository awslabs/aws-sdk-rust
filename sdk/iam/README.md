# aws-sdk-iam

Identity and Access Management (IAM) is a web service for securely controlling access to Amazon Web Services services. With IAM, you can centrally manage users, security credentials such as access keys, and permissions that control which Amazon Web Services resources users and applications can access. For more information about IAM, see [Identity and Access Management (IAM)](http://aws.amazon.com/iam/) and the [Identity and Access Management User Guide](https://docs.aws.amazon.com/IAM/latest/UserGuide/).

__Programmatic access to IAM__

We recommend that you use the Amazon Web Services SDKs to make programmatic API calls to IAM. The Amazon Web Services SDKs consist of libraries and sample code for various programming languages and platforms (for example, Java, Ruby, .NET, iOS, and Android). The SDKs provide a convenient way to create programmatic access to IAM and Amazon Web Services. For example, the SDKs take care of tasks such as cryptographically signing requests, managing errors, and retrying requests automatically. For more information, see [Tools to build on Amazon Web Services](http://aws.amazon.com/tools/).

Alternatively, you can also use the IAM Query API to make direct calls to the IAM service. For more information about calling the IAM Query API, see [Making query requests](https://docs.aws.amazon.com/IAM/latest/UserGuide/IAM_UsingQueryAPI.html) in the _Identity and Access Management User Guide_. IAM supports GET and POST requests for all actions. That is, the API does not require you to use GET for some actions and POST for others. However, GET requests are subject to the limitation size of a URL. Therefore, for operations that require larger sizes, use a POST request.

__Signing requests__

Requests must be signed using an access key ID and a secret access key. We strongly recommend that you do not use your Amazon Web Services account access key ID and secret access key for everyday work with IAM. You can use the access key ID and secret access key for an IAM user or you can use the Security Token Service to generate temporary security credentials and use those to sign requests.

To sign requests, we recommend that you use [Signature Version 4](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html). If you have an existing application that uses Signature Version 2, you do not have to update it to use Signature Version 4. However, some operations now require Signature Version 4. The documentation for operations that require version 4 indicate this requirement.

__Additional resources__
  - [Amazon Web Services security credentials](https://docs.aws.amazon.com/general/latest/gr/aws-security-credentials.html). This topic provides general information about the types of credentials used for accessing Amazon Web Services.
  - [IAM best practices](https://docs.aws.amazon.com/IAM/latest/UserGuide/IAMBestPractices.html). This topic presents a list of suggestions for using the IAM service to help secure your Amazon Web Services resources.
  - [Signing Amazon Web Services API requests](https://docs.aws.amazon.com/general/latest/gr/signing_aws_api_requests.html). This set of topics walk you through the process of signing a request using an access key ID and secret access key.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-iam` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-iam = "1.113.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_iam as iam;

#[::tokio::main]
async fn main() -> Result<(), iam::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_iam::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-iam/latest/aws_sdk_iam/client/struct.Client.html)
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

