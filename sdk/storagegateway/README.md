# aws-sdk-storagegateway

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

Storage Gateway is the service that connects an on-premises software appliance with cloud-based storage to provide seamless and secure integration between an organization's on-premises IT environment and the Amazon Web Services storage infrastructure. The service enables you to securely upload data to the Amazon Web Services Cloud for cost effective backup and rapid disaster recovery.

Use the following links to get started using the _Storage Gateway Service API Reference_:
  - [Storage Gateway required request headers](https://docs.aws.amazon.com/storagegateway/latest/userguide/AWSStorageGatewayAPI.html#AWSStorageGatewayHTTPRequestsHeaders): Describes the required headers that you must send with every POST request to Storage Gateway.
  - [Signing requests](https://docs.aws.amazon.com/storagegateway/latest/userguide/AWSStorageGatewayAPI.html#AWSStorageGatewaySigningRequests): Storage Gateway requires that you authenticate every request you send; this topic describes how sign such a request.
  - [Error responses](https://docs.aws.amazon.com/storagegateway/latest/userguide/AWSStorageGatewayAPI.html#APIErrorResponses): Provides reference information about Storage Gateway errors.
  - [Operations in Storage Gateway](https://docs.aws.amazon.com/storagegateway/latest/APIReference/API_Operations.html): Contains detailed descriptions of all Storage Gateway operations, their request parameters, response elements, possible errors, and examples of requests and responses.
  - [Storage Gateway endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/sg.html): Provides a list of each Amazon Web Services Region and the endpoints available for use with Storage Gateway.

IDs for Storage Gateway volumes and Amazon EBS snapshots created from gateway volumes are changing to a longer format. Starting in December 2016, all new volumes and snapshots will be created with a 17-character string. Starting in April 2016, you will be able to use these longer IDs so you can test your systems with the new format. For more information, see [Longer EC2 and EBS resource IDs](http://aws.amazon.com/ec2/faqs/#longer-ids).

For example, a volume Amazon Resource Name (ARN) with the longer volume ID format looks like the following:

arn:aws:storagegateway:us-west-2:111122223333:gateway/sgw-12A3456B/volume/vol-1122AABBCCDDEEFFG.

A snapshot ID with the longer ID format looks like the following: snap-78e226633445566ee.

For more information, see [Announcement: Heads-up – Longer Storage Gateway volume and snapshot IDs coming in 2016](http://forums.aws.amazon.com/ann.jspa?annID=3557).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-storagegateway` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-storagegateway = "0.14.0"
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

