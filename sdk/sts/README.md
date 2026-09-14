# aws-sdk-sts

Amazon Web Services provides Security Token Service (STS) as a web service that enables you to request temporary, limited-privilege credentials for users. This guide describes the STS API. For more information, see [Temporary Security Credentials](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_credentials_temp.html) in the _IAM User Guide_.

For information about setting up signatures and authorization through the API, see [Signing Amazon Web Services API Requests](https://docs.aws.amazon.com/general/latest/gr/signing_aws_api_requests.html) in the _Amazon Web Services General Reference_. For general information about the Query API, see [Making Query Requests](https://docs.aws.amazon.com/IAM/latest/UserGuide/IAM_UsingQueryAPI.html) in the _IAM User Guide_. For information about using security tokens with other Amazon Web Services products, see [Amazon Web Services Services That Work with IAM](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_aws-services-that-work-with-iam.html) in the _IAM User Guide_.

For information about STS endpoints, see [STS Regions and endpoints](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_credentials_temp_region-endpoints.html) in the _IAM User Guide_. For information about logging STS API calls, see [Logging IAM and STS API calls with CloudTrail](https://docs.aws.amazon.com/IAM/latest/UserGuide/cloudtrail-integration.html) in the _IAM User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-sts` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-sts = "1.115.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,ignore
use aws_sdk_sts as sts;

#[::tokio::main]
async fn main() -> Result<(), sts::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sts::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-sts/latest/aws_sdk_sts/client/struct.Client.html)
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

