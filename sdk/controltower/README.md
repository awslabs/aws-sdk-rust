# aws-sdk-controltower

These interfaces allow you to apply the Amazon Web Services library of pre-defined _controls_ to your organizational units, programmatically. In Amazon Web Services Control Tower, the terms "control" and "guardrail" are synonyms.

To call these APIs, you'll need to know:
  - the controlIdentifier for the control--or guardrail--you are targeting.
  - the ARN associated with the target organizational unit (OU), which we call the targetIdentifier.
  - the ARN associated with a resource that you wish to tag or untag.

__To get the controlIdentifier for your Amazon Web Services Control Tower control:__

The controlIdentifier is an ARN that is specified for each control. You can view the controlIdentifier in the console on the __Control details__ page, as well as in the documentation.

The controlIdentifier is unique in each Amazon Web Services Region for each control. You can find the controlIdentifier for each Region and control in the [Tables of control metadata](https://docs.aws.amazon.com/controltower/latest/userguide/control-metadata-tables.html) in the _Amazon Web Services Control Tower User Guide._

A quick-reference list of control identifers for the Amazon Web Services Control Tower legacy _Strongly recommended_ and _Elective_ controls is given in [Resource identifiers for APIs and controls](https://docs.aws.amazon.com/controltower/latest/userguide/control-identifiers.html.html) in the [Controls reference guide section](https://docs.aws.amazon.com/controltower/latest/userguide/control-identifiers.html) of the _Amazon Web Services Control Tower User Guide_. Remember that _Mandatory_ controls cannot be added or removed.

__To get the targetIdentifier:__

The targetIdentifier is the ARN for an OU.

In the Amazon Web Services Organizations console, you can find the ARN for the OU on the __Organizational unit details__ page associated with that OU.

__Details and examples__
  - [Control API input and output examples with CLI](https://docs.aws.amazon.com/controltower/latest/userguide/control-api-examples-short.html)
  - [Enable controls with CloudFormation](https://docs.aws.amazon.com/controltower/latest/userguide/enable-controls.html)
  - [Control metadata tables](https://docs.aws.amazon.com/controltower/latest/userguide/control-metadata-tables.html)
  - [List of identifiers for legacy controls](https://docs.aws.amazon.com/controltower/latest/userguide/control-identifiers.html)
  - [Controls reference guide](https://docs.aws.amazon.com/controltower/latest/userguide/controls.html)
  - [Controls library groupings](https://docs.aws.amazon.com/controltower/latest/userguide/controls-reference.html)
  - [Creating Amazon Web Services Control Tower resources with Amazon Web Services CloudFormation](https://docs.aws.amazon.com/controltower/latest/userguide/creating-resources-with-cloudformation.html)

To view the open source resource repository on GitHub, see [aws-cloudformation/aws-cloudformation-resource-providers-controltower](https://github.com/aws-cloudformation/aws-cloudformation-resource-providers-controltower)

__Recording API Requests__

Amazon Web Services Control Tower supports Amazon Web Services CloudTrail, a service that records Amazon Web Services API calls for your Amazon Web Services account and delivers log files to an Amazon S3 bucket. By using information collected by CloudTrail, you can determine which requests the Amazon Web Services Control Tower service received, who made the request and when, and so on. For more about Amazon Web Services Control Tower and its support for CloudTrail, see [Logging Amazon Web Services Control Tower Actions with Amazon Web Services CloudTrail](https://docs.aws.amazon.com/controltower/latest/userguide/logging-using-cloudtrail.html) in the Amazon Web Services Control Tower User Guide. To learn more about CloudTrail, including how to turn it on and find your log files, see the Amazon Web Services CloudTrail User Guide.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-controltower` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-controltower = "1.12.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_controltower as controltower;

#[::tokio::main]
async fn main() -> Result<(), controltower::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_controltower::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-controltower/latest/aws_sdk_controltower/client/struct.Client.html)
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

