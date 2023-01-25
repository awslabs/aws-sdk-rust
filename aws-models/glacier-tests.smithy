$version: "1.0"

namespace com.amazonaws.glacier

use smithy.test#httpRequestTests
use aws.protocols#restJson1

apply UploadArchive @httpRequestTests([
    {
        id: "GlacierVersionHeader",
        documentation: "Glacier requires that a version header be set on all requests.",
        protocol: restJson1,
        method: "POST",
        uri: "/foo/vaults/bar/archives",
        headers: {
            "X-Amz-Glacier-Version": "2012-06-01",
        },
        body: "",
        params: {
            accountId: "foo",
            vaultName: "bar",
        },
        vendorParams: {
            "endpointParams": {
                "builtInParams": {
                    "AWS::Region": "us-east-1"
                }
            }
        }
    },
    {
        id: "GlacierChecksums",
        documentation: "Glacier requires checksum headers that are cumbersome to provide.",
        protocol: restJson1,
        method: "POST",
        uri: "/foo/vaults/bar/archives",
        headers: {
            "X-Amz-Glacier-Version": "2012-06-01",
            "X-Amz-Content-Sha256": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            "X-Amz-Sha256-Tree-Hash": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        },
        body: "hello world",
        params: {
            accountId: "foo",
            vaultName: "bar",
            body: "hello world"
        },
        vendorParams: {
            "endpointParams": {
                "builtInParams": {
                    "AWS::Region": "us-east-1"
                }
            }
        }
        appliesTo: "client",
    },
    {
        id: "GlacierAccountIdEmpty",
        documentation: """
            Glacier requires that the account id be set, but you can just use a
            hyphen (-) to indicate the current account. This should be default
            behavior if the customer provides a null or empty string.""",
        protocol: restJson1,
        method: "POST",
        uri: "/-/vaults/bar/archives",
        headers: {
            "X-Amz-Glacier-Version": "2012-06-01",
        },
        body: "",
        params: {
            accountId: "",
            vaultName: "bar",
        },
        vendorParams: {
            "endpointParams": {
                "builtInParams": {
                    "AWS::Region": "us-east-1"
                }
            }
        }
        appliesTo: "client",
    },
    {
        id: "GlacierAccountIdUnset",
        documentation: """
            Glacier requires that the account id be set, but you can just use a
            hyphen (-) to indicate the current account. This should be default
            behavior if the customer provides a null or empty string.""",
        protocol: restJson1,
        method: "POST",
        uri: "/-/vaults/bar/archives",
        headers: {
            "X-Amz-Glacier-Version": "2012-06-01",
        },
        body: "",
        params: {
            vaultName: "bar",
            accountId: null
        },
        vendorParams: {
            "endpointParams": {
                "builtInParams": {
                    "AWS::Region": "us-east-1"
                }
            }
        }
        appliesTo: "client",
    }
])

apply UploadMultipartPart @httpRequestTests([
    {
        id: "GlacierMultipartChecksums",
        documentation: "Glacier requires checksum headers that are cumbersome to provide.",
        protocol: restJson1,
        method: "PUT",
        uri: "/foo/vaults/bar/multipart-uploads/baz",
        headers: {
            "X-Amz-Glacier-Version": "2012-06-01",
            "X-Amz-Content-Sha256": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            "X-Amz-Sha256-Tree-Hash": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        },
        body: "hello world",
        params: {
            accountId: "foo",
            vaultName: "bar",
            uploadId: "baz",
            body: "hello world"
        },
        vendorParams: {
            "endpointParams": {
                "builtInParams": {
                    "AWS::Region": "us-east-1"
                }
            }
        }
        appliesTo: "client",
    }
])
