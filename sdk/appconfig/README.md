# aws-sdk-appconfig

Use AppConfig, a capability of Amazon Web Services Systems Manager, to create, manage, and quickly deploy application configurations. AppConfig supports controlled deployments to applications of any size and includes built-in validation checks and monitoring. You can use AppConfig with applications hosted on Amazon EC2 instances, Lambda, containers, mobile applications, or IoT devices.

To prevent errors when deploying application configurations, especially for production systems where a simple typo could cause an unexpected outage, AppConfig includes validators. A validator provides a syntactic or semantic check to ensure that the configuration you want to deploy works as intended. To validate your application configuration data, you provide a schema or an Amazon Web Services Lambda function that runs against the configuration. The configuration deployment or update can only proceed when the configuration data is valid.

During a configuration deployment, AppConfig monitors the application to ensure that the deployment is successful. If the system encounters an error, AppConfig rolls back the change to minimize impact for your application users. You can configure a deployment strategy for each application or environment that includes deployment criteria, including velocity, bake time, and alarms to monitor. Similar to error monitoring, if a deployment triggers an alarm, AppConfig automatically rolls back to the previous version.

AppConfig supports multiple use cases. Here are some examples:
  - __Feature flags__: Use AppConfig to turn on new features that require a timely deployment, such as a product launch or announcement.
  - __Application tuning__: Use AppConfig to carefully introduce changes to your application that can only be tested with production traffic.
  - __Allow list__: Use AppConfig to allow premium subscribers to access paid content.
  - __Operational issues__: Use AppConfig to reduce stress on your application when a dependency or other external factor impacts the system.

This reference is intended to be used with the [AppConfig User Guide](http://docs.aws.amazon.com/appconfig/latest/userguide/what-is-appconfig.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-appconfig` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-appconfig = "1.12.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_appconfig as appconfig;

#[::tokio::main]
async fn main() -> Result<(), appconfig::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_appconfig::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-appconfig/latest/aws_sdk_appconfig/client/struct.Client.html)
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

