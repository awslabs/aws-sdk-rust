# aws-sdk-acmpca

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

This is the _ACM Private CA API Reference_. It provides descriptions, syntax, and usage examples for each of the actions and data types involved in creating and managing private certificate authorities (CA) for your organization.

The documentation for each action shows the Query API request parameters and the XML response. Alternatively, you can use one of the AWS SDKs to access an API that's tailored to the programming language or platform that you're using. For more information, see [AWS SDKs](https://aws.amazon.com/tools/#SDKs).

Each ACM Private CA API operation has a quota that determines the number of times the operation can be called per second. ACM Private CA throttles API requests at different rates depending on the operation. Throttling means that ACM Private CA rejects an otherwise valid request because the request exceeds the operation's quota for the number of requests per second. When a request is throttled, ACM Private CA returns a [ThrottlingException](https://docs.aws.amazon.com/acm-pca/latest/APIReference/CommonErrors.html) error. ACM Private CA does not guarantee a minimum request rate for APIs.

To see an up-to-date list of your ACM Private CA quotas, or to request a quota increase, log into your AWS account and visit the [Service Quotas](https://console.aws.amazon.com/servicequotas/) console.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-acmpca` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.9.0"
aws-sdk-acmpca = "0.9.0"
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

