# aws-sdk-pricing

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Web Services Price List Service API (Amazon Web Services Price List Service) is a centralized and convenient way to programmatically query Amazon Web Services for services, products, and pricing information. The Amazon Web Services Price List Service uses standardized product attributes such as Location, Storage Class, and Operating System, and provides prices at the SKU level. You can use the Amazon Web Services Price List Service to build cost control and scenario planning tools, reconcile billing data, forecast future spend for budgeting purposes, and provide cost benefit analysis that compare your internal workloads with Amazon Web Services.

Use GetServices without a service code to retrieve the service codes for all AWS services, then GetServices with a service code to retrieve the attribute names for that service. After you have the service code and attribute names, you can use GetAttributeValues to see what values are available for an attribute. With the service code and an attribute name and value, you can use GetProducts to find specific products that you're interested in, such as an AmazonEC2 instance, with a Provisioned IOPS volumeType.

Service Endpoint

Amazon Web Services Price List Service API provides the following two endpoints:
  - https://api.pricing.us-east-1.amazonaws.com
  - https://api.pricing.ap-south-1.amazonaws.com

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-pricing` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-pricing = "0.14.0"
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

