# aws-sdk-pinpointsmsvoicev2

Welcome to the _Amazon Pinpoint SMS and Voice, version 2 API Reference_. This guide provides information about Amazon Pinpoint SMS and Voice, version 2 API resources, including supported HTTP methods, parameters, and schemas.

Amazon Pinpoint is an Amazon Web Services service that you can use to engage with your recipients across multiple messaging channels. The Amazon Pinpoint SMS and Voice, version 2 API provides programmatic access to options that are unique to the SMS and voice channels and supplements the resources provided by the Amazon Pinpoint API.

If you're new to Amazon Pinpoint, it's also helpful to review the [Amazon Pinpoint Developer Guide](https://docs.aws.amazon.com/pinpoint/latest/developerguide/welcome.html). The _Amazon Pinpoint Developer Guide_ provides tutorials, code samples, and procedures that demonstrate how to use Amazon Pinpoint features programmatically and how to integrate Amazon Pinpoint functionality into mobile apps and other types of applications. The guide also provides key information, such as Amazon Pinpoint integration with other Amazon Web Services services, and the quotas that apply to use of the service.

__Regional availability__

The _Amazon Pinpoint SMS and Voice, version 2 API Reference_ is available in several Amazon Web Services Regions and it provides an endpoint for each of these Regions. For a list of all the Regions and endpoints where the API is currently available, see [Amazon Web Services Service Endpoints](https://docs.aws.amazon.com/general/latest/gr/rande.html#pinpoint_region) and [Amazon Pinpoint endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/pinpoint.html) in the Amazon Web Services General Reference. To learn more about Amazon Web Services Regions, see [Managing Amazon Web Services Regions](https://docs.aws.amazon.com/general/latest/gr/rande-manage.html) in the Amazon Web Services General Reference.

In each Region, Amazon Web Services maintains multiple Availability Zones. These Availability Zones are physically isolated from each other, but are united by private, low-latency, high-throughput, and highly redundant network connections. These Availability Zones enable us to provide very high levels of availability and redundancy, while also minimizing latency. To learn more about the number of Availability Zones that are available in each Region, see [Amazon Web Services Global Infrastructure.](https://aws.amazon.com/about-aws/global-infrastructure/)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-pinpointsmsvoicev2` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-pinpointsmsvoicev2 = "1.12.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_pinpointsmsvoicev2 as pinpointsmsvoicev2;

#[::tokio::main]
async fn main() -> Result<(), pinpointsmsvoicev2::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_pinpointsmsvoicev2::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-pinpointsmsvoicev2/latest/aws_sdk_pinpointsmsvoicev2/client/struct.Client.html)
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

