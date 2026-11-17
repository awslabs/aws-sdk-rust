/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::{io::Error as IoError, net::IpAddr, time::Duration};

use aws_smithy_runtime_api::client::dns::{DnsFuture, ResolveDns, ResolveDnsError};
use hickory_resolver::{
    config::{ConnectionConfig, LookupIpStrategy, NameServerConfig, ResolverConfig},
    net::runtime::TokioRuntimeProvider,
    TokioResolver,
};

/// Number of responses cached when [`HickoryDnsResolverBuilder::cache_size`] is not set.
const DEFAULT_CACHE_SIZE: u64 = 32;

/// DNS resolver that uses [hickory_resolver] and caches DNS entries in memory.
///
/// Lookups query for IPv4 (`A`) records and query for IPv6 (`AAAA`) records only when the
/// IPv4 lookup fails.
///
/// Queries carry an EDNS(0) record only when the system configuration is read from
/// `/etc/resolv.conf` and it sets `options edns0`.
///
/// This resolver requires a [tokio] runtime to function and isn't available for WASM targets.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct HickoryDnsResolver {
    resolver: TokioResolver,
}

impl Default for HickoryDnsResolver {
    /// Constructs a new Tokio based [ResolveDns] with the system configuration.
    ///
    /// The system configuration is read from the System Configuration framework on Apple
    /// platforms, from the `ConnectivityManager` on Android, from the registry on Windows, and
    /// from `/etc/resolv.conf` on other Unix OSes.
    ///
    /// # Panics
    ///
    /// Panics under the same conditions as [`HickoryDnsResolverBuilder::build`].
    fn default() -> Self {
        Self::builder().build()
    }
}

impl HickoryDnsResolver {
    /// Creates a new DNS resolver that caches IP addresses in memory.
    pub fn builder() -> HickoryDnsResolverBuilder {
        HickoryDnsResolverBuilder {
            nameservers: None,
            timeout: None,
            attempts: None,
            cache_size: None,
            num_concurrent_reqs: None,
        }
    }

    /// Flush the cache
    pub fn clear_cache(&self) {
        self.resolver.clear_cache();
    }
}

impl ResolveDns for HickoryDnsResolver {
    fn resolve_dns<'a>(&'a self, name: &'a str) -> DnsFuture<'a> {
        DnsFuture::new(async move {
            let result = self.resolver.lookup_ip(name).await;

            match result {
                Ok(ips) => Ok(ips.into_iter().collect()),
                Err(failure) => Err(ResolveDnsError::new(IoError::other(failure))),
            }
        })
    }
}

/// Builder for [HickoryDnsResolver]
pub struct HickoryDnsResolverBuilder {
    nameservers: Option<Nameservers>,
    timeout: Option<Duration>,
    attempts: Option<usize>,
    cache_size: Option<usize>,
    num_concurrent_reqs: Option<usize>,
}

struct Nameservers {
    ips: Vec<IpAddr>,
    port: u16,
}

impl HickoryDnsResolverBuilder {
    /// Configure upstream nameservers and the port to use for resolution. Defaults to the system
    /// configuration.
    pub fn nameservers(mut self, ips: &[IpAddr], port: u16) -> Self {
        self.nameservers = Some(Nameservers {
            ips: ips.to_vec(),
            port,
        });
        self
    }

    /// Specify the timeout for one lookup attempt, which bounds every nameserver queried in that
    /// attempt. Defaults to 5 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Number of retries after lookup failure before giving up. Defaults to 2.
    pub fn attempts(mut self, attempts: usize) -> Self {
        self.attempts = Some(attempts);
        self
    }

    /// Cache size is in number of responses (some responses can be large). Defaults to 32.
    pub fn cache_size(mut self, cache_size: usize) -> Self {
        self.cache_size = Some(cache_size);
        self
    }

    /// Number of concurrent requests per query.
    ///
    /// Where more than one nameserver is configured, this configures the resolver
    /// to send queries to a number of servers in parallel. Defaults to 2. Setting
    /// to 0 or 1 will execute requests serially.
    pub fn num_concurrent_reqs(mut self, num_concurrent_reqs: usize) -> Self {
        self.num_concurrent_reqs = Some(num_concurrent_reqs);
        self
    }

    /// Build a [HickoryDnsResolver]
    ///
    /// # Panics
    ///
    /// Panics if no nameservers were configured and the system configuration cannot be read,
    /// or if `hickory-resolver` fails to construct the resolver.
    pub fn build(self) -> HickoryDnsResolver {
        // Whether `read_system_conf` takes `edns0` from `/etc/resolv.conf` on this platform.
        let system_sets_edns0 = cfg!(all(
            unix,
            not(any(target_os = "android", target_vendor = "apple"))
        ));
        let (mut builder, edns0_from_config) = if let Some(nameservers) = self.nameservers {
            let name_servers = nameservers
                .ips
                .iter()
                .map(|ip| {
                    let mut udp = ConnectionConfig::udp();
                    udp.port = nameservers.port;
                    let mut tcp = ConnectionConfig::tcp();
                    tcp.port = nameservers.port;
                    NameServerConfig::new(*ip, true, vec![udp, tcp])
                })
                .collect();
            let resolver_config = ResolverConfig::from_name_servers(name_servers);

            (
                TokioResolver::builder_with_config(
                    resolver_config,
                    TokioRuntimeProvider::default(),
                ),
                false,
            )
        } else {
            (
                TokioResolver::builder_tokio().expect("Successfully read system config"),
                system_sets_edns0,
            )
        };

        let opts = builder.options_mut();
        // hickory-resolver 0.26 defaults to querying `A` and `AAAA` in parallel; the documented
        // contract of this resolver is IPv4 first with IPv6 as a fallback.
        opts.ip_strategy = LookupIpStrategy::Ipv4thenIpv6;
        // hickory-resolver 0.26 enables EDNS(0) in its default options and treats a FORMERR
        // response as final, so a server that rejects OPT records would fail every lookup.
        if !edns0_from_config {
            opts.edns0 = false;
        }

        if let Some(timeout) = self.timeout {
            opts.timeout = timeout;
        }

        if let Some(attempts) = self.attempts {
            opts.attempts = attempts;
        }

        opts.cache_size = self.cache_size.map_or(DEFAULT_CACHE_SIZE, |size| {
            u64::try_from(size).unwrap_or(u64::MAX)
        });

        if let Some(num_concurrent_reqs) = self.num_concurrent_reqs {
            opts.num_concurrent_reqs = num_concurrent_reqs;
        }

        HickoryDnsResolver {
            resolver: builder.build().expect("Successfully built resolver"),
        }
    }
}
