# aws-sdk-socialmessaging

_Amazon Web Services End User Messaging Social_, also referred to as Social messaging, is a messaging service that enables application developers to incorporate WhatsApp into their existing workflows. The _Amazon Web Services End User Messaging Social API_ provides information about the _Amazon Web Services End User Messaging Social API_ resources, including supported HTTP methods, parameters, and schemas.

The _Amazon Web Services End User Messaging Social API_ provides programmatic access to options that are unique to the WhatsApp Business Platform.

If you're new to the _Amazon Web Services End User Messaging Social API_, it's also helpful to review [What is Amazon Web Services End User Messaging Social](https://docs.aws.amazon.com/sms-voice/latest/userguide/what-is-service.html) in the _Amazon Web Services End User Messaging Social User Guide_. The _Amazon Web Services End User Messaging Social User Guide_ provides tutorials, code samples, and procedures that demonstrate how to use _Amazon Web Services End User Messaging Social API_ features programmatically and how to integrate functionality into applications. The guide also provides key information, such as integration with other Amazon Web Services services, and the quotas that apply to use of the service.

__Regional availability__

The _Amazon Web Services End User Messaging Social API_ is available across several Amazon Web Services Regions and it provides a dedicated endpoint for each of these Regions. For a list of all the Regions and endpoints where the API is currently available, see [Amazon Web Services Service Endpoints](https://docs.aws.amazon.com/general/latest/gr/rande.html#pinpoint_region) and [Amazon Web Services End User Messaging endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/end-user-messaging.html) in the Amazon Web Services General Reference. To learn more about Amazon Web Services Regions, see [Managing Amazon Web Services Regions](https://docs.aws.amazon.com/general/latest/gr/rande-manage.html) in the Amazon Web Services General Reference.

In each Region, Amazon Web Services maintains multiple Availability Zones. These Availability Zones are physically isolated from each other, but are united by private, low-latency, high-throughput, and highly redundant network connections. These Availability Zones enable us to provide very high levels of availability and redundancy, while also minimizing latency. To learn more about the number of Availability Zones that are available in each Region, see [Amazon Web Services Global Infrastructure.](https://aws.amazon.com/about-aws/global-infrastructure/)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-socialmessaging` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-socialmessaging = "1.37.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_socialmessaging as socialmessaging;

#[::tokio::main]
async fn main() -> Result<(), socialmessaging::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_socialmessaging::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-socialmessaging/latest/aws_sdk_socialmessaging/client/struct.Client.html)
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

