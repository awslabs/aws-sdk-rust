# aws-sdk-auditmanager

Welcome to the Audit Manager API reference. This guide is for developers who need detailed information about the Audit Manager API operations, data types, and errors.

Audit Manager is a service that provides automated evidence collection so that you can continually audit your Amazon Web Services usage. You can use it to assess the effectiveness of your controls, manage risk, and simplify compliance.

Audit Manager provides prebuilt frameworks that structure and automate assessments for a given compliance standard. Frameworks include a prebuilt collection of controls with descriptions and testing procedures. These controls are grouped according to the requirements of the specified compliance standard or regulation. You can also customize frameworks and controls to support internal audits with specific requirements.

Use the following links to get started with the Audit Manager API:
  - [Actions](https://docs.aws.amazon.com/audit-manager/latest/APIReference/API_Operations.html): An alphabetical list of all Audit Manager API operations.
  - [Data types](https://docs.aws.amazon.com/audit-manager/latest/APIReference/API_Types.html): An alphabetical list of all Audit Manager data types.
  - [Common parameters](https://docs.aws.amazon.com/audit-manager/latest/APIReference/CommonParameters.html): Parameters that all operations can use.
  - [Common errors](https://docs.aws.amazon.com/audit-manager/latest/APIReference/CommonErrors.html): Client and server errors that all operations can return.

If you're new to Audit Manager, we recommend that you review the [Audit Manager User Guide](https://docs.aws.amazon.com/audit-manager/latest/userguide/what-is.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-auditmanager` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-auditmanager = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_auditmanager as auditmanager;

#[::tokio::main]
async fn main() -> Result<(), auditmanager::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_auditmanager::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-auditmanager/latest/aws_sdk_auditmanager/client/struct.Client.html)
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

