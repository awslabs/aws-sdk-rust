# aws-sdk-marketplacemetering

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

This reference provides descriptions of the low-level AWS Marketplace Metering Service API.

AWS Marketplace sellers can use this API to submit usage data for custom usage dimensions.

For information on the permissions you need to use this API, see [AWS Marketplace metering and entitlement API permissions](https://docs.aws.amazon.com/marketplace/latest/userguide/iam-user-policy-for-aws-marketplace-actions.html) in the _AWS Marketplace Seller Guide._

__Submitting Metering Records__
  - _MeterUsage_ - Submits the metering record for an AWS Marketplace product. MeterUsage is called from an EC2 instance or a container running on EKS or ECS.
  - _BatchMeterUsage_ - Submits the metering record for a set of customers. BatchMeterUsage is called from a software-as-a-service (SaaS) application.

__Accepting New Customers__
  - _ResolveCustomer_ - Called by a SaaS application during the registration process. When a buyer visits your website during the registration process, the buyer submits a Registration Token through the browser. The Registration Token is resolved through this API to obtain a CustomerIdentifier along with the CustomerAWSAccountId and ProductCode.

__Entitlement and Metering for Paid Container Products__
  - Paid container software products sold through AWS Marketplace must integrate with the AWS Marketplace Metering Service and call the RegisterUsage operation for software entitlement and metering. Free and BYOL products for Amazon ECS or Amazon EKS aren't required to call RegisterUsage, but you can do so if you want to receive usage data in your seller reports. For more information on using the RegisterUsage operation, see [Container-Based Products](https://docs.aws.amazon.com/marketplace/latest/userguide/container-based-products.html).

BatchMeterUsage API calls are captured by AWS CloudTrail. You can use Cloudtrail to verify that the SaaS metering records that you sent are accurate by searching for records with the eventName of BatchMeterUsage. You can also use CloudTrail to audit records over time. For more information, see the _ [AWS CloudTrail User Guide](http://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-concepts.html)._

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-marketplacemetering` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-marketplacemetering = "0.14.0"
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

