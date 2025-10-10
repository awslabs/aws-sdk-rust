# aws-sdk-odb

Oracle Database@Amazon Web Services is an offering that enables you to access Oracle Exadata infrastructure managed by Oracle Cloud Infrastructure (OCI) inside Amazon Web Services data centers. You can migrate your Oracle Exadata workloads, establish low-latency connectivity with applications running on Amazon Web Services, and integrate with Amazon Web Services services. For example, you can run application servers in a Virtual Private Cloud (VPC) and access an Oracle Exadata system running in Oracle Database@Amazon Web Services. You can get started with Oracle Database@Amazon Web Services by using the familiar Amazon Web Services Management Console, APIs, or CLI.

This interface reference for Oracle Database@Amazon Web Services contains documentation for a programming or command line interface that you can use to manage Oracle Database@Amazon Web Services. Oracle Database@Amazon Web Services is asynchronous, which means that some interfaces might require techniques such as polling or callback functions to determine when a command has been applied. The reference structure is as follows.

__Oracle Database@Amazon Web Services API Reference__
  - For the alphabetical list of API actions, see [API Actions](https://docs.aws.amazon.com/odb/latest/APIReference/API_Operations.html).
  - For the alphabetical list of data types, see [Data Types](https://docs.aws.amazon.com/odb/latest/APIReference/API_Types.html).
  - For a list of common query parameters, see [Common Parameters](https://docs.aws.amazon.com/odb/latest/APIReference/CommonParameters.html).
  - For descriptions of the error codes, see [Common Errors](https://docs.aws.amazon.com/odb/latest/APIReference/CommonErrors.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-odb` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-odb = "1.13.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_odb as odb;

#[::tokio::main]
async fn main() -> Result<(), odb::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_odb::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-odb/latest/aws_sdk_odb/client/struct.Client.html)
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

