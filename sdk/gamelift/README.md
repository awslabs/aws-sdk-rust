# aws-sdk-gamelift

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Amazon Web Services provides solutions for hosting session-based multiplayer game servers in the cloud, including tools for deploying, operating, and scaling game servers. Built on Amazon Web Services global computing infrastructure, GameLift helps you deliver high-performance, high-reliability, low-cost game servers while dynamically scaling your resource usage to meet player demand.

__About GameLift solutions__

Get more information on these GameLift solutions in the [GameLift Developer Guide](https://docs.aws.amazon.com/gamelift/latest/developerguide/).
  - GameLift managed hosting -- GameLift offers a fully managed service to set up and maintain computing machines for hosting, manage game session and player session life cycle, and handle security, storage, and performance tracking. You can use automatic scaling tools to balance player demand and hosting costs, configure your game session management to minimize player latency, and add FlexMatch for matchmaking.
  - Managed hosting with Realtime Servers -- With GameLift Realtime Servers, you can quickly configure and set up ready-to-go game servers for your game. Realtime Servers provides a game server framework with core GameLift infrastructure already built in. Then use the full range of GameLift managed hosting features, including FlexMatch, for your game.
  - GameLift FleetIQ -- Use GameLift FleetIQ as a standalone service while hosting your games using EC2 instances and Auto Scaling groups. GameLift FleetIQ provides optimizations for game hosting, including boosting the viability of low-cost Spot Instances gaming. For a complete solution, pair the GameLift FleetIQ and FlexMatch standalone services.
  - GameLift FlexMatch -- Add matchmaking to your game hosting solution. FlexMatch is a customizable matchmaking service for multiplayer games. Use FlexMatch as integrated with GameLift managed hosting or incorporate FlexMatch as a standalone service into your own hosting solution.

__About this API Reference__

This reference guide describes the low-level service API for Amazon Web Services. With each topic in this guide, you can find links to language-specific SDK guides and the Amazon Web Services CLI reference. Useful links:
  - [GameLift API operations listed by tasks](https://docs.aws.amazon.com/gamelift/latest/developerguide/reference-awssdk.html)
  - [GameLift tools and resources](https://docs.aws.amazon.com/gamelift/latest/developerguide/gamelift-components.html)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-gamelift` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-gamelift = "0.14.0"
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

