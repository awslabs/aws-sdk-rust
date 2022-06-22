# aws-sdk-codestar

**Please Note: The SDK is currently in Developer Preview and is intended strictly for
feedback purposes only. Do not use this SDK for production workloads.**

This is the API reference for AWS CodeStar. This reference provides descriptions of the operations and data types for the AWS CodeStar API along with usage examples.

You can use the AWS CodeStar API to work with:

Projects and their resources, by calling the following:
  - DeleteProject, which deletes a project.
  - DescribeProject, which lists the attributes of a project.
  - ListProjects, which lists all projects associated with your AWS account.
  - ListResources, which lists the resources associated with a project.
  - ListTagsForProject, which lists the tags associated with a project.
  - TagProject, which adds tags to a project.
  - UntagProject, which removes tags from a project.
  - UpdateProject, which updates the attributes of a project.

Teams and team members, by calling the following:
  - AssociateTeamMember, which adds an IAM user to the team for a project.
  - DisassociateTeamMember, which removes an IAM user from the team for a project.
  - ListTeamMembers, which lists all the IAM users in the team for a project, including their roles and attributes.
  - UpdateTeamMember, which updates a team member's attributes in a project.

Users, by calling the following:
  - CreateUserProfile, which creates a user profile that contains data associated with the user across all projects.
  - DeleteUserProfile, which deletes all user profile information across all projects.
  - DescribeUserProfile, which describes the profile of a user.
  - ListUserProfiles, which lists all user profiles.
  - UpdateUserProfile, which updates the profile for a user.

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-codestar` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = "0.14.0"
aws-sdk-codestar = "0.14.0"
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

