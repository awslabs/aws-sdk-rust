# aws-sdk-codestarconnections

This Amazon Web Services CodeStar Connections API Reference provides descriptions and usage examples of the operations and data types for the Amazon Web Services CodeStar Connections API. You can use the connections API to work with connections and installations.

_Connections_ are configurations that you use to connect Amazon Web Services resources to external code repositories. Each connection is a resource that can be given to services such as CodePipeline to connect to a third-party repository such as Bitbucket. For example, you can add the connection in CodePipeline so that it triggers your pipeline when a code change is made to your third-party code repository. Each connection is named and associated with a unique ARN that is used to reference the connection.

When you create a connection, the console initiates a third-party connection handshake. _Installations_ are the apps that are used to conduct this handshake. For example, the installation for the Bitbucket provider type is the Bitbucket app. When you create a connection, you can choose an existing installation or create one.

When you want to create a connection to an installed provider type such as GitHub Enterprise Server, you create a _host_ for your connections.

You can work with connections by calling:
  - CreateConnection, which creates a uniquely named connection that can be referenced by services such as CodePipeline.
  - DeleteConnection, which deletes the specified connection.
  - GetConnection, which returns information about the connection, including the connection status.
  - ListConnections, which lists the connections associated with your account.

You can work with hosts by calling:
  - CreateHost, which creates a host that represents the infrastructure where your provider is installed.
  - DeleteHost, which deletes the specified host.
  - GetHost, which returns information about the host, including the setup status.
  - ListHosts, which lists the hosts associated with your account.

You can work with tags in Amazon Web Services CodeStar Connections by calling the following:
  - ListTagsForResource, which gets information about Amazon Web Services tags for a specified Amazon Resource Name (ARN) in Amazon Web Services CodeStar Connections.
  - TagResource, which adds or updates tags for a resource in Amazon Web Services CodeStar Connections.
  - UntagResource, which removes tags for a resource in Amazon Web Services CodeStar Connections.

For information about how to use Amazon Web Services CodeStar Connections, see the [Developer Tools User Guide](https://docs.aws.amazon.com/dtconsole/latest/userguide/welcome-connections.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-codestarconnections` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-codestarconnections = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_codestarconnections as codestarconnections;

#[::tokio::main]
async fn main() -> Result<(), codestarconnections::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_codestarconnections::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-codestarconnections/latest/aws_sdk_codestarconnections/client/struct.Client.html)
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

