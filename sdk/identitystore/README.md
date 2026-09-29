# aws-sdk-identitystore

The Identity Store service used by IAM Identity Center provides a single place to retrieve all of your identities (users and groups). You can use the identity store API operations in this guide to manage your identity data programmatically. The scope of these APIs allows you to create, read, update, delete, and list users, groups, and memberships.

This guide also describes identity store operations that you can call and includes detailed information about data types and errors.

If you use an external identity provider or Active Directory as your identity source, we recommend that you use the Create, Update, and Delete APIs with caution. Because IAM Identity Center doesn't support outbound synchronization, your identity source won't automatically update with the changes that you make to users or groups using these APIs.

Amazon Web Services provides SDKs that consist of libraries and sample code for various programming languages and platforms (Java, Ruby, .Net, iOS, Android, and more). The SDKs provide a convenient way to programmatically access the identity store and other Amazon Web Services services. For more information about the Amazon Web Services SDKs, including how to download and install them, see [Amazon Web Services Builder Center Toolbox](http://aws.amazon.com/tools/).

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-identitystore` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-identitystore = "1.116.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_identitystore as identitystore;

#[::tokio::main]
async fn main() -> Result<(), identitystore::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_identitystore::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-identitystore/latest/aws_sdk_identitystore/client/struct.Client.html)
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

