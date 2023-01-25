$version: "1.0"

namespace com.amazonaws.s3

use smithy.test#httpResponseTests
use smithy.test#httpRequestTests

apply NotFound @httpResponseTests([
    {
        id: "HeadObjectEmptyBody",
        documentation: "This test case validates https://github.com/awslabs/smithy-rs/issues/456",
        params: {
        },
        bodyMediaType: "application/xml",
        body: "",
        protocol: "aws.protocols#restXml",
        code: 404,
        headers: {
            "x-amz-request-id": "GRZ6BZ468DF52F2E",
            "x-amz-id-2": "UTniwu6QmCIjVeuK2ZfeWBOnu7SqMQOS3Vac6B/K4H2ZCawYUl+nDbhGTImuyhZ5DFiojR3Kcz4=",
            "content-type": "application/xml",
            "date": "Thu, 03 Jun 2021 04:05:52 GMT",
            "server": "AmazonS3"
        }
    }
])


apply GetBucketLocation @httpResponseTests([
    {
        id: "GetBucketLocation",
        documentation: "This test case validates https://github.com/awslabs/aws-sdk-rust/issues/116",
        code: 200,
        body: "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<LocationConstraint xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">us-west-2</LocationConstraint>",
        params: {
            "LocationConstraint": "us-west-2"
        },
        bodyMediaType: "application/xml",
        protocol: "aws.protocols#restXml"
    }
])

apply ListObjects @httpResponseTests([
    {
        id: "KeysWithWhitespace",
        documentation: "This test validates that parsing respects whitespace",
        code: 200,
        bodyMediaType: "application/xml",
        protocol: "aws.protocols#restXml",
        body: """
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>\n
        <ListBucketResult
        	xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">
        	<Name>bucketname</Name>
        	<Prefix></Prefix>
        	<Marker></Marker>
        	<MaxKeys>1000</MaxKeys>
        	<IsTruncated>false</IsTruncated>
        	<Contents>
        		<Key>    </Key>
        		<LastModified>2021-07-16T16:20:53.000Z</LastModified>
        		<ETag>&quot;etag123&quot;</ETag>
        		<Size>0</Size>
        		<Owner>
        			<ID>owner</ID>
        		</Owner>
        		<StorageClass>STANDARD</StorageClass>
        	</Contents>
        	<Contents>
        		<Key> a </Key>
        		<LastModified>2021-07-16T16:02:10.000Z</LastModified>
        		<ETag>&quot;etag123&quot;</ETag>
        		<Size>0</Size>
        		<Owner>
        			<ID>owner</ID>
        		</Owner>
        		<StorageClass>STANDARD</StorageClass>
        	</Contents>
        </ListBucketResult>
        """,
        params: {
            MaxKeys: 1000,
            IsTruncated: false,
            Marker: "",
            Name: "bucketname",
            Prefix: "",
            Contents: [{
                           Key: "    ",
                           LastModified: 1626452453,
                           ETag: "\"etag123\"",
                           Size: 0,
                           Owner: { ID: "owner" },
                           StorageClass: "STANDARD"
                       }, {
                           Key: " a ",
                           LastModified: 1626451330,
                           ETag: "\"etag123\"",
                           Size: 0,
                           Owner: { ID: "owner" },
                           StorageClass: "STANDARD"
                       }]
        }
    }
])

apply PutBucketLifecycleConfiguration @httpRequestTests([
    {
        id: "PutBucketLifecycleConfiguration",
        documentation: "This test validates that the content md5 header is set correctly",
        method: "PUT",
        protocol: "aws.protocols#restXml",
        uri: "/",
        headers: {
            // we can assert this, but when this test is promoted, it can't assert
            // on the exact contents
            "content-md5": "JP8DTuCSH6yDC8wNGg4+mA==",
        },
        bodyMediaType: "application/xml",
        body: """
        <LifecycleConfiguration xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">
            <Rule>
                <Expiration>
                    <Days>1</Days>
                </Expiration>
                <ID>Expire</ID>
                <Status>Enabled</Status>
            </Rule>
        </LifecycleConfiguration>
        """,
        params: {
            "Bucket": "test-bucket",
            "LifecycleConfiguration": {
                "Rules": [
                    {"Expiration": { "Days": 1 }, "Status": "Enabled", "ID": "Expire" },
                ]
            }
        },
        vendorParams: {
            "endpointParams": {
                "builtInParams": {
                    "AWS::Region": "us-east-1"
                }
            }
        }
    }
])

apply CreateMultipartUpload @httpRequestTests([
    {
        id: "CreateMultipartUploadUriConstruction",
        documentation: "This test validates that the URI for CreateMultipartUpload is created correctly",
        method: "POST",
        protocol: "aws.protocols#restXml",
        uri: "/object.txt",
        queryParams: [
            "uploads",
            "x-id=CreateMultipartUpload"
        ],
        params: {
            "Bucket": "test-bucket",
            "Key": "object.txt"
        },
        vendorParams: {
            "endpointParams": {
                "builtInParams": {
                    "AWS::Region": "us-east-1"
                }
            }
        }
    }
])

apply PutObject @httpRequestTests([
    {
        id: "DontSendDuplicateContentType",
        documentation: "This test validates that if a content-type is specified, that only one content-type header is sent",
        method: "PUT",
        protocol: "aws.protocols#restXml",
        uri: "/test-key",
        headers: { "content-type": "text/html" },
        params: {
            Bucket: "test-bucket",
            Key: "test-key",
            ContentType: "text/html"
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
        id: "DontSendDuplicateContentLength",
        documentation: "This test validates that if a content-length is specified, that only one content-length header is sent",
        method: "PUT",
        protocol: "aws.protocols#restXml",
        uri: "/test-key",
        headers: { "content-length": "2" },
        params: {
            Bucket: "test-bucket",
            Key: "test-key",
            ContentLength: 2,
            Body: "ab"
        },
        vendorParams: {
            "endpointParams": {
                "builtInParams": {
                    "AWS::Region": "us-east-1"
                }
            }
        }
    }
])

apply HeadObject @httpRequestTests([
    {
        id: "HeadObjectUriEncoding",
        documentation: "https://github.com/awslabs/aws-sdk-rust/issues/331",

        method: "HEAD",
        protocol: "aws.protocols#restXml",
        uri: "/%3C%3E%20%60%3F%F0%9F%90%B1",
        params: {
            Bucket: "test-bucket",
            Key: "<> `?🐱",
        },
        vendorParams: {
            "endpointParams": {
                "builtInParams": {
                    "AWS::Region": "us-east-1"
                }
            }
        }
    }
])
