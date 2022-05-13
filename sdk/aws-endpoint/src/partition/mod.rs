/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod endpoint;

use aws_types::endpoint::{AwsEndpoint, BoxError, ResolveAwsEndpoint};
use aws_types::region::Region;
use regex::Regex;
use std::collections::HashMap;
use std::iter;

/// Root level resolver for an AWS Service
///
/// PartitionResolver resolves the endpoint for an AWS Service. Each partition will be checked
/// in turn, checking if the partition [can resolve](Partition::can_resolve) the given region. If
/// no regions match, `base` is used.
///
/// Once a partition has been identified, endpoint resolution is delegated to the underlying
/// partition.
#[derive(Debug)]
pub struct PartitionResolver {
    /// Base partition used if no partitions match the region regex
    base: Partition,

    // base and rest are split so that we can validate that at least 1 partition is defined
    // at compile time.
    rest: Vec<Partition>,
}

impl PartitionResolver {
    /// Construct a new  `PartitionResolver` from a list of partitions
    pub fn new(base: Partition, rest: Vec<Partition>) -> Self {
        Self { base, rest }
    }

    fn partitions(&self) -> impl Iterator<Item = &Partition> {
        iter::once(&self.base).chain(self.rest.iter())
    }
}

impl ResolveAwsEndpoint for PartitionResolver {
    fn resolve_endpoint(&self, region: &Region) -> Result<AwsEndpoint, BoxError> {
        let matching_partition = self
            .partitions()
            .find(|partition| partition.can_resolve(region))
            .unwrap_or(&self.base);
        matching_partition.resolve_endpoint(region)
    }
}

#[derive(Debug)]
pub struct Partition {
    _id: &'static str,
    region_regex: Regex,
    partition_endpoint: Option<Region>,
    regionalized: Regionalized,
    default_endpoint: endpoint::Metadata,
    endpoints: HashMap<Region, endpoint::Metadata>,
}

#[derive(Default)]
pub struct Builder {
    id: Option<&'static str>,
    region_regex: Option<Regex>,
    partition_endpoint: Option<Region>,
    regionalized: Option<Regionalized>,
    default_endpoint: Option<endpoint::Metadata>,
    endpoints: HashMap<Region, endpoint::Metadata>,
}

impl Builder {
    pub fn id(mut self, id: &'static str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn default_endpoint(mut self, default: endpoint::Metadata) -> Self {
        self.default_endpoint = Some(default);
        self
    }

    pub fn region_regex(mut self, regex: &'static str) -> Self {
        // We use a stripped down version of the regex crate without unicode support
        // To support `\d` and `\w`, we need to explicitly opt into the ascii-only version.
        let ascii_only = regex
            .replace("\\d", "(?-u:\\d)")
            .replace("\\w", "(?-u:\\w)");
        self.region_regex = Some(Regex::new(&ascii_only).expect("invalid regex"));
        self
    }

    pub fn partition_endpoint(mut self, partition_endpoint: &'static str) -> Self {
        self.partition_endpoint = Some(Region::new(partition_endpoint));
        self
    }

    pub fn regionalized(mut self, regionalized: Regionalized) -> Self {
        self.regionalized = Some(regionalized);
        self
    }

    pub fn endpoint(mut self, region: &'static str, endpoint: endpoint::Metadata) -> Self {
        self.endpoints.insert(Region::new(region), endpoint);
        self
    }

    /// Construct a Partition from the builder
    ///
    /// Returns `None` if:
    /// - DefaultEndpoint is not set
    /// - DefaultEndpoint has an empty list of supported signature versions
    pub fn build(self) -> Option<Partition> {
        let default_endpoint = self.default_endpoint?;
        let endpoints = self.endpoints.into_iter().collect();
        Some(Partition {
            _id: self.id?,
            region_regex: self.region_regex?,
            partition_endpoint: self.partition_endpoint,
            regionalized: self.regionalized.unwrap_or_default(),
            default_endpoint,
            endpoints,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Regionalized {
    Regionalized,
    NotRegionalized,
}

impl Default for Regionalized {
    fn default() -> Self {
        Regionalized::Regionalized
    }
}

impl Partition {
    pub fn can_resolve(&self, region: &Region) -> bool {
        self.region_regex.is_match(region.as_ref())
    }

    pub fn builder() -> Builder {
        Builder::default()
    }
}

impl ResolveAwsEndpoint for Partition {
    fn resolve_endpoint(&self, region: &Region) -> Result<AwsEndpoint, BoxError> {
        if let Some(endpoint) = self.endpoints.get(region) {
            return endpoint.resolve_endpoint(region);
        }
        let resolved_region = match self.regionalized {
            Regionalized::NotRegionalized => self.partition_endpoint.as_ref(),
            Regionalized::Regionalized => Some(region),
        };
        let endpoint_for_region = resolved_region
            .and_then(|region| self.endpoints.get(region))
            .unwrap_or(&self.default_endpoint);
        endpoint_for_region.resolve_endpoint(region)
    }
}

#[cfg(test)]
mod test {
    use crate::partition::endpoint::Metadata;
    use crate::partition::endpoint::Protocol::{Http, Https};
    use crate::partition::endpoint::SignatureVersion::{self, V4};
    use crate::partition::{endpoint, Partition};
    use crate::partition::{PartitionResolver, Regionalized};
    use crate::{CredentialScope, ResolveAwsEndpoint};
    use aws_types::region::{Region, SigningRegion};
    use aws_types::SigningService;
    use http::Uri;

    fn basic_partition() -> Partition {
        Partition::builder()
            .id("part-id-1")
            .region_regex(r#"^(us)-\w+-\d+$"#)
            .default_endpoint(endpoint::Metadata {
                uri_template: "service.{region}.amazonaws.com",
                protocol: Https,
                credential_scope: CredentialScope::default(),
                signature_versions: SignatureVersion::V4,
            })
            .partition_endpoint("")
            .regionalized(Regionalized::Regionalized)
            .endpoint(
                "us-west-1",
                endpoint::Metadata {
                    uri_template: "service.{region}.amazonaws.com",
                    protocol: Https,
                    credential_scope: CredentialScope::default(),
                    signature_versions: SignatureVersion::V4,
                },
            )
            .endpoint(
                "us-west-1-alt",
                Metadata {
                    uri_template: "service-alt.us-west-1.amazonaws.com",
                    protocol: Http,
                    credential_scope: CredentialScope::builder()
                        .region(SigningRegion::from_static("us-west-1"))
                        .service(SigningService::from_static("foo"))
                        .build(),
                    signature_versions: V4,
                },
            )
            .build()
            .expect("valid partition")
    }

    fn global_partition() -> Partition {
        Partition::builder()
            .id("part-id-1")
            .region_regex(r#"^(cn)-\w+-\d+$"#)
            .default_endpoint(Metadata {
                uri_template: "service.{region}.amazonaws.com",
                protocol: Https,
                credential_scope: CredentialScope::builder()
                    .service(SigningService::from_static("foo"))
                    .build(),
                signature_versions: SignatureVersion::V4,
            })
            .partition_endpoint("partition")
            .regionalized(Regionalized::NotRegionalized)
            .endpoint(
                "partition",
                Metadata {
                    uri_template: "some-global-thing.amazonaws.cn",
                    protocol: Https,
                    credential_scope: CredentialScope::builder()
                        .region(SigningRegion::from_static("cn-east-1"))
                        .service(SigningService::from_static("foo"))
                        .build(),
                    signature_versions: SignatureVersion::V4,
                },
            )
            .endpoint(
                "cn-fips-1",
                Metadata {
                    uri_template: "fips.amazonaws.cn",
                    protocol: Https,
                    credential_scope: CredentialScope::builder()
                        .region(SigningRegion::from_static("cn-fips"))
                        .build(),
                    signature_versions: SignatureVersion::V4,
                },
            )
            .build()
            .expect("valid partition")
    }

    fn partition_resolver() -> PartitionResolver {
        PartitionResolver::new(
            basic_partition(),
            vec![global_partition(), default_partition()],
        )
    }

    fn default_partition() -> Partition {
        Partition::builder()
            .id("part-id-3")
            .region_regex(r#"^(eu)-\w+-\d+$"#)
            .default_endpoint(Metadata {
                uri_template: "service.{region}.amazonaws.com",
                protocol: Https,
                signature_versions: V4,
                credential_scope: CredentialScope::builder()
                    .service(SigningService::from_static("foo"))
                    .build(),
            })
            .build()
            .expect("valid partition")
    }

    struct TestCase {
        region: &'static str,
        uri: &'static str,
        signing_region: &'static str,
        signing_service: Option<&'static str>,
    }

    /// Modeled region with no endpoint overrides
    const MODELED_REGION: TestCase = TestCase {
        region: "us-west-1",
        uri: "https://service.us-west-1.amazonaws.com",
        signing_region: "us-west-1",
        signing_service: None,
    };

    /// Modeled region with endpoint overrides
    const MODELED_REGION_OVERRIDE: TestCase = TestCase {
        region: "us-west-1-alt",
        uri: "http://service-alt.us-west-1.amazonaws.com",
        signing_region: "us-west-1",
        signing_service: Some("foo"),
    };

    /// Validates falling back onto the default endpoint
    const FALLBACK_REGION: TestCase = TestCase {
        region: "us-east-1",
        uri: "https://service.us-east-1.amazonaws.com",
        signing_region: "us-east-1",
        signing_service: None,
    };

    /// Validates "PartitionName"
    const PARTITION_NAME: TestCase = TestCase {
        region: "cn-central-1",
        uri: "https://some-global-thing.amazonaws.cn",
        signing_region: "cn-east-1",
        signing_service: Some("foo"),
    };

    /// Validates non-regionalized endpoints still use endpoints
    const NON_REGIONALIZED_EXACT_MATCH: TestCase = TestCase {
        region: "cn-fips-1",
        uri: "https://fips.amazonaws.cn",
        signing_region: "cn-fips",
        signing_service: None,
    };

    const DEFAULT_ENDPOINT: TestCase = TestCase {
        region: "eu-west-1",
        uri: "https://service.eu-west-1.amazonaws.com",
        signing_region: "eu-west-1",
        signing_service: Some("foo"),
    };

    const TEST_CASES: &[TestCase] = &[
        MODELED_REGION,
        MODELED_REGION_OVERRIDE,
        FALLBACK_REGION,
        PARTITION_NAME,
        DEFAULT_ENDPOINT,
        NON_REGIONALIZED_EXACT_MATCH,
    ];

    #[test]
    fn validate_basic_partition() {
        let p10n = basic_partition();
        check_endpoint(&p10n, &MODELED_REGION);
        check_endpoint(&p10n, &MODELED_REGION_OVERRIDE);
        check_endpoint(&p10n, &FALLBACK_REGION);
    }

    #[test]
    fn validate_global_partition() {
        let partition = global_partition();
        check_endpoint(&partition, &PARTITION_NAME);
        check_endpoint(&partition, &NON_REGIONALIZED_EXACT_MATCH)
    }

    #[test]
    fn validate_default_endpoint() {
        check_endpoint(&default_partition(), &DEFAULT_ENDPOINT);
    }

    #[test]
    fn validate_partition_resolver() {
        let resolver = partition_resolver();
        for test_case in TEST_CASES {
            check_endpoint(&resolver, test_case);
        }
    }

    #[track_caller]
    fn check_endpoint(resolver: &impl ResolveAwsEndpoint, test_case: &TestCase) {
        let endpoint = resolver
            .resolve_endpoint(&Region::new(test_case.region))
            .expect("valid region");
        let mut test_uri = Uri::from_static("/");
        endpoint.set_endpoint(&mut test_uri, None);
        assert_eq!(test_uri, Uri::from_static(test_case.uri));
        assert_eq!(
            endpoint.credential_scope().region(),
            Some(&SigningRegion::from_static(test_case.signing_region))
        );
        assert_eq!(
            endpoint.credential_scope().service(),
            test_case
                .signing_service
                .map(SigningService::from_static)
                .as_ref()
        )
    }
}
