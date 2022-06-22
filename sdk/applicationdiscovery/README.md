# aws-sdk-applicationdiscovery

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Web Services Application Discovery Service helps you plan application migration projects. It automatically identifies servers, virtual machines (VMs), and network dependencies in your on-premises data centers. For more information, see the [Amazon Web Services Application Discovery Service FAQ](http://aws.amazon.com/application-discovery/faqs/). Application Discovery Service offers three ways of performing discovery and collecting data about your on-premises servers:
  - __Agentless discovery__ is recommended for environments that use VMware vCenter Server. This mode doesn't require you to install an agent on each host. It does not work in non-VMware environments.
    - Agentless discovery gathers server information regardless of the operating systems, which minimizes the time required for initial on-premises infrastructure assessment.
    - Agentless discovery doesn't collect information about network dependencies, only agent-based discovery collects that information.

  - __Agent-based discovery__ collects a richer set of data than agentless discovery by using the Amazon Web Services Application Discovery Agent, which you install on one or more hosts in your data center.
    - The agent captures infrastructure and application information, including an inventory of running processes, system performance information, resource utilization, and network dependencies.
    - The information collected by agents is secured at rest and in transit to the Application Discovery Service database in the cloud.

  - __Amazon Web Services Partner Network (APN) solutions__ integrate with Application Discovery Service, enabling you to import details of your on-premises environment directly into Migration Hub without using the discovery connector or discovery agent.
    - Third-party application discovery tools can query Amazon Web Services Application Discovery Service, and they can write to the Application Discovery Service database using the public API.
    - In this way, you can import data into Migration Hub and view it, so that you can associate applications with servers and track migrations.

__Recommendations__

We recommend that you use agent-based discovery for non-VMware environments, and whenever you want to collect information about network dependencies. You can run agent-based and agentless discovery simultaneously. Use agentless discovery to complete the initial infrastructure assessment quickly, and then install agents on select hosts to collect additional information.

__Working With This Guide__

This API reference provides descriptions, syntax, and usage examples for each of the actions and data types for Application Discovery Service. The topic for each action shows the API request parameters and the response. Alternatively, you can use one of the Amazon Web Services SDKs to access an API that is tailored to the programming language or platform that you're using. For more information, see [Amazon Web Services SDKs](http://aws.amazon.com/tools/#SDKs).

This guide is intended for use with the [Amazon Web Services Application Discovery Service User Guide](http://docs.aws.amazon.com/application-discovery/latest/userguide/).

All data is handled according to the [Amazon Web Services Privacy Policy](http://aws.amazon.com/privacy/). You can operate Application Discovery Service offline to inspect collected data before it is shared with the service.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-applicationdiscovery` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-applicationdiscovery = "0.14.0"
tokio = { version = "1", features = ["full"] }
```

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Developer Guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) – For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

