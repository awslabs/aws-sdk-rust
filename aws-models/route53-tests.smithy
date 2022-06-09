$version: "1.0"

namespace com.amazonaws.route53

use smithy.test#httpRequestTests


apply ListResourceRecordSets @httpRequestTests([
    {
        id: "ListResourceRecordSetsTrimHostdZone",
        documentation: "This test validates that that hosted zone is correctly trimmed",
        method: "GET",
        protocol: "aws.protocols#restXml",
        uri: "/2013-04-01/hostedzone/IDOFMYHOSTEDZONE/rrset",
        bodyMediaType: "application/xml",
        params: {
            "HostedZoneId": "/hostedzone/IDOFMYHOSTEDZONE"
        }
    }
])
