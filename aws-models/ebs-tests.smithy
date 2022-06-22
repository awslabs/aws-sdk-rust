$version: "1.0"
namespace com.amazonaws.ebs

use smithy.test#httpResponseTests

apply ValidationException @httpResponseTests([
    {
        id: "LowercaseMessage",
        documentation: "This test case validates case insensitive parsing of `message`",
        params: {
            Message: "1 validation error detected"
        },
        bodyMediaType: "application/json",
        body: """
        {
          "message": "1 validation error detected"
        }
        """,
        protocol: "aws.protocols#restJson1",
        code: 400,
        headers:  {
            "x-amzn-requestid": "2af8f013-250a-4f6e-88ae-6dd7f6e12807",
            "x-amzn-errortype": "ValidationException:http://internal.amazon.com/coral/com.amazon.coral.validate/",
            "content-type": "application/json",
            "content-length": "77",
            "date": "Wed, 30 Jun 2021 23:42:27 GMT"
        },
    },

    {
        id: "UppercaseMessage",
        documentation: "This test case validates case insensitive parsing of `message`",
        params: {
            Message: "Invalid volume size: 99999999999",
            Reason: "INVALID_VOLUME_SIZE"
        },
        bodyMediaType: "application/json",
        body: """
        {"Message":"Invalid volume size: 99999999999","Reason":"INVALID_VOLUME_SIZE"}
        """,
        protocol: "aws.protocols#restJson1",
        code: 400,
        headers:  {
            "x-amzn-requestid": "2af8f013-250a-4f6e-88ae-6dd7f6e12807",
            "x-amzn-errortype": "ValidationException:http://internal.amazon.com/coral/com.amazon.zeppelindataservice/",
            "content-type": "application/json",
            "content-length": "77",
            "date": "Wed, 30 Jun 2021 23:42:27 GMT"
        },
    },
])
