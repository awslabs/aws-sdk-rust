# aws-sdk-networksecuritymanager

AWS Network Security Manager helps you centrally configure and deploy network security protections across your organization. Supported protections include AWS WAF and AWS Shield Advanced. This centralized approach reduces the overhead of managing protections individually across accounts and ensures consistent security at scale.

You define reusable _rules_ and _templates_, then combine them into _policies_. Next, you select the accounts and resources to protect with _scopes_ and roll the protections out with _deployments_. For example, you can define a set of AWS WAF rules and group them into a policy. Then deploy that policy across all accounts in your organization with a single deployment.

This API reference describes the operations and data types for AWS Network Security Manager.

For conceptual information, tutorials, and guidance on writing rule configurations, see the [AWS Network Security Manager Developer Guide](https://docs.aws.amazon.com/network-security-manager/latest/devguide/what-is.html). For the default quotas that apply to your account, see [Quotas](https://docs.aws.amazon.com/network-security-manager/latest/devguide/quotas.html). For the service endpoints available in each Region, see [AWS Network Security Manager endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/network-security-manager.html) in the _AWS General Reference_.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-networksecuritymanager` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-networksecuritymanager = "1.1.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_networksecuritymanager as networksecuritymanager;

#[::tokio::main]
async fn main() -> Result<(), networksecuritymanager::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_networksecuritymanager::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-networksecuritymanager/latest/aws_sdk_networksecuritymanager/client/struct.Client.html)
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

