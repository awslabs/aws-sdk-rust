# aws-sdk-chime

__Most of these APIs are no longer supported and will not be updated.__ We recommend using the latest versions in the [Amazon Chime SDK API reference](https://docs.aws.amazon.com/chime-sdk/latest/APIReference/welcome.html), in the Amazon Chime SDK.

Using the latest versions requires migrating to dedicated namespaces. For more information, refer to [Migrating from the Amazon Chime namespace](https://docs.aws.amazon.com/chime-sdk/latest/dg/migrate-from-chm-namespace.html) in the _Amazon Chime SDK Developer Guide_.

The Amazon Chime application programming interface (API) is designed so administrators can perform key tasks, such as creating and managing Amazon Chime accounts, users, and Voice Connectors. This guide provides detailed information about the Amazon Chime API, including operations, types, inputs and outputs, and error codes.

You can use an AWS SDK, the AWS Command Line Interface (AWS CLI), or the REST API to make API calls for Amazon Chime. We recommend using an AWS SDK or the AWS CLI. The page for each API action contains a _See Also_ section that includes links to information about using the action with a language-specific AWS SDK or the AWS CLI.

__Using an AWS SDK__

You don't need to write code to calculate a signature for request authentication. The SDK clients authenticate your requests by using access keys that you provide. For more information about AWS SDKs, see the [AWS Developer Center](http://aws.amazon.com/developer/).

__Using the AWS CLI__

Use your access keys with the AWS CLI to make API calls. For information about setting up the AWS CLI, see [Installing the AWS Command Line Interface](https://docs.aws.amazon.com/cli/latest/userguide/installing.html) in the _AWS Command Line Interface User Guide_. For a list of available Amazon Chime commands, see the [Amazon Chime commands](https://docs.aws.amazon.com/cli/latest/reference/chime/index.html) in the _AWS CLI Command Reference_.

__Using REST APIs__

If you use REST to make API calls, you must authenticate your request by providing a signature. Amazon Chime supports Signature Version 4. For more information, see [Signature Version 4 Signing Process](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html) in the _Amazon Web Services General Reference_. When making REST API calls, use the service name chime and REST endpoint https://service.chime.aws.amazon.com.


Administrative permissions are controlled using AWS Identity and Access Management (IAM). For more information, see [Identity and Access Management for Amazon Chime](https://docs.aws.amazon.com/chime/latest/ag/security-iam.html) in the _Amazon Chime Administration Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-chime` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-chime = "0.0.0-local"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_chime as chime;

#[::tokio::main]
async fn main() -> Result<(), chime::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_chime::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-chime/latest/aws_sdk_chime/client/struct.Client.html)
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

