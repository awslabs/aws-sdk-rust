# aws-sdk-directoryservicedata

Amazon Web Services Directory Service Data is an extension of Directory Service. This API reference provides detailed information about Directory Service Data operations and object types.

With Directory Service Data, you can create, read, update, and delete users, groups, and memberships from your Managed Microsoft AD without additional costs and without deploying dedicated management instances. You can also perform built-in object management tasks across directories without direct network connectivity, which simplifies provisioning and access management to achieve fully automated deployments. Directory Service Data supports user and group write operations, such as CreateUser and CreateGroup, within the organizational unit (OU) of your Managed Microsoft AD. Directory Service Data supports read operations, such as ListUsers and ListGroups, on all users, groups, and group memberships within your Managed Microsoft AD and across trusted realms. Directory Service Data supports adding and removing group members in your OU and the Amazon Web Services Delegated Groups OU, so you can grant and deny access to specific roles and permissions. For more information, see [Manage users and groups](https://docs.aws.amazon.com/directoryservice/latest/admin-guide/ms_ad_manage_users_groups.html) in the _Directory Service Administration Guide_.

Directory Service Data connects to your Managed Microsoft AD domain controllers and performs operations on underlying directory objects. When you create your Managed Microsoft AD, you choose subnets for domain controllers that Directory Service creates on your behalf. If a domain controller is unavailable, Directory Service Data uses an available domain controller. As a result, you might notice eventual consistency while objects replicate from one domain controller to another domain controller. For more information, see [What gets created](https://docs.aws.amazon.com/directoryservice/latest/admin-guide/ms_ad_getting_started_what_gets_created.html) in the _Directory Service Administration Guide_. Directory limits vary by Managed Microsoft AD edition:
  - __Standard edition__ – Supports 8 transactions per second (TPS) for read operations and 4 TPS for write operations per directory. There's a concurrency limit of 10 concurrent requests.
  - __Enterprise edition__ – Supports 16 transactions per second (TPS) for read operations and 8 TPS for write operations per directory. There's a concurrency limit of 10 concurrent requests.
  - __Amazon Web Services Account__ - Supports a total of 100 TPS for Directory Service Data operations across all directories.

Directory Service Data only supports the Managed Microsoft AD directory type and is only available in the primary Amazon Web Services Region. For more information, see [Managed Microsoft AD](https://docs.aws.amazon.com/directoryservice/latest/admin-guide/directory_microsoft_ad.html) and [Primary vs additional Regions](https://docs.aws.amazon.com/directoryservice/latest/admin-guide/multi-region-global-primary-additional.html) in the _Directory Service Administration Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-directoryservicedata` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-directoryservicedata = "1.36.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_directoryservicedata as directoryservicedata;

#[::tokio::main]
async fn main() -> Result<(), directoryservicedata::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_directoryservicedata::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-directoryservicedata/latest/aws_sdk_directoryservicedata/client/struct.Client.html)
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

