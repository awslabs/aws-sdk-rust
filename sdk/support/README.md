# aws-sdk-support

**Please Note: The SDK is currently released as an alpha and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

The _AWS Support API Reference_ is intended for programmers who need detailed information about the AWS Support operations and data types. You can use the API to manage your support cases programmatically. The AWS Support API uses HTTP methods that return results in JSON format.

The AWS Support service also exposes a set of [AWS Trusted Advisor](http://aws.amazon.com/premiumsupport/trustedadvisor/) features. You can retrieve a list of checks and their descriptions, get check results, specify checks to refresh, and get the refresh status of checks.

The following list describes the AWS Support case management operations:
  - Service names, issue categories, and available severity levels - The DescribeServices and DescribeSeverityLevels operations return AWS service names, service codes, service categories, and problem severity levels. You use these values when you call the CreateCase operation.
  - Case creation, case details, and case resolution - The CreateCase, DescribeCases, DescribeAttachment, and ResolveCase operations create AWS Support cases, retrieve information about cases, and resolve cases.
  - Case communication - The DescribeCommunications, AddCommunicationToCase, and AddAttachmentsToSet operations retrieve and add communications and attachments to AWS Support cases.

The following list describes the operations available from the AWS Support service for Trusted Advisor:
  - DescribeTrustedAdvisorChecks returns the list of checks that run against your AWS resources.
  - Using the checkId for a specific check returned by DescribeTrustedAdvisorChecks, you can call DescribeTrustedAdvisorCheckResult to obtain the results for the check that you specified.
  - DescribeTrustedAdvisorCheckSummaries returns summarized results for one or more Trusted Advisor checks.
  - RefreshTrustedAdvisorCheck requests that Trusted Advisor rerun a specified check.
  - DescribeTrustedAdvisorCheckRefreshStatuses reports the refresh status of one or more checks.

For authentication of requests, AWS Support uses [Signature Version 4 Signing Process](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html).

See [About the AWS Support API](https://docs.aws.amazon.com/awssupport/latest/user/Welcome.html) in the _AWS Support User Guide_ for information about how to use this service to create and manage your support cases, and how to call Trusted Advisor for results of checks on your resources.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/sdk/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-support` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.0.24-alpha"
aws-sdk-support = "0.0.24-alpha"
tokio = { version = "1", features = ["full"] }
```

## Using the SDK

Until the SDK is released, we will be adding information about using the SDK to the
[Guide](https://github.com/awslabs/aws-sdk-rust/blob/main/Guide.md). Feel free to suggest
additional sections for the guide by opening an issue and describing what you are trying to do.

## Getting Help

* [GitHub discussions](https://github.com/awslabs/aws-sdk-rust/discussions) - For ideas, RFCs & general questions
* [GitHub issues](https://github.com/awslabs/aws-sdk-rust/issues/new/choose) – For bug reports & feature requests
* [Generated Docs (latest version)](https://awslabs.github.io/aws-sdk-rust/)
* [Usage examples](https://github.com/awslabs/aws-sdk-rust/tree/main/sdk/examples)

## License

This project is licensed under the Apache-2.0 License.

