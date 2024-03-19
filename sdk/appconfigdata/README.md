# aws-sdk-appconfigdata

AppConfig Data provides the data plane APIs your application uses to retrieve configuration data. Here's how it works:

Your application retrieves configuration data by first establishing a configuration session using the AppConfig Data StartConfigurationSession API action. Your session's client then makes periodic calls to GetLatestConfiguration to check for and retrieve the latest data available.

When calling StartConfigurationSession, your code sends the following information:
  - Identifiers (ID or name) of an AppConfig application, environment, and configuration profile that the session tracks.
  - (Optional) The minimum amount of time the session's client must wait between calls to GetLatestConfiguration.

In response, AppConfig provides an InitialConfigurationToken to be given to the session's client and used the first time it calls GetLatestConfiguration for that session.

This token should only be used once in your first call to GetLatestConfiguration. You _must_ use the new token in the GetLatestConfiguration response (NextPollConfigurationToken) in each subsequent call to GetLatestConfiguration.

When calling GetLatestConfiguration, your client code sends the most recent ConfigurationToken value it has and receives in response:
  - NextPollConfigurationToken: the ConfigurationToken value to use on the next call to GetLatestConfiguration.
  - NextPollIntervalInSeconds: the duration the client should wait before making its next call to GetLatestConfiguration. This duration may vary over the course of the session, so it should be used instead of the value sent on the StartConfigurationSession call.
  - The configuration: the latest data intended for the session. This may be empty if the client already has the latest version of the configuration.

The InitialConfigurationToken and NextPollConfigurationToken should only be used once. To support long poll use cases, the tokens are valid for up to 24 hours. If a GetLatestConfiguration call uses an expired token, the system returns BadRequestException.

For more information and to view example CLI commands that show how to retrieve a configuration using the AppConfig Data StartConfigurationSession and GetLatestConfiguration API actions, see [Retrieving the configuration](http://docs.aws.amazon.com/appconfig/latest/userguide/appconfig-retrieving-the-configuration) in the _AppConfig User Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-appconfigdata` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-appconfigdata = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_appconfigdata as appconfigdata;

#[::tokio::main]
async fn main() -> Result<(), appconfigdata::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_appconfigdata::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-appconfigdata/latest/aws_sdk_appconfigdata/client/struct.Client.html)
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

