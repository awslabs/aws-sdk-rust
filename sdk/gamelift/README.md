# aws-sdk-gamelift

Amazon GameLift Servers provides solutions for hosting session-based multiplayer game servers in the cloud, including tools for deploying, operating, and scaling game servers. Built on Amazon Web Services global computing infrastructure, GameLift helps you deliver high-performance, high-reliability, low-cost game servers while dynamically scaling your resource usage to meet player demand.

__About Amazon GameLift Servers solutions__

Get more information on these Amazon GameLift Servers solutions in the [Amazon GameLift Servers Developer Guide](https://docs.aws.amazon.com/gamelift/latest/developerguide/).
  - Amazon GameLift Servers managed hosting -- Amazon GameLift Servers offers a fully managed service to set up and maintain computing machines for hosting, manage game session and player session life cycle, and handle security, storage, and performance tracking. You can use automatic scaling tools to balance player demand and hosting costs, configure your game session management to minimize player latency, and add FlexMatch for matchmaking.
  - Managed hosting with Amazon GameLift Servers Realtime -- With Amazon GameLift Servers Amazon GameLift Servers Realtime, you can quickly configure and set up ready-to-go game servers for your game. Amazon GameLift Servers Realtime provides a game server framework with core Amazon GameLift Servers infrastructure already built in. Then use the full range of Amazon GameLift Servers managed hosting features, including FlexMatch, for your game.
  - Amazon GameLift Servers FleetIQ -- Use Amazon GameLift Servers FleetIQ as a standalone service while hosting your games using EC2 instances and Auto Scaling groups. Amazon GameLift Servers FleetIQ provides optimizations for game hosting, including boosting the viability of low-cost Spot Instances gaming. For a complete solution, pair the Amazon GameLift Servers FleetIQ and FlexMatch standalone services.
  - Amazon GameLift Servers FlexMatch -- Add matchmaking to your game hosting solution. FlexMatch is a customizable matchmaking service for multiplayer games. Use FlexMatch as integrated with Amazon GameLift Servers managed hosting or incorporate FlexMatch as a standalone service into your own hosting solution.

__About this API Reference__

This reference guide describes the low-level service API for Amazon GameLift Servers. With each topic in this guide, you can find links to language-specific SDK guides and the Amazon Web Services CLI reference. Useful links:
  - [Amazon GameLift Servers API operations listed by tasks](https://docs.aws.amazon.com/gamelift/latest/developerguide/reference-awssdk.html)
  - [Amazon GameLift Servers tools and resources](https://docs.aws.amazon.com/gamelift/latest/developerguide/gamelift-components.html)

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-gamelift` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-gamelift = "1.89.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_gamelift as gamelift;

#[::tokio::main]
async fn main() -> Result<(), gamelift::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_gamelift::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-gamelift/latest/aws_sdk_gamelift/client/struct.Client.html)
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

