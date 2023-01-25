$version: "1.0"

namespace com.amazonaws.route53

use smithy.test#httpRequestTests


apply ListResourceRecordSets @httpRequestTests([
    {
        id: "ListResourceRecordSetsTrimHostedZone",
        documentation: "This test validates that hosted zone is correctly trimmed",
        method: "GET",
        protocol: "aws.protocols#restXml",
        uri: "/2013-04-01/hostedzone/IDOFMYHOSTEDZONE/rrset",
        bodyMediaType: "application/xml",
        params: {
            "HostedZoneId": "/hostedzone/IDOFMYHOSTEDZONE"
        }
        vendorParams: {
            "endpointParams": {
                "builtInParams": {
                    "AWS::Region": "us-east-1"
                }
            }
        }
    }
])

apply GetChange @httpRequestTests([
    {
        id: "GetChangeTrimChangeId",
        documentation: "This test validates that change id is correctly trimmed",
        method: "GET",
        protocol: "aws.protocols#restXml",
        uri: "/2013-04-01/change/SOMECHANGEID",
        bodyMediaType: "application/xml",
        params: {
            "Id": "/change/SOMECHANGEID"
        }
        vendorParams: {
            "endpointParams": {
                "builtInParams": {
                    "AWS::Region": "us-east-1"
                }
            }
        }
    },
])

apply GetReusableDelegationSet @httpRequestTests([
    {
        id: "GetReusableDelegationSetTrimDelegationSetId",
        documentation: "This test validates that delegation set id is correctly trimmed",
        method: "GET",
        protocol: "aws.protocols#restXml",
        uri: "/2013-04-01/delegationset/DELEGATIONSETID",
        bodyMediaType: "application/xml",
        params: {
            "Id": "/delegationset/DELEGATIONSETID"
        }
        vendorParams: {
            "endpointParams": {
                "builtInParams": {
                    "AWS::Region": "us-east-1"
                }
            }
        }
    },
])
