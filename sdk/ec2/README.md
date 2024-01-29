# aws-sdk-ec2

Amazon Elastic Compute Cloud (Amazon EC2) provides secure and resizable computing capacity in the Amazon Web Services Cloud. Using Amazon EC2 eliminates the need to invest in hardware up front, so you can develop and deploy applications faster. Amazon Virtual Private Cloud (Amazon VPC) enables you to provision a logically isolated section of the Amazon Web Services Cloud where you can launch Amazon Web Services resources in a virtual network that you've defined. Amazon Elastic Block Store (Amazon EBS) provides block level storage volumes for use with EC2 instances. EBS volumes are highly available and reliable storage volumes that can be attached to any running instance and used like a hard drive.

To learn more, see the following resources:
  - Amazon EC2: [Amazon EC2 product page](http://aws.amazon.com/ec2), [Amazon EC2 documentation](https://docs.aws.amazon.com/ec2/index.html)
  - Amazon EBS: [Amazon EBS product page](http://aws.amazon.com/ebs), [Amazon EBS documentation](https://docs.aws.amazon.com/ebs/index.html)
  - Amazon VPC: [Amazon VPC product page](http://aws.amazon.com/vpc), [Amazon VPC documentation](https://docs.aws.amazon.com/vpc/index.html)
  - VPN: [VPN product page](http://aws.amazon.com/vpn), [VPN documentation](https://docs.aws.amazon.com/vpn/index.html)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ec2` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-ec2 = "1.18.0"
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
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

