# Contributing Guidelines

Thank you for your interest in contributing to the AWS SDK for Rust! Whether it's a bug report, new feature, correction, or additional
documentation, we greatly value feedback and contributions from our community.

Please read through this document before submitting any contributions to ensure your contribution goes to the correct code repository and we have all the necessary
information to effectively respond to your request.

## The AWS SDK for Rust has three GitHub repositories

1) This repository ([awslabs/aws-sdk-rust](https://github.com/awslabs/aws-sdk-rust)) 

    This repository contains code generated from [awslabs/smithy-rs](https://github.com/awslabs/smithy-rs). If you want to contribute to the SDK by submitting feedback to our roadmap or filing a bug report, you can do so using this GitHub repository. However, because this repository is code generated from Smithy models, **please do not submit PRs modifying code or examples to this repository.**

2) Examples repository ([awsdocs/aws-doc-sdk-examples](https://github.com/awsdocs/aws-doc-sdk-examples))

    All the SDK code examples are in the [`rust_dev_preview`](https://github.com/awsdocs/aws-doc-sdk-examples/tree/main/rust_dev_preview) directory of `aws-doc-sdk-examples`. They are automatically copied over to `aws-sdk-rust` when a new release is cut. Please make example contributions there, and be sure to take a look at its [CONTRIBUTING.md](https://github.com/awsdocs/aws-doc-sdk-examples/blob/main/CONTRIBUTING.md).

3) Smithy-based SDK generation repo ([awslabs/smithy-rs](https://github.com/awslabs/smithy-rs))

    If you want to contribute by diving into the codegen machinery and helping develop the SDK, please navigate to the [Smithy code gen machinery repo](https://github.com/awslabs/smithy-rs).  Here you'll be able to see all the ins-and-outs of the code generator. We welcome PRs and other contributions to this repository, but please make sure to review its [CONTRIBUTING.MD](https://github.com/awslabs/smithy-rs/blob/main/CONTRIBUTING.md)

## Reporting Bugs/Feature Requests

We welcome you to use the GitHub issue tracker to report bugs or suggest features to this code repository.

When filing an issue, please check existing open, or recently closed, issues to make sure somebody else hasn't already
reported the issue. Please try to include as much information as you can. Details like these for bug reports are incredibly useful:

* A reproducible test case or series of steps
* The version of our code being used
* Any modifications you've made relevant to the bug
* Anything unusual about your environment or deployment


## Contributing via Pull Requests
Contributions via pull requests are much appreciated, however, because all of the code on this repository has been code generated from the [smithy-rs](https://github.com/awslabs/smithy-rs), **please do not submit PRs modifying the `sdk` folder to this repo**. The below instructions are for PR that do not update code, such as documentation changes.

Before sending us a pull request, please ensure that:

1. You are working against the latest source on the *main* branch.
2. You check existing open, and recently merged, pull requests to make sure someone else hasn't addressed the problem already.

To send us a pull request, please:

1. Fork the repository.
2. Modify the source; please focus on the specific change you are contributing.
4. Commit to your fork using clear commit messages.
5. Send us a pull request, answering any default questions in the pull request interface.

GitHub provides additional document on [forking a repository](https://help.github.com/articles/fork-a-repo/) and
[creating a pull request](https://help.github.com/articles/creating-a-pull-request/).


## Building High Level Libraries

This is one of the best ways to contribute to the SDK. Looking for an open issue labeled 'help wanted' is a great way to find one that we could use your help with. Please comment on the issue to communicate your interest so we can work with you in its development.


## Code of Conduct
This project has adopted the [Amazon Open Source Code of Conduct](https://aws.github.io/code-of-conduct).
For more information see the [Code of Conduct FAQ](https://aws.github.io/code-of-conduct-faq) or contact
opensource-codeofconduct@amazon.com with any additional questions or comments.


## Security issue notifications
If you discover a potential security issue in this project we ask that you notify AWS/Amazon Security via our [vulnerability reporting page](http://aws.amazon.com/security/vulnerability-reporting/). Please do **not** create a public github issue.


## Licensing

See the [LICENSE](LICENSE) file for our project's licensing. We will ask you to confirm the licensing of your contribution.
