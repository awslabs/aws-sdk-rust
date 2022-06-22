# aws-sdk-ec2

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Elastic Compute Cloud (Amazon EC2) provides secure and resizable computing capacity in the Amazon Web Services Cloud. Using Amazon EC2 eliminates the need to invest in hardware up front, so you can develop and deploy applications faster. Amazon Virtual Private Cloud (Amazon VPC) enables you to provision a logically isolated section of the Amazon Web Services Cloud where you can launch Amazon Web Services resources in a virtual network that you've defined. Amazon Elastic Block Store (Amazon EBS) provides block level storage volumes for use with EC2 instances. EBS volumes are highly available and reliable storage volumes that can be attached to any running instance and used like a hard drive.

To learn more, see the following resources:
  - Amazon EC2: [AmazonEC2 product page](http://aws.amazon.com/ec2), [Amazon EC2 documentation](http://aws.amazon.com/documentation/ec2)
  - Amazon EBS: [Amazon EBS product page](http://aws.amazon.com/ebs), [Amazon EBS documentation](http://aws.amazon.com/documentation/ebs)
  - Amazon VPC: [Amazon VPC product page](http://aws.amazon.com/vpc), [Amazon VPC documentation](http://aws.amazon.com/documentation/vpc)
  - Amazon Web Services VPN: [Amazon Web Services VPN product page](http://aws.amazon.com/vpn), [Amazon Web Services VPN documentation](http://aws.amazon.com/documentation/vpn)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ec2` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-ec2 = "0.14.0"
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

