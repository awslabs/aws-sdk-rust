# aws-sdk-ivsrealtime

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

__Introduction__

The Amazon Interactive Video Service (IVS) real-time API is REST compatible, using a standard HTTP API and an AWS EventBridge event stream for responses. JSON is used for both requests and responses, including errors.

Terminology:
  - A _stage_ is a virtual space where participants can exchange video in real time.
  - A _participant token_ is a token that authenticates a participant when they join a stage.
  - A _participant object_ represents participants (people) in the stage and contains information about them. When a token is created, it includes a participant ID; when a participant uses that token to join a stage, the participant is associated with that participant ID There is a 1:1 mapping between participant tokens and participants.

__Resources__

The following resources contain information about your IVS live stream (see [Getting Started with Amazon IVS Real-Time Streaming](https://docs.aws.amazon.com/ivs/latest/RealTimeUserGuide/getting-started.html)):
  - __Stage__ — A stage is a virtual space where participants can exchange video in real time.

__Tagging__

A _tag_ is a metadata label that you assign to an AWS resource. A tag comprises a _key_ and a _value_, both set by you. For example, you might set a tag as topic:nature to label a particular video category. See [Tagging AWS Resources](https://docs.aws.amazon.com/general/latest/gr/aws_tagging.html) for more information, including restrictions that apply to tags and "Tag naming limits and requirements"; Amazon IVS stages has no service-specific constraints beyond what is documented there.

Tags can help you identify and organize your AWS resources. For example, you can use the same tag for different resources to indicate that they are related. You can also use tags to manage access (see [Access Tags](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_tags.html)).

The Amazon IVS real-time API has these tag-related endpoints: TagResource, UntagResource, and ListTagsForResource. The following resource supports tagging: Stage.

At most 50 tags can be applied to a resource.

__Stages Endpoints__
  - CreateParticipantToken — Creates an additional token for a specified stage. This can be done after stage creation or when tokens expire.
  - CreateStage — Creates a new stage (and optionally participant tokens).
  - DeleteStage — Shuts down and deletes the specified stage (disconnecting all participants).
  - DisconnectParticipant — Disconnects a specified participant and revokes the participant permanently from a specified stage.
  - GetParticipant — Gets information about the specified participant token.
  - GetStage — Gets information for the specified stage.
  - GetStageSession — Gets information for the specified stage session.
  - ListParticipantEvents — Lists events for a specified participant that occurred during a specified stage session.
  - ListParticipants — Lists all participants in a specified stage session.
  - ListStages — Gets summary information about all stages in your account, in the AWS region where the API request is processed.
  - ListStageSessions — Gets all sessions for a specified stage.
  - UpdateStage — Updates a stage’s configuration.

__Tags Endpoints__
  - ListTagsForResource — Gets information about AWS tags for the specified ARN.
  - TagResource — Adds or updates tags for the AWS resource with the specified ARN.
  - UntagResource — Removes tags from the resource with the specified ARN.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-ivsrealtime` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.57.1"
aws-sdk-ivsrealtime = "0.11.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_ivsrealtime as ivsrealtime;

#[::tokio::main]
async fn main() -> Result<(), ivsrealtime::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_ivsrealtime::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-ivsrealtime/latest/aws_sdk_ivsrealtime/client/struct.Client.html)
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

