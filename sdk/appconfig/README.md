# aws-sdk-appconfig

AppConfig feature flags and dynamic configurations help software builders quickly and securely adjust application behavior in production environments without full code deployments. AppConfig speeds up software release frequency, improves application resiliency, and helps you address emergent issues more quickly. With feature flags, you can gradually release new capabilities to users and measure the impact of those changes before fully deploying the new capabilities to all users. With operational flags and dynamic configurations, you can update block lists, allow lists, throttling limits, logging verbosity, and perform other operational tuning to quickly respond to issues in production environments.

Despite the fact that application configuration content can vary greatly from application to application, AppConfig supports the following use cases, which cover a broad spectrum of customer needs:
  - __Feature flags and toggles__ - Safely release new capabilities to your customers in a controlled environment. Instantly roll back changes if you experience a problem.
  - __Application tuning__ - Carefully introduce application changes while testing the impact of those changes with users in production environments.
  - __Allow list or block list__ - Control access to premium features or instantly block specific users without deploying new code.
  - __Centralized configuration storage__ - Keep your configuration data organized and consistent across all of your workloads. You can use AppConfig to deploy configuration data stored in the AppConfig hosted configuration store, Secrets Manager, Systems Manager, Parameter Store, or Amazon S3.

__How AppConfig works__

This section provides a high-level description of how AppConfig works and how you get started.

__1. Identify configuration values in code you want to manage in the cloud__

Before you start creating AppConfig artifacts, we recommend you identify configuration data in your code that you want to dynamically manage using AppConfig. Good examples include feature flags or toggles, allow and block lists, logging verbosity, service limits, and throttling rules, to name a few. If your configuration data already exists in the cloud, you can take advantage of AppConfig validation, deployment, and extension features to further streamline configuration data management.

__2. Create an application namespace__

To create a namespace, you create an AppConfig artifact called an application. An application is simply an organizational construct like a folder.

__3. Create environments__

For each AppConfig application, you define one or more environments. An environment is a logical grouping of targets, such as applications in a Beta or Production environment, Lambda functions, or containers. You can also define environments for application subcomponents, such as the Web, Mobile, and Back-end. You can configure Amazon CloudWatch alarms for each environment. The system monitors alarms during a configuration deployment. If an alarm is triggered, the system rolls back the configuration.

__4. Create a configuration profile__

A configuration profile includes, among other things, a URI that enables AppConfig to locate your configuration data in its stored location and a profile type. AppConfig supports two configuration profile types: feature flags and freeform configurations. Feature flag configuration profiles store their data in the AppConfig hosted configuration store and the URI is simply hosted. For freeform configuration profiles, you can store your data in the AppConfig hosted configuration store or any Amazon Web Services service that integrates with AppConfig, as described in [Creating a free form configuration profile](http://docs.aws.amazon.com/appconfig/latest/userguide/appconfig-free-form-configurations-creating.html) in the the _AppConfig User Guide_. A configuration profile can also include optional validators to ensure your configuration data is syntactically and semantically correct. AppConfig performs a check using the validators when you start a deployment. If any errors are detected, the deployment rolls back to the previous configuration data.

__5. Deploy configuration data__

When you create a new deployment, you specify the following:   - An application ID
  - A configuration profile ID
  - A configuration version
  - An environment ID where you want to deploy the configuration data
  - A deployment strategy ID that defines how fast you want the changes to take effect
When you call the [StartDeployment](https://docs.aws.amazon.com/appconfig/2019-10-09/APIReference/API_StartDeployment.html) API action, AppConfig performs the following tasks:   1. Retrieves the configuration data from the underlying data store by using the location URI in the configuration profile.
  1. Verifies the configuration data is syntactically and semantically correct by using the validators you specified when you created your configuration profile.
  1. Caches a copy of the data so it is ready to be retrieved by your application. This cached copy is called the _deployed data_.


__6. Retrieve the configuration__

You can configure AppConfig Agent as a local host and have the agent poll AppConfig for configuration updates. The agent calls the [StartConfigurationSession](https://docs.aws.amazon.com/appconfig/2019-10-09/APIReference/API_appconfigdata_StartConfigurationSession.html) and [GetLatestConfiguration](https://docs.aws.amazon.com/appconfig/2019-10-09/APIReference/API_appconfigdata_GetLatestConfiguration.html) API actions and caches your configuration data locally. To retrieve the data, your application makes an HTTP call to the localhost server. AppConfig Agent supports several use cases, as described in [Simplified retrieval methods](http://docs.aws.amazon.com/appconfig/latest/userguide/appconfig-retrieving-simplified-methods.html) in the the _AppConfig User Guide_. If AppConfig Agent isn't supported for your use case, you can configure your application to poll AppConfig for configuration updates by directly calling the [StartConfigurationSession](https://docs.aws.amazon.com/appconfig/2019-10-09/APIReference/API_appconfigdata_StartConfigurationSession.html) and [GetLatestConfiguration](https://docs.aws.amazon.com/appconfig/2019-10-09/APIReference/API_appconfigdata_GetLatestConfiguration.html) API actions.


This reference is intended to be used with the [AppConfig User Guide](http://docs.aws.amazon.com/appconfig/latest/userguide/what-is-appconfig.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-appconfig` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-appconfig = "0.0.0-local"
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

