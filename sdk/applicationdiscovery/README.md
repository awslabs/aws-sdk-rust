# aws-sdk-applicationdiscovery

Amazon Web Services Application Discovery Service (Application Discovery Service) helps you plan application migration projects. It automatically identifies servers, virtual machines (VMs), and network dependencies in your on-premises data centers. For more information, see the [Amazon Web Services Application Discovery Service FAQ](http://aws.amazon.com/application-discovery/faqs/).

Application Discovery Service offers three ways of performing discovery and collecting data about your on-premises servers:
  - __Agentless discovery__ using Amazon Web Services Application Discovery Service Agentless Collector (Agentless Collector), which doesn't require you to install an agent on each host.
    - Agentless Collector gathers server information regardless of the operating systems, which minimizes the time required for initial on-premises infrastructure assessment.
    - Agentless Collector doesn't collect information about network dependencies, only agent-based discovery collects that information.

  - __Agent-based discovery__ using the Amazon Web Services Application Discovery Agent (Application Discovery Agent) collects a richer set of data than agentless discovery, which you install on one or more hosts in your data center.
    - The agent captures infrastructure and application information, including an inventory of running processes, system performance information, resource utilization, and network dependencies.
    - The information collected by agents is secured at rest and in transit to the Application Discovery Service database in the Amazon Web Services cloud. For more information, see [Amazon Web Services Application Discovery Agent](https://docs.aws.amazon.com/application-discovery/latest/userguide/discovery-agent.html).

  - __Amazon Web Services Partner Network (APN) solutions__ integrate with Application Discovery Service, enabling you to import details of your on-premises environment directly into Amazon Web Services Migration Hub (Migration Hub) without using Agentless Collector or Application Discovery Agent.
    - Third-party application discovery tools can query Amazon Web Services Application Discovery Service, and they can write to the Application Discovery Service database using the public API.
    - In this way, you can import data into Migration Hub and view it, so that you can associate applications with servers and track migrations.

__Working With This Guide__

This API reference provides descriptions, syntax, and usage examples for each of the actions and data types for Application Discovery Service. The topic for each action shows the API request parameters and the response. Alternatively, you can use one of the Amazon Web Services SDKs to access an API that is tailored to the programming language or platform that you're using. For more information, see [Amazon Web Services SDKs](http://aws.amazon.com/tools/#SDKs).

This guide is intended for use with the [Amazon Web Services Application Discovery Service User Guide](https://docs.aws.amazon.com/application-discovery/latest/userguide/).

All data is handled according to the [Amazon Web Services Privacy Policy](https://aws.amazon.com/privacy/). You can operate Application Discovery Service offline to inspect collected data before it is shared with the service.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-applicationdiscovery` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-applicationdiscovery = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_applicationdiscovery as applicationdiscovery;

#[::tokio::main]
async fn main() -> Result<(), applicationdiscovery::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_applicationdiscovery::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-applicationdiscovery/latest/aws_sdk_applicationdiscovery/client/struct.Client.html)
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

