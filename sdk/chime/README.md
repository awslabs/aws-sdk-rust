# aws-sdk-chime

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

The Amazon Chime API (application programming interface) is designed for developers to perform key tasks, such as creating and managing Amazon Chime accounts, users, and Voice Connectors. This guide provides detailed information about the Amazon Chime API, including operations, types, inputs and outputs, and error codes. It also includes API actions for use with the Amazon Chime SDK, which developers use to build their own communication applications. For more information about the Amazon Chime SDK, see [Using the Amazon Chime SDK](https://docs.aws.amazon.com/chime/latest/dg/meetings-sdk.html) in the _Amazon Chime Developer Guide_.

You can use an AWS SDK, the AWS Command Line Interface (AWS CLI), or the REST API to make API calls. We recommend using an AWS SDK or the AWS CLI. Each API operation includes links to information about using it with a language-specific AWS SDK or the AWS CLI.

__Using an AWS SDK__

You don't need to write code to calculate a signature for request authentication. The SDK clients authenticate your requests by using access keys that you provide. For more information about AWS SDKs, see the [AWS Developer Center](http://aws.amazon.com/developer/).

__Using the AWS CLI__

Use your access keys with the AWS CLI to make API calls. For information about setting up the AWS CLI, see [Installing the AWS Command Line Interface](https://docs.aws.amazon.com/cli/latest/userguide/installing.html) in the _AWS Command Line Interface User Guide_. For a list of available Amazon Chime commands, see the [Amazon Chime commands](https://docs.aws.amazon.com/cli/latest/reference/chime/index.html) in the _AWS CLI Command Reference_.

__Using REST APIs__

If you use REST to make API calls, you must authenticate your request by providing a signature. Amazon Chime supports signature version 4. For more information, see [Signature Version 4 Signing Process](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html) in the _Amazon Web Services General Reference_. When making REST API calls, use the service name chime and REST endpoint https://service.chime.aws.amazon.com.


Administrative permissions are controlled using AWS Identity and Access Management (IAM). For more information, see [Identity and Access Management for Amazon Chime](https://docs.aws.amazon.com/chime/latest/ag/security-iam.html) in the _Amazon Chime Administration Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-chime` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-chime = "0.14.0"
tokio = { version = "1", features = ["full"] }
```

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

