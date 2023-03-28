# aws-sdk-codecatalyst

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Welcome to the Amazon CodeCatalyst API reference. This reference provides descriptions of operations and data types for Amazon CodeCatalyst. You can use the Amazon CodeCatalyst API to work with the following objects.

Dev Environments and the Amazon Web Services Toolkits, by calling the following:
  - CreateAccessToken, which creates a personal access token (PAT) for the current user.
  - CreateDevEnvironment, which creates a Dev Environment, where you can quickly work on the code stored in the source repositories of your project.
  - CreateProject which creates a project in a specified space.
  - CreateSourceRepositoryBranch, which creates a branch in a specified repository where you can work on code.
  - DeleteDevEnvironment, which deletes a Dev Environment.
  - GetDevEnvironment, which returns information about a Dev Environment.
  - GetProject, which returns information about a project.
  - GetSourceRepositoryCloneUrls, which returns information about the URLs that can be used with a Git client to clone a source repository.
  - GetSubscription, which returns information about the Amazon Web Services account used for billing purposes and the billing plan for the space.
  - GetUserDetails, which returns information about a user in Amazon CodeCatalyst.
  - ListDevEnvironments, which retrives a list of Dev Environments in a project.
  - ListProjects, which retrieves a list of projects in a space.
  - ListSourceRepositories, which retrieves a list of source repositories in a project.
  - ListSourceRepositoryBranches, which retrieves a list of branches in a source repository.
  - ListSpaces, which retrieves a list of spaces.
  - StartDevEnvironment, which starts a specified Dev Environment and puts it into an active state.
  - StartDevEnvironmentSession, which starts a session to a specified Dev Environment.
  - StopDevEnvironment, which stops a specified Dev Environment and puts it into an stopped state.
  - UpdateDevEnvironment, which changes one or more values for a Dev Environment.
  - VerifySession, which verifies whether the calling user has a valid Amazon CodeCatalyst login and session.

Security, activity, and resource management in Amazon CodeCatalyst, by calling the following:
  - DeleteAccessToken, which deletes a specified personal access token (PAT).
  - ListAccessTokens, which lists all personal access tokens (PATs) associated with a user.
  - ListEventLogs, which retrieves a list of events that occurred during a specified time period in a space.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-codecatalyst` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.0.0-smithy-rs-head"
aws-sdk-codecatalyst = "0.12.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust
use aws_sdk_codecatalyst as codecatalyst;

#[tokio::main]
async fn main() -> Result<(), codecatalyst::Error> {
    let config = aws_config::load_from_env().await;
    let client = codecatalyst::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-codecatalyst/latest/aws_sdk_codecatalyst/client/struct.Client.html)
for information on what calls can be made, and the inputs and outputs for each of those calls.

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

