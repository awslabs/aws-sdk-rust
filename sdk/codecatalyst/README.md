# aws-sdk-codecatalyst

Welcome to the Amazon CodeCatalyst API reference. This reference provides descriptions of operations and data types for Amazon CodeCatalyst. You can use the Amazon CodeCatalyst API to work with the following objects.

Spaces, by calling the following:
  - DeleteSpace, which deletes a space.
  - GetSpace, which returns information about a space.
  - GetSubscription, which returns information about the Amazon Web Services account used for billing purposes and the billing plan for the space.
  - ListSpaces, which retrieves a list of spaces.
  - UpdateSpace, which changes one or more values for a space.

Projects, by calling the following:
  - CreateProject which creates a project in a specified space.
  - GetProject, which returns information about a project.
  - ListProjects, which retrieves a list of projects in a space.

Users, by calling the following:
  - GetUserDetails, which returns information about a user in Amazon CodeCatalyst.

Source repositories, by calling the following:
  - CreateSourceRepository, which creates an empty Git-based source repository in a specified project.
  - CreateSourceRepositoryBranch, which creates a branch in a specified repository where you can work on code.
  - DeleteSourceRepository, which deletes a source repository.
  - GetSourceRepository, which returns information about a source repository.
  - GetSourceRepositoryCloneUrls, which returns information about the URLs that can be used with a Git client to clone a source repository.
  - ListSourceRepositories, which retrieves a list of source repositories in a project.
  - ListSourceRepositoryBranches, which retrieves a list of branches in a source repository.

Dev Environments and the Amazon Web Services Toolkits, by calling the following:
  - CreateDevEnvironment, which creates a Dev Environment, where you can quickly work on the code stored in the source repositories of your project.
  - DeleteDevEnvironment, which deletes a Dev Environment.
  - GetDevEnvironment, which returns information about a Dev Environment.
  - ListDevEnvironments, which retrieves a list of Dev Environments in a project.
  - ListDevEnvironmentSessions, which retrieves a list of active Dev Environment sessions in a project.
  - StartDevEnvironment, which starts a specified Dev Environment and puts it into an active state.
  - StartDevEnvironmentSession, which starts a session to a specified Dev Environment.
  - StopDevEnvironment, which stops a specified Dev Environment and puts it into an stopped state.
  - StopDevEnvironmentSession, which stops a session for a specified Dev Environment.
  - UpdateDevEnvironment, which changes one or more values for a Dev Environment.

Workflows, by calling the following:
  - GetWorkflow, which returns information about a workflow.
  - GetWorkflowRun, which returns information about a specified run of a workflow.
  - ListWorkflowRuns, which retrieves a list of runs of a specified workflow.
  - ListWorkflows, which retrieves a list of workflows in a specified project.
  - StartWorkflowRun, which starts a run of a specified workflow.

Security, activity, and resource management in Amazon CodeCatalyst, by calling the following:
  - CreateAccessToken, which creates a personal access token (PAT) for the current user.
  - DeleteAccessToken, which deletes a specified personal access token (PAT).
  - ListAccessTokens, which lists all personal access tokens (PATs) associated with a user.
  - ListEventLogs, which retrieves a list of events that occurred during a specified time period in a space.
  - VerifySession, which verifies whether the calling user has a valid Amazon CodeCatalyst login and session.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-codecatalyst` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-codecatalyst = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_codecatalyst as codecatalyst;

#[::tokio::main]
async fn main() -> Result<(), codecatalyst::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_codecatalyst::Client::new(&config);

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
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) - For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

## License

This project is licensed under the Apache-2.0 License.

