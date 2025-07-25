# aws-sdk-marketplacemetering

This reference provides descriptions of the low-level Marketplace Metering Service API.

Amazon Web Services Marketplace sellers can use this API to submit usage data for custom usage dimensions.

For information about the permissions that you need to use this API, see [Amazon Web Services Marketplace metering and entitlement API permissions](https://docs.aws.amazon.com/marketplace/latest/userguide/iam-user-policy-for-aws-marketplace-actions.html) in the _Amazon Web Services Marketplace Seller Guide._

__Submitting metering records__

_MeterUsage_
  - Submits the metering record for an Amazon Web Services Marketplace product.
  - Called from: Amazon Elastic Compute Cloud (Amazon EC2) instance or a container running on either Amazon Elastic Kubernetes Service (Amazon EKS) or Amazon Elastic Container Service (Amazon ECS)
  - Supported product types: Amazon Machine Images (AMIs) and containers
  - Vendor-metered tagging: Supported allocation tagging

_BatchMeterUsage_
  - Submits the metering record for a set of customers. BatchMeterUsage API calls are captured by CloudTrail. You can use CloudTrail to verify that the software as a subscription (SaaS) metering records that you sent are accurate by searching for records with the eventName of BatchMeterUsage. You can also use CloudTrail to audit records over time. For more information, see the [CloudTrail User Guide](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-concepts.html).
  - Called from: SaaS applications
  - Supported product type: SaaS
  - Vendor-metered tagging: Supports allocation tagging

__Accepting new customers__

_ResolveCustomer_
  - Resolves the registration token that the buyer submits through the browser during the registration process. Obtains a CustomerIdentifier along with the CustomerAWSAccountId and ProductCode.
  - Called from: SaaS application during the registration process
  - Supported product type: SaaS
  - Vendor-metered tagging: Not applicable

__Entitlement and metering for paid container products__

_RegisteredUsage_
  - Provides software entitlement and metering. Paid container software products sold through Amazon Web Services Marketplace must integrate with the Marketplace Metering Service and call the RegisterUsage operation. Free and Bring Your Own License model (BYOL) products for Amazon ECS or Amazon EKS aren't required to call RegisterUsage. However, you can do so if you want to receive usage data in your seller reports. For more information about using the RegisterUsage operation, see [Container-based products](https://docs.aws.amazon.com/marketplace/latest/userguide/container-based-products.html).
  - Called from: Paid container software products
  - Supported product type: Containers
  - Vendor-metered tagging: Not applicable

__Entitlement custom metering for container products__
  - MeterUsage API is available in GovCloud Regions but only supports AMI FCP products in GovCloud Regions. Flexible Consumption Pricing (FCP) Container products aren’t supported in GovCloud Regions: us-gov-west-1 and us-gov-east-1. For more information, see [Container-based products](https://docs.aws.amazon.com/marketplace/latest/userguide/container-based-products.html).
  - Custom metering for container products are called using the MeterUsage API. The API is used for FCP AMI and FCP Container product metering.

__Custom metering for Amazon EKS is available in 17 Amazon Web Services Regions__
  - The metering service supports Amazon ECS and EKS for Flexible Consumption Pricing (FCP) products using MeterUsage API. Amazon ECS is supported in all Amazon Web Services Regions that MeterUsage API is available except for GovCloud.
  - Amazon EKS is supported in the following: us-east-1, us-east-2, us-west-1, us-west-2, eu-west-1, eu-central-1, eu-west-2, eu-west-3, eu-north-1, ap-east-1, ap-southeast-1, ap-northeast-1, ap-southeast-2, ap-northeast-2, ap-south-1, ca-central-1, sa-east-1.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-marketplacemetering` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-marketplacemetering = "1.80.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_marketplacemetering as marketplacemetering;

#[::tokio::main]
async fn main() -> Result<(), marketplacemetering::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_marketplacemetering::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-marketplacemetering/latest/aws_sdk_marketplacemetering/client/struct.Client.html)
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

