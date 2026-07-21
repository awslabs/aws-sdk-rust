# aws-sdk-organizations

Organizations is a web service that enables you to consolidate your multiple Amazon Web Services accounts into an _organization_ and centrally manage your accounts and their resources.

This guide provides descriptions of the Organizations API. For more information about using this service, see the [Organizations User Guide](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_introduction.html).

__API version__

This version of the Organizations API Reference documents the Organizations API version 2016-11-28.

We recommend that you use the Amazon Web Services SDKs to make programmatic API calls to Organizations. However, you also can use the Organizations Query API to make direct calls to the Organizations web service. To learn more about the Organizations Query API, see [Calling the API by making HTTP Query requests](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_query-requests.html) in the _Organizations User Guide_. Organizations supports GET and POST requests for all actions. That is, the API doesn't require you to use GET for some actions and POST for others. However, GET requests are subject to the limitation size of a URL. Therefore, for operations that require larger sizes, use a POST request.

__Signing requests__

When you send HTTP requests to Amazon Web Services, sign the requests so that Amazon Web Services can identify who sent them. You sign requests with your Amazon Web Services access key, which consists of an access key ID and a secret access key. We strongly recommend that you don't create an access key for your root account. Anyone who has the access key for your root account has unrestricted access to all the resources in your account. Instead, create an access key for an IAM user that has administrative permissions. As another option, use Amazon Web Services Security Token Service (Amazon Web Services STS) to generate temporary security credentials, and use those credentials to sign requests.

To sign requests, we recommend that you use [Signature Version 4](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html). If you have an existing application that uses Signature Version 2, you don't have to update it to use Signature Version 4. However, some operations now require Signature Version 4. The documentation for operations that require version 4 indicate this requirement.

When you use the Command Line Interface (CLI) or one of the Amazon Web Services SDKs to make requests to Amazon Web Services, these tools automatically sign the requests for you with the access key that you specify when you configure the tools.

In this release, each organization can have only one root.

__Support and feedback for Organizations__

We welcome your feedback. You can post your feedback and questions in the [Organizations support forum](https://forums.aws.amazon.com/forum.jspa?forumID=219). For more information about the Amazon Web Services Support forums, see [Forums Help](https://forums.aws.amazon.com/help.jspa).

__Endpoint to call When using the CLI or the Amazon Web Services SDK__

For the current release of Organizations, specify the us-east-1 region for all Amazon Web Services API and CLI calls made from the commercial Amazon Web Services Regions outside of China. If calling from one of the Amazon Web Services Regions in China, then specify cn-northwest-1. You can do this in the CLI by using these parameters and commands:
  - Use the following parameter with each command to specify both the endpoint and its region: --endpoint-url https://organizations.us-east-1.amazonaws.com _(from commercial Amazon Web Services Regions outside of China)_ or --endpoint-url https://organizations.cn-northwest-1.amazonaws.com.cn _(from Amazon Web Services Regions in China)_
  - Use the default endpoint, but configure your default region with this command: aws configure set default.region us-east-1 _(from commercial Amazon Web Services Regions outside of China)_ or aws configure set default.region cn-northwest-1 _(from Amazon Web Services Regions in China)_
  - Use the following parameter with each command to specify the endpoint: --region us-east-1 _(from commercial Amazon Web Services Regions outside of China)_ or --region cn-northwest-1 _(from Amazon Web Services Regions in China)_

__How examples are presented__

The JSON returned by the Organizations service as response to your requests arrives as a single long string without line breaks or formatting whitespace. The examples in this guide include both line breaks and whitespace to improve readability. When example input parameters also would result in long strings that would extend beyond the screen, we insert line breaks to enhance readability. Always submit the input as a single JSON text string.

__Recording API Requests__

Organizations supports CloudTrail, a service that records Amazon Web Services API calls for your Amazon Web Services account and delivers log files to an Amazon S3 bucket. By using information collected by CloudTrail, you can determine which requests the Organizations service received, who made the request and when, and so on. For more about Organizations and its support for CloudTrail, see [Logging Organizations API calls with CloudTrail](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_security_incident-response.html#orgs_cloudtrail-integration) in the _Organizations User Guide_. To learn more about CloudTrail, including how to turn it on and find your log files, see the [CloudTrail User Guide](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/what_is_cloud_trail_top_level.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [usage examples](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rustv1).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-organizations` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-organizations = "1.119.1"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_organizations as organizations;

#[::tokio::main]
async fn main() -> Result<(), organizations::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_organizations::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-organizations/latest/aws_sdk_organizations/client/struct.Client.html)
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

