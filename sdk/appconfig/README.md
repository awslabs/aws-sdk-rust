# aws-sdk-appconfig

AppConfig helps you safely change application behavior in production without redeploying code. Using feature flags and dynamic free-form configurations, you can control how your application runs in real time. This approach reduces risk, accelerates releases, and enables faster responses to issues. You can gradually roll out new features to specific users, monitor their impact, and expand availability with confidence. You can also update block lists, allow lists, throttling limits, and logging levels instantly, allowing you to mitigate issues and fine-tune performance without a deployment.

AppConfig supports a broad spectrum of use cases:
  - __Feature flags and toggles__ – Gradually release new capabilities to targeted users, monitor impact, and instantly roll back changes if issues occur.
  - __Application tuning__ – Introduce changes safely in production, measure their effects, and refine behavior without redeploying code.
  - __Allow list or block list__ – Control access to features or restrict specific users in real time, without modifying application code.
  - __Centralized configuration storage__ – Manage configuration data consistently across workloads. AppConfig can deploy configuration from the AppConfig hosted configuration store, Secrets Manager, Systems Manager, Systems Manager Parameter Store, or Amazon S3.

__How AppConfig works__

This section provides a high-level description of how AppConfig works and how you get started.

__1. Identify configuration data to manage in AppConfig__

Before creating a configuration profile, identify the configuration data in your code that you want to manage dynamically using AppConfig. Common examples include feature flags, allow and block lists, logging levels, service limits, and throttling rules. These values tend to change frequently and can cause issues if misconfigured. If your configuration data already exists in cloud services such as Systems Manager Parameter Store or Amazon S3, you can use AppConfig to validate, deploy, and manage that data more effectively.

__2. Create a configuration profile in AppConfig__

A configuration profile defines how AppConfig locates and manages your configuration data. It includes a URI that points to the data source and a profile type. AppConfig supports two profile types   - __Feature flags__ – Enable controlled feature releases, gradual rollouts, and testing in production.
  - __Free-form configurations__ – Store and retrieve configuration data from external sources and update it without redeploying code.
Both profile types help decouple configuration from code, support continuous delivery, and reduce deployment risk. You can also add optional validators to ensure that configuration data is syntactically and semantically correct. During deployment, AppConfig evaluates these validators and automatically rolls back changes if validation fails. Each configuration profile is associated with an application, which acts as a logical container for your configuration resources. For more information about creating a configuration profile, see [Creating a configuration profile in AppConfig](http://docs.aws.amazon.com/appconfig/latest/userguide/appconfig-creating-configuration-profile.html) in the the _AppConfig User Guide_.

__3. Deploy configuration data__

When you start a deployment, AppConfig:   1. Retrieves configuration data from the source defined in the configuration profile
  1. Validates the data using the configured validators
  1. Delivers the validated configuration to AppConfig Agent
The delivered configuration becomes the deployed version used by your application. For more information about deploying a configuration, see [Deploying feature flags and configuration data in AppConfig](http://docs.aws.amazon.com/appconfig/latest/userguide/deploying-feature-flags.html).

__4. Retrieve configuration data__

Your application retrieves configuration data by calling a local endpoint exposed by AppConfig Agent, which caches the deployed configuration. Retrieving data is a metered event. AppConfig Agent supports a variety of use cases, as described in [How to use AppConfig Agent to retrieve configuration data](http://docs.aws.amazon.com/appconfig/latest/userguide/appconfig-agent-how-to-use.html). If the agent is not suitable for your use case, your application can retrieve configuration data directly from AppConfig by calling the [StartConfigurationSession](https://docs.aws.amazon.com/appconfig/2019-10-09/APIReference/API_appconfigdata_StartConfigurationSession.html) and [GetLatestConfiguration](https://docs.aws.amazon.com/appconfig/2019-10-09/APIReference/API_appconfigdata_GetLatestConfiguration.html) API actions. For more information about retrieving a configuration, see [Retrieving feature flags and configuration data in AppConfig](http://docs.aws.amazon.com/appconfig/latest/userguide/retrieving-feature-flags.html).


This reference is intended to be used with the [AppConfig User Guide](http://docs.aws.amazon.com/appconfig/latest/userguide/what-is-appconfig.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-appconfig` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-appconfig = "1.108.0"
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
* [Usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1)

## License

This project is licensed under the Apache-2.0 License.

