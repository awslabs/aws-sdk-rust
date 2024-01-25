# aws-sdk-ivschat

__Introduction__

The Amazon IVS Chat control-plane API enables you to create and manage Amazon IVS Chat resources. You also need to integrate with the [Amazon IVS Chat Messaging API](https://docs.aws.amazon.com/ivs/latest/chatmsgapireference/chat-messaging-api.html), to enable users to interact with chat rooms in real time.

The API is an AWS regional service. For a list of supported regions and Amazon IVS Chat HTTPS service endpoints, see the Amazon IVS Chat information on the [Amazon IVS page](https://docs.aws.amazon.com/general/latest/gr/ivs.html) in the _AWS General Reference_.

__Notes on terminology:__
  - You create service applications using the Amazon IVS Chat API. We refer to these as _applications_.
  - You create front-end client applications (browser and Android/iOS apps) using the Amazon IVS Chat Messaging API. We refer to these as _clients_.

__Resources__

The following resources are part of Amazon IVS Chat:
  - __LoggingConfiguration__ — A configuration that allows customers to store and record sent messages in a chat room. See the Logging Configuration endpoints for more information.
  - __Room__ — The central Amazon IVS Chat resource through which clients connect to and exchange chat messages. See the Room endpoints for more information.

__Tagging__

A _tag_ is a metadata label that you assign to an AWS resource. A tag comprises a _key_ and a _value_, both set by you. For example, you might set a tag as topic:nature to label a particular video category. See [Tagging AWS Resources](https://docs.aws.amazon.com/general/latest/gr/aws_tagging.html) for more information, including restrictions that apply to tags and "Tag naming limits and requirements"; Amazon IVS Chat has no service-specific constraints beyond what is documented there.

Tags can help you identify and organize your AWS resources. For example, you can use the same tag for different resources to indicate that they are related. You can also use tags to manage access (see [Access Tags](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_tags.html)).

The Amazon IVS Chat API has these tag-related endpoints: TagResource, UntagResource, and ListTagsForResource. The following resource supports tagging: Room.

At most 50 tags can be applied to a resource.

__API Access Security__

Your Amazon IVS Chat applications (service applications and clients) must be authenticated and authorized to access Amazon IVS Chat resources. Note the differences between these concepts:
  - _Authentication_ is about verifying identity. Requests to the Amazon IVS Chat API must be signed to verify your identity.
  - _Authorization_ is about granting permissions. Your IAM roles need to have permissions for Amazon IVS Chat API requests.

Users (viewers) connect to a room using secure access tokens that you create using the CreateChatToken endpoint through the AWS SDK. You call CreateChatToken for every user’s chat session, passing identity and authorization information about the user.

__Signing API Requests__

HTTP API requests must be signed with an AWS SigV4 signature using your AWS security credentials. The AWS Command Line Interface (CLI) and the AWS SDKs take care of signing the underlying API calls for you. However, if your application calls the Amazon IVS Chat HTTP API directly, it’s your responsibility to sign the requests.

You generate a signature using valid AWS credentials for an IAM role that has permission to perform the requested action. For example, DeleteMessage requests must be made using an IAM role that has the ivschat:DeleteMessage permission.

For more information:
  - Authentication and generating signatures — See [Authenticating Requests (Amazon Web Services Signature Version 4)](https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-authenticating-requests.html) in the _Amazon Web Services General Reference_.
  - Managing Amazon IVS permissions — See [Identity and Access Management](https://docs.aws.amazon.com/ivs/latest/userguide/security-iam.html) on the Security page of the _Amazon IVS User Guide_.

__Amazon Resource Names (ARNs)__

ARNs uniquely identify AWS resources. An ARN is required when you need to specify a resource unambiguously across all of AWS, such as in IAM policies and API calls. For more information, see [Amazon Resource Names](https://docs.aws.amazon.com/general/latest/gr/aws-arns-and-namespaces.html) in the _AWS General Reference_.

__Messaging Endpoints__
  - DeleteMessage — Sends an event to a specific room which directs clients to delete a specific message; that is, unrender it from view and delete it from the client’s chat history. This event’s EventName is aws:DELETE_MESSAGE. This replicates the [DeleteMessage](https://docs.aws.amazon.com/ivs/latest/chatmsgapireference/actions-deletemessage-publish.html) WebSocket operation in the Amazon IVS Chat Messaging API.
  - DisconnectUser — Disconnects all connections using a specified user ID from a room. This replicates the [DisconnectUser](https://docs.aws.amazon.com/ivs/latest/chatmsgapireference/actions-disconnectuser-publish.html) WebSocket operation in the Amazon IVS Chat Messaging API.
  - SendEvent — Sends an event to a room. Use this within your application’s business logic to send events to clients of a room; e.g., to notify clients to change the way the chat UI is rendered.

__Chat Token Endpoint__
  - CreateChatToken — Creates an encrypted token that is used by a chat participant to establish an individual WebSocket chat connection to a room. When the token is used to connect to chat, the connection is valid for the session duration specified in the request. The token becomes invalid at the token-expiration timestamp included in the response.

__Room Endpoints__
  - CreateRoom — Creates a room that allows clients to connect and pass messages.
  - DeleteRoom — Deletes the specified room.
  - GetRoom — Gets the specified room.
  - ListRooms — Gets summary information about all your rooms in the AWS region where the API request is processed.
  - UpdateRoom — Updates a room’s configuration.

__Logging Configuration Endpoints__
  - CreateLoggingConfiguration — Creates a logging configuration that allows clients to store and record sent messages.
  - DeleteLoggingConfiguration — Deletes the specified logging configuration.
  - GetLoggingConfiguration — Gets the specified logging configuration.
  - ListLoggingConfigurations — Gets summary information about all your logging configurations in the AWS region where the API request is processed.
  - UpdateLoggingConfiguration — Updates a specified logging configuration.

__Tags Endpoints__
  - ListTagsForResource — Gets information about AWS tags for the specified ARN.
  - TagResource — Adds or updates tags for the AWS resource with the specified ARN.
  - UntagResource — Removes tags from the resource with the specified ARN.

All the above are HTTP operations. There is a separate _messaging_ API for managing Chat resources; see the [Amazon IVS Chat Messaging API Reference](https://docs.aws.amazon.com/ivs/latest/chatmsgapireference/chat-messaging-api.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ivschat` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-ivschat = "1.12.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_ivschat as ivschat;

#[::tokio::main]
async fn main() -> Result<(), ivschat::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_ivschat::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-ivschat/latest/aws_sdk_ivschat/client/struct.Client.html)
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

