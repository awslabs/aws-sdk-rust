# aws-sdk-ec2

This is the _Amazon EC2 API Reference_. It provides descriptions, API request parameters, and the XML response for each of the Amazon EC2 Query API actions. Note that the Amazon EC2 API includes actions for Amazon EC2 plus additional services, such as Amazon EBS and Amazon VPC.

__Learn more__
  - To learn about using the Query API, see [Using the API for Amazon EC2](https://docs.aws.amazon.com/ec2/latest/devguide/ec2-low-level-api.html).
  - To learn about the permissions required to call an Amazon EC2 API action, see [Actions, resources, and condition keys for Amazon EC2](https://docs.aws.amazon.com/service-authorization/latest/reference/list_amazonec2.html).
  - To get the list of API actions by service and resource, see [Actions by service](https://docs.aws.amazon.com/ec2/latest/devguide/OperationList-query.html).
  - To get the alphabetical list of API actions, see .
  - To get descriptions of the API error codes, see [Error codes for the Amazon EC2 API](https://docs.aws.amazon.com/ec2/latest/devguide/errors-overview.html).

Alternatively, use one of the following methods to access the Amazon EC2 API, instead of using the Query API directly:
  - [Amazon Web Services CLI Command Reference - ec2 commands](https://docs.aws.amazon.com/cli/latest/reference/ec2/)
  - [CloudFormation - Amazon EC2 resource type reference](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/AWS_EC2.html)
  - [Amazon Web Services Tools for PowerShell Cmdlet Reference - Amazon EC2 cmdlets](https://docs.aws.amazon.com/powershell/v5/reference/items/EC2_cmdlets.html)
  - [Amazon Web Services SDKs](https://builder.aws.com/build/tools)

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ec2` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-ec2 = "1.228.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_ec2 as ec2;

#[::tokio::main]
async fn main() -> Result<(), ec2::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_ec2::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-ec2/latest/aws_sdk_ec2/client/struct.Client.html)
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

