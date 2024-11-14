# aws-sdk-partnercentralselling

__AWS Partner Central API for Selling Reference Guide__

Amazon Web Services (AWS) Partner Central API reference guide is designed to help [AWS Partners](https://docs.aws.amazon.com/partners/programs/) programmatically integrate their Customer Relationship Management (CRM) systems with AWS Partner Central. Through the Partner Central APIs, partners can automate and streamline their interactions with AWS Partner Central, ensuring a more efficient and effective engagement in joint business activities.

The AWS Partner Central API service provides standard AWS API functionality. You can directly use the API [Actions](https://docs.aws.amazon.com/partner-central/latest/selling-api/API_Operations.html), or you can use an AWS SDK to access an API that's tailored to the programming language or platform that you're using. For more information about AWS application development, see [Getting Started with AWS](https://docs.aws.amazon.com/getting-started). For more information about using AWS SDKs, see [AWS SDKs](https://docs.aws.amazon.com/aws-sdk).

__Features offered by AWS Partner Central API__
  1. __Opportunity management:__ Facilitates the management of co-selling opportunities with AWS using API actions such as CreateOpportunity, UpdateOpportunity, ListOpportunities, GetOpportunity, and AssignOpportunity.
  1. __AWS referral management:__ Facilitates receiving referrals shared by AWS using actions like ListEngagementInvitations, GetEngagementInvitation, StartEngagementByAcceptingInvitation, and RejectEngagementInvitation.
  1. __Entity association:__ Associate related entities such as _AWS Products_, _Partner Solutions_, and _AWS Marketplace Private Offers_ with opportunities using the actions AssociateOpportunity and DisassociateOpportunity.
  1. __View AWS opportunity details:__ Use the GetAWSOpportunitySummary action to retrieve real-time summaries of AWS opportunities that are linked to your opportunities.
  1. __List solutions:__ Provides list APIs for listing solutions partners offer using ListSolutions.
  1. __Event subscription:__ Partners can subscribe to real-time updates on opportunities by listening to events such as _Opportunity Created_, _Opportunity Updated_, _Engagement Invitation Accepted_, _Engagement Invitation Rejected_ and _Engagement Invitation Created_ using AWS EventBridge.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-partnercentralselling` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-partnercentralselling = "1.0.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_partnercentralselling as partnercentralselling;

#[::tokio::main]
async fn main() -> Result<(), partnercentralselling::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_partnercentralselling::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-partnercentralselling/latest/aws_sdk_partnercentralselling/client/struct.Client.html)
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

