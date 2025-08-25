# aws-sdk-braket

The Amazon Braket API Reference provides information about the operations and structures supported by Amazon Braket.

To learn about the permissions required to call an Amazon Braket API action, see [Actions, resources, and condition keys for Amazon Braket](https://docs.aws.amazon.com/service-authorization/latest/reference/list_amazonbraket.html). [Amazon Braket Python SDK](https://amazon-braket-sdk-python.readthedocs.io/en/latest/#) and the [AWS Command Line Interface](https://docs.aws.amazon.com/cli/latest/reference/braket/) can be used to make discovery and creation of API calls easier. For more information about Amazon Braket features, see [What is Amazon Braket?](https://docs.aws.amazon.com/braket/latest/developerguide/what-is-braket.html) and important [terms and concepts](https://docs.aws.amazon.com/braket/latest/developerguide/braket-terms.html) in the _Amazon Braket Developer Guide_.

__In this guide:__
  -
  -
  - CommonParameters
  - CommonErrors

__Available languages for AWS SDK:__
  - [.NET](https://docs.aws.amazon.com/sdkfornet/v3/apidocs/items/Braket/NBraket.html)
  - [C++](https://sdk.amazonaws.com/cpp/api/LATEST/root/html/index.html)
  - [Go API reference](https://docs.aws.amazon.com/sdk-for-go/api/service/braket/)
  - [Java](https://docs.aws.amazon.com/AWSJavaSDK/latest/javadoc/com/amazonaws/services/braket/package-summary.html)
  - [JavaScript](https://docs.aws.amazon.com/AWSJavaScriptSDK/latest/AWS/Braket.html)
  - [PHP](https://docs.aws.amazon.com/aws-sdk-php/v3/api/class-Aws.Braket.BraketClient.html)
  - [Python (Boto)](https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/braket.html)
  - [Ruby](https://docs.aws.amazon.com/sdk-for-ruby/v3/api/Aws/Braket.html)

__Code examples from the Amazon Braket Tutorials GitHub repository:__
  - [Amazon Braket Examples](https://github.com/amazon-braket/amazon-braket-examples)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-braket` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-braket = "1.85.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_braket as braket;

#[::tokio::main]
async fn main() -> Result<(), braket::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_braket::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-braket/latest/aws_sdk_braket/client/struct.Client.html)
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

