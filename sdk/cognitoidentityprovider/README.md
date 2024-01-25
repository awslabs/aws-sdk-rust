# aws-sdk-cognitoidentityprovider

With the Amazon Cognito user pools API, you can configure user pools and authenticate users. To authenticate users from third-party identity providers (IdPs) in this API, you can [link IdP users to native user profiles](https://docs.aws.amazon.com/cognito/latest/developerguide/cognito-user-pools-identity-federation-consolidate-users.html). Learn more about the authentication and authorization of federated users at [Adding user pool sign-in through a third party](https://docs.aws.amazon.com/cognito/latest/developerguide/cognito-user-pools-identity-federation.html) and in the [User pool federation endpoints and hosted UI reference](https://docs.aws.amazon.com/cognito/latest/developerguide/cognito-userpools-server-contract-reference.html).

This API reference provides detailed information about API operations and object types in Amazon Cognito.

Along with resource management operations, the Amazon Cognito user pools API includes classes of operations and authorization models for client-side and server-side authentication of users. You can interact with operations in the Amazon Cognito user pools API as any of the following subjects.
  1. An administrator who wants to configure user pools, app clients, users, groups, or other user pool functions.
  1. A server-side app, like a web application, that wants to use its Amazon Web Services privileges to manage, authenticate, or authorize a user.
  1. A client-side app, like a mobile app, that wants to make unauthenticated requests to manage, authenticate, or authorize a user.

For more information, see [Using the Amazon Cognito user pools API and user pool endpoints](https://docs.aws.amazon.com/cognito/latest/developerguide/user-pools-API-operations.html) in the _Amazon Cognito Developer Guide_.

With your Amazon Web Services SDK, you can build the logic to support operational flows in every use case for this API. You can also make direct REST API requests to [Amazon Cognito user pools service endpoints](https://docs.aws.amazon.com/general/latest/gr/cognito_identity.html#cognito_identity_your_user_pools_region). The following links can get you started with the CognitoIdentityProvider client in other supported Amazon Web Services SDKs.
  - [Amazon Web Services Command Line Interface](https://docs.aws.amazon.com/cli/latest/reference/cognito-idp/index.html#cli-aws-cognito-idp)
  - [Amazon Web Services SDK for .NET](https://docs.aws.amazon.com/sdkfornet/v3/apidocs/items/CognitoIdentityProvider/TCognitoIdentityProviderClient.html)
  - [Amazon Web Services SDK for C++](https://sdk.amazonaws.com/cpp/api/LATEST/aws-cpp-sdk-cognito-idp/html/class_aws_1_1_cognito_identity_provider_1_1_cognito_identity_provider_client.html)
  - [Amazon Web Services SDK for Go](https://docs.aws.amazon.com/sdk-for-go/api/service/cognitoidentityprovider/#CognitoIdentityProvider)
  - [Amazon Web Services SDK for Java V2](https://sdk.amazonaws.com/java/api/latest/software/amazon/awssdk/services/cognitoidentityprovider/CognitoIdentityProviderClient.html)
  - [Amazon Web Services SDK for JavaScript](https://docs.aws.amazon.com/AWSJavaScriptSDK/latest/AWS/CognitoIdentityServiceProvider.html)
  - [Amazon Web Services SDK for PHP V3](https://docs.aws.amazon.com/aws-sdk-php/v3/api/api-cognito-idp-2016-04-18.html)
  - [Amazon Web Services SDK for Python](https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/cognito-idp.html)
  - [Amazon Web Services SDK for Ruby V3](https://docs.aws.amazon.com/sdk-for-ruby/v3/api/Aws/CognitoIdentityProvider/Client.html)

To get started with an Amazon Web Services SDK, see [Tools to Build on Amazon Web Services](http://aws.amazon.com/developer/tools/). For example actions and scenarios, see [Code examples for Amazon Cognito Identity Provider using Amazon Web Services SDKs](https://docs.aws.amazon.com/cognito/latest/developerguide/service_code_examples_cognito-identity-provider.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-cognitoidentityprovider` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-cognitoidentityprovider = "1.13.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_cognitoidentityprovider as cognitoidentityprovider;

#[::tokio::main]
async fn main() -> Result<(), cognitoidentityprovider::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_cognitoidentityprovider::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-cognitoidentityprovider/latest/aws_sdk_cognitoidentityprovider/client/struct.Client.html)
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

