# aws-sdk-transfer

Transfer Family offers fully managed support for the transfer of files over SFTP, AS2, FTPS, FTP, and web browser-based transfers directly into and out of Amazon Web Services storage services.

File transfer protocols are used in data exchange workflows across different industries such as financial services, healthcare, advertising, and retail, among others. Transfer Family simplifies the migration of file transfer workflows to Amazon Web Services.

To use the Transfer Family service, you instantiate a server in the Amazon Web Services Region of your choice. You can create the server, list available servers, and update and delete servers. The server is the entity that requests file operations from Transfer Family. Servers have a number of important properties. The server is a named instance as identified by a system assigned ServerId identifier. You can optionally assign a hostname, or even a custom hostname to a server. The service bills for any instantiated servers (even ones OFFLINE), and for the amount of data transferred.

Users must be known to the server that requests file operations. A user as identified by their username is assigned to a server. Usernames are used to authenticate requests. A server can have only one authentication method: AWS_DIRECTORY_SERVICE, SERVICE_MANAGED, AWS_LAMBDA, or API_GATEWAY.

Transfer Family also supports web applications that provide browser-based file transfer capabilities. Web applications can be configured with VPC endpoints to enable secure, private connectivity within your Virtual Private Cloud (VPC). This allows you to control network access and route traffic through your VPC infrastructure while maintaining the managed benefits of Transfer Family.

This API interface reference for Transfer Family contains documentation for a programming interface that you can use to manage Transfer Family. The reference structure is as follows:
  - For the alphabetical list of API actions, see .
  - For the alphabetical list of data types, see .
  - For a list of common query parameters, see CommonParameters.
  - For descriptions of the error codes, see CommonErrors.

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-transfer` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-transfer = "1.127.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_transfer as transfer;

#[::tokio::main]
async fn main() -> Result<(), transfer::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_transfer::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-transfer/latest/aws_sdk_transfer/client/struct.Client.html)
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

