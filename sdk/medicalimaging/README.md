# aws-sdk-medicalimaging

This is the _AWS HealthImaging API Reference_. AWS HealthImaging is a HIPAA-eligible service that helps health care providers and their medical imaging ISV partners store, transform, and apply machine learning to medical images. For an introduction to the service, see the [_AWS HealthImaging Developer Guide_](https://docs.aws.amazon.com/healthimaging/latest/devguide/what-is.html).

The following sections list AWS HealthImaging API actions categorized according to functionality. Links are provided to actions within this Reference, along with links back to corresponding sections in the _AWS HealthImaging Developer Guide_ where you can view console procedures and CLI/SDK code examples.

__Data store actions__
  - [CreateDatastore](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_CreateDatastore.html) – See [Creating a data store](https://docs.aws.amazon.com/healthimaging/latest/devguide/create-data-store.html).
  - [GetDatastore](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_GetDatastore.html) – See [Getting data store properties](https://docs.aws.amazon.com/healthimaging/latest/devguide/get-data-store.html).
  - [ListDatastores](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_ListDatastores.html) – See [Listing data stores](https://docs.aws.amazon.com/healthimaging/latest/devguide/list-data-stores.html).
  - [DeleteDatastore](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_DeleteDatastore.html) – See [Deleting a data store](https://docs.aws.amazon.com/healthimaging/latest/devguide/delete-data-store.html).

__Import job actions__
  - [StartDICOMImportJob](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_StartDICOMImportJob.html) – See [Starting an import job](https://docs.aws.amazon.com/healthimaging/latest/devguide/start-dicom-import-job.html).
  - [GetDICOMImportJob](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_GetDICOMImportJob.html) – See [Getting import job properties](https://docs.aws.amazon.com/healthimaging/latest/devguide/get-dicom-import-job.html).
  - [ListDICOMImportJobs](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_ListDICOMImportJobs.html) – See [Listing import jobs](https://docs.aws.amazon.com/healthimaging/latest/devguide/list-dicom-import-jobs.html).

__Image set access actions__
  - [SearchImageSets](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_SearchImageSets.html) – See [Searching image sets](https://docs.aws.amazon.com/healthimaging/latest/devguide/search-image-sets.html).
  - [GetImageSet](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_GetImageSet.html) – See [Getting image set properties](https://docs.aws.amazon.com/healthimaging/latest/devguide/get-image-set-properties.html).
  - [GetImageSetMetadata](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_GetImageSetMetadata.html) – See [Getting image set metadata](https://docs.aws.amazon.com/healthimaging/latest/devguide/get-image-set-metadata.html).
  - [GetImageFrame](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_GetImageFrame.html) – See [Getting image set pixel data](https://docs.aws.amazon.com/healthimaging/latest/devguide/get-image-frame.html).

__Image set modification actions__
  - [ListImageSetVersions](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_ListImageSetVersions.html) – See [Listing image set versions](https://docs.aws.amazon.com/healthimaging/latest/devguide/list-image-set-versions.html).
  - [UpdateImageSetMetadata](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_UpdateImageSetMetadata.html) – See [Updating image set metadata](https://docs.aws.amazon.com/healthimaging/latest/devguide/update-image-set-metadata.html).
  - [CopyImageSet](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_CopyImageSet.html) – See [Copying an image set](https://docs.aws.amazon.com/healthimaging/latest/devguide/copy-image-set.html).
  - [DeleteImageSet](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_DeleteImageSet.html) – See [Deleting an image set](https://docs.aws.amazon.com/healthimaging/latest/devguide/delete-image-set.html).

__Tagging actions__
  - [TagResource](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_TagResource.html) – See [Tagging a data store](https://docs.aws.amazon.com/healthimaging/latest/devguide/tag-list-untag-data-store.html) and [Tagging an image set](https://docs.aws.amazon.com/healthimaging/latest/devguide/tag-list-untag-image-set.html).
  - [ListTagsForResource](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_ListTagsForResource.html) – See [Tagging a data store](https://docs.aws.amazon.com/healthimaging/latest/devguide/tag-list-untag-data-store.html) and [Tagging an image set](https://docs.aws.amazon.com/healthimaging/latest/devguide/tag-list-untag-image-set.html).
  - [UntagResource](https://docs.aws.amazon.com/healthimaging/latest/APIReference/API_UntagResource.html) – See [Tagging a data store](https://docs.aws.amazon.com/healthimaging/latest/devguide/tag-list-untag-data-store.html) and [Tagging an image set](https://docs.aws.amazon.com/healthimaging/latest/devguide/tag-list-untag-image-set.html).

## Getting Started

> Examples are available for many services and operations, check out the
> [examples folder in GitHub](https://github.com/awslabs/aws-sdk-rust/tree/main/examples).

The SDK provides one crate per AWS service. You must add [Tokio](https://crates.io/crates/tokio)
as a dependency within your Rust project to execute asynchronous code. To add `aws-sdk-medicalimaging` to
your project, add the following to your **Cargo.toml** file:

```toml
[dependencies]
aws-config = { version = "1.1.4", features = ["behavior-version-latest"] }
aws-sdk-medicalimaging = "1.12.0"
tokio = { version = "1", features = ["full"] }
```

Then in code, a client can be created with the following:

```rust,no_run
use aws_sdk_medicalimaging as medicalimaging;

#[::tokio::main]
async fn main() -> Result<(), medicalimaging::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_medicalimaging::Client::new(&config);

    // ... make some calls with the client

    Ok(())
}
```

See the [client documentation](https://docs.rs/aws-sdk-medicalimaging/latest/aws_sdk_medicalimaging/client/struct.Client.html)
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

