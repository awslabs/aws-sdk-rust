# aws-sdk-sqs

Welcome to the _Amazon SQS API Reference_.

Amazon SQS is a reliable, highly-scalable hosted queue for storing messages as they travel between applications or microservices. Amazon SQS moves data between distributed application components and helps you decouple these components.

For information on the permissions you need to use this API, see [Identity and access management](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/SQSDeveloperGuide/sqs-authentication-and-access-control.html) in the _Amazon SQS Developer Guide._

You can use [Amazon Web Services SDKs](http://aws.amazon.com/tools/#sdk) to access Amazon SQS using your favorite programming language. The SDKs perform tasks such as the following automatically:
  - Cryptographically sign your service requests
  - Retry requests
  - Handle error responses

__Additional information__
  - [Amazon SQS Product Page](http://aws.amazon.com/sqs/)
  - _Amazon SQS Developer Guide_
    - [Making API Requests](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/SQSDeveloperGuide/sqs-making-api-requests.html)
    - [Amazon SQS Message Attributes](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/SQSDeveloperGuide/sqs-message-metadata.html#sqs-message-attributes)
    - [Amazon SQS Dead-Letter Queues](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/SQSDeveloperGuide/sqs-dead-letter-queues.html)

  - [Amazon SQS in the _Command Line Interface_](http://docs.aws.amazon.com/cli/latest/reference/sqs/index.html)
  - _Amazon Web Services General Reference_
    - [Regions and Endpoints](https://docs.aws.amazon.com/general/latest/gr/rande.html#sqs_region)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-sqs` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-sqs = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_sqs as sqs;

#[::tokio::main]
async fn main() -> Result<(), sqs::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sqs::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-sqs/latest/aws_sdk_sqs/client/struct.Client.html)
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

