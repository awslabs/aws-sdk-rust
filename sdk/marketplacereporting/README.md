# aws-sdk-marketplacereporting

The Amazon Web Services Marketplace GetBuyerDashboard API enables you to get a procurement insights dashboard programmatically. The API gets the agreement and cost analysis dashboards with data for all of the Amazon Web Services accounts in your Amazon Web Services Organization.

To use the Amazon Web Services Marketplace Reporting API, you must complete the following prerequisites:
  - Enable all features for your organization. For more information, see [Enabling all features for an organization with Organizations](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_manage_org_support-all-features.html), in the _Organizations User Guide_.
  - Call the service as the Organizations management account or an account registered as a delegated administrator for the procurement insights service. For more information about management accounts, see [Tutorial: Creating and configuring an organization](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_tutorials_basic.html) and [Managing the management account with Organizations](https://docs.aws.amazon.com/organizations/latest/userguide/orgs-manage_accounts_management.html), both in the _Organizations User Guide_. For more information about delegated administrators, see [Using delegated administrators](https://docs.aws.amazon.com/marketplace/latest/buyerguide/management-delegates.html), in the _Amazon Web Services Marketplace Buyer Guide_.
  - Create an IAM policy that enables the aws-marketplace:GetBuyerDashboard and organizations:DescribeOrganization permissions. In addition, the management account requires the organizations:EnableAWSServiceAccess and iam:CreateServiceLinkedRole permissions to create. For more information about creating the policy, see [Policies and permissions in Identity and Access Management](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies.html), in the _IAM User Guide_.
  - Use the Amazon Web Services Marketplace console to create the AWSServiceRoleForProcurementInsightsPolicy service-linked role. The role enables Amazon Web Services Marketplace procurement visibility integration. The management account requires an IAM policy with the organizations:EnableAWSServiceAccess and iam:CreateServiceLinkedRole permissions to create the service-linked role and enable the service access. For more information, see [Granting access to Organizations](https://docs.aws.amazon.com/marketplace/latest/buyerguide/orgs-access-slr.html) and [Service-linked role to share procurement data](https://docs.aws.amazon.com/marketplace/latest/buyerguide/buyer-service-linked-role-procurement.html) in the _Amazon Web Services Marketplace Buyer Guide_.
  - After creating the service-linked role, you must enable trusted access that grants Amazon Web Services Marketplace permission to access data from your Organizations. For more information, see [Granting access to Organizations](https://docs.aws.amazon.com/marketplace/latest/buyerguide/orgs-access-slr.html) in the _Amazon Web Services Marketplace Buyer Guide_.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-marketplacereporting` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-marketplacereporting = "1.3.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_marketplacereporting as marketplacereporting;

#[::tokio::main]
async fn main() -> Result<(), marketplacereporting::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_marketplacereporting::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-marketplacereporting/latest/aws_sdk_marketplacereporting/client/struct.Client.html)
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

