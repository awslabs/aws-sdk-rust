# aws-sdk-organizations

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

AWS Organizations is a web service that enables you to consolidate your multiple AWS accounts into an _organization_ and centrally manage your accounts and their resources.

This guide provides descriptions of the Organizations operations. For more information about using this service, see the [AWS Organizations User Guide](http://docs.aws.amazon.com/organizations/latest/userguide/orgs_introduction.html).

__Support and feedback for AWS Organizations__

We welcome your feedback. Send your comments to [feedback-awsorganizations@amazon.com](mailto:feedback-awsorganizations@amazon.com) or post your feedback and questions in the [AWS Organizations support forum](http://forums.aws.amazon.com/forum.jspa?forumID=219). For more information about the AWS support forums, see [Forums Help](http://forums.aws.amazon.com/help.jspa).

__Endpoint to call When using the AWS CLI or the AWS SDK__

For the current release of Organizations, specify the us-east-1 region for all AWS API and AWS CLI calls made from the commercial AWS Regions outside of China. If calling from one of the AWS Regions in China, then specify cn-northwest-1. You can do this in the AWS CLI by using these parameters and commands:
  - Use the following parameter with each command to specify both the endpoint and its region: --endpoint-url https://organizations.us-east-1.amazonaws.com _(from commercial AWS Regions outside of China)_ or --endpoint-url https://organizations.cn-northwest-1.amazonaws.com.cn _(from AWS Regions in China)_
  - Use the default endpoint, but configure your default region with this command: aws configure set default.region us-east-1 _(from commercial AWS Regions outside of China)_ or aws configure set default.region cn-northwest-1 _(from AWS Regions in China)_
  - Use the following parameter with each command to specify the endpoint: --region us-east-1 _(from commercial AWS Regions outside of China)_ or --region cn-northwest-1 _(from AWS Regions in China)_

__Recording API Requests__

AWS Organizations supports AWS CloudTrail, a service that records AWS API calls for your AWS account and delivers log files to an Amazon S3 bucket. By using information collected by AWS CloudTrail, you can determine which requests the Organizations service received, who made the request and when, and so on. For more about AWS Organizations and its support for AWS CloudTrail, see [Logging AWS Organizations Events with AWS CloudTrail](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_incident-response.html#orgs_cloudtrail-integration) in the _AWS Organizations User Guide_. To learn more about AWS CloudTrail, including how to turn it on and find your log files, see the [AWS CloudTrail User Guide](http://docs.aws.amazon.com/awscloudtrail/latest/userguide/what_is_cloud_trail_top_level.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-organizations` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.9.0"
aws-sdk-organizations = "0.9.0"
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

