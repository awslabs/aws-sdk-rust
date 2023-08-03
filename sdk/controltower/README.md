# aws-sdk-controltower

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

These interfaces allow you to apply the AWS library of pre-defined _controls_ to your organizational units, programmatically. In this context, controls are the same as AWS Control Tower guardrails.

To call these APIs, you'll need to know:
  - the ControlARN for the control--that is, the guardrail--you are targeting,
  - and the ARN associated with the target organizational unit (OU).

__To get the ControlARN for your AWS Control Tower guardrail:__

The ControlARN contains the control name which is specified in each guardrail. For a list of control names for _Strongly recommended_ and _Elective_ guardrails, see [Resource identifiers for APIs and guardrails](https://docs.aws.amazon.com/controltower/latest/userguide/control-identifiers.html.html) in the [Automating tasks section](https://docs.aws.amazon.com/controltower/latest/userguide/automating-tasks.html) of the AWS Control Tower User Guide. Remember that _Mandatory_ guardrails cannot be added or removed.

__To get the ARN for an OU:__

In the AWS Organizations console, you can find the ARN for the OU on the __Organizational unit details__ page associated with that OU.

__Details and examples__
  - [List of resource identifiers for APIs and guardrails](https://docs.aws.amazon.com/controltower/latest/userguide/control-identifiers.html)
  - [Guardrail API examples (CLI)](https://docs.aws.amazon.com/controltower/latest/userguide/guardrail-api-examples-short.html)
  - [Enable controls with AWS CloudFormation](https://docs.aws.amazon.com/controltower/latest/userguide/enable-controls.html)
  - [Creating AWS Control Tower resources with AWS CloudFormation](https://docs.aws.amazon.com/controltower/latest/userguide/creating-resources-with-cloudformation.html)

To view the open source resource repository on GitHub, see [aws-cloudformation/aws-cloudformation-resource-providers-controltower](https://github.com/aws-cloudformation/aws-cloudformation-resource-providers-controltower)

__Recording API Requests__

AWS Control Tower supports AWS CloudTrail, a service that records AWS API calls for your AWS account and delivers log files to an Amazon S3 bucket. By using information collected by CloudTrail, you can determine which requests the AWS Control Tower service received, who made the request and when, and so on. For more about AWS Control Tower and its support for CloudTrail, see [Logging AWS Control Tower Actions with AWS CloudTrail](https://docs.aws.amazon.com/controltower/latest/userguide/logging-using-cloudtrail.html) in the AWS Control Tower User Guide. To learn more about CloudTrail, including how to turn it on and find your log files, see the AWS CloudTrail User Guide.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-controltower` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.56.0"
aws-sdk-controltower = "0.10.0"
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

