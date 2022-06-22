# aws-sdk-wafv2

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

WAF is a web application firewall that lets you monitor the HTTP and HTTPS requests that are forwarded to Amazon CloudFront, an Amazon API Gateway REST API, an Application Load Balancer, or an AppSync GraphQL API. WAF also lets you control access to your content. Based on conditions that you specify, such as the IP addresses that requests originate from or the values of query strings, the Amazon API Gateway REST API, CloudFront distribution, the Application Load Balancer, or the AppSync GraphQL API responds to requests either with the requested content or with an HTTP 403 status code (Forbidden). You also can configure CloudFront to return a custom error page when a request is blocked.

This API guide is for developers who need detailed information about WAF API actions, data types, and errors. For detailed information about WAF features and an overview of how to use WAF, see the [WAF Developer Guide](https://docs.aws.amazon.com/waf/latest/developerguide/what-is-aws-waf.html).

You can make calls using the endpoints listed in [WAF endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/waf.html).
  - For regional applications, you can use any of the endpoints in the list. A regional application can be an Application Load Balancer (ALB), an Amazon API Gateway REST API, or an AppSync GraphQL API.
  - For Amazon CloudFront applications, you must use the API endpoint listed for US East (N. Virginia): us-east-1.

Alternatively, you can use one of the Amazon Web Services SDKs to access an API that's tailored to the programming language or platform that you're using. For more information, see [Amazon Web Services SDKs](http://aws.amazon.com/tools/#SDKs).

We currently provide two versions of the WAF API: this API and the prior versions, the classic WAF APIs. This new API provides the same functionality as the older versions, with the following major improvements:
  - You use one API for both global and regional applications. Where you need to distinguish the scope, you specify a Scope parameter and set it to CLOUDFRONT or REGIONAL.
  - You can define a web ACL or rule group with a single call, and update it with a single call. You define all rule specifications in JSON format, and pass them to your rule group or web ACL calls.
  - The limits WAF places on the use of rules more closely reflects the cost of running each type of rule. Rule groups include capacity settings, so you know the maximum cost of a rule group when you use it.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-wafv2` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-wafv2 = "0.14.0"
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

