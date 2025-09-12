/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind},
    net::IpAddr,
    time::Duration,
};

use aws_smithy_runtime_api::client::dns::{DnsFuture, ResolveDns, ResolveDnsError};
use hickory_resolver::{
    config::{NameServerConfigGroup, ResolverConfig},
    name_server::TokioConnectionProvider,
    Resolver,
};

/// DNS resolver that uses [hickory_resolver] and caches DNS entries in memory.
///
/// This resolver requires a [tokio] runtime to function and isn't available for WASM targets.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct HickoryDnsResolver {
    resolver: Resolver<TokioConnectionProvider>,
}

impl Default for HickoryDnsResolver {
    /// Constructs a new Tokio based [ResolveDns] with the system configuration.
    /// This uses `/etc/resolv.conf` on Unix OSes and registry settings on Windows.
    fn default() -> Self {
        Self {
            resolver: Resolver::builder_tokio().expect("In tokio runtime").build(),
        }
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
                Err(failure) => Err(ResolveDnsError::new(IoError::new(
                    IoErrorKind::Other,
                    failure,
                ))),
            }
        })
    }
}

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

    /// Specify the timeout for a request. Defaults to 5 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Number of retries after lookup failure before giving up. Defaults to 2.
    pub fn attempts(mut self, attempts: usize) -> Self {
        self.attempts = Some(attempts);
        self
    }

    /// Cache size is in number of records (some records can be large). Defaults to 32.
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

    pub fn build(self) -> HickoryDnsResolver {
        let mut builder = if let Some(nameservers) = self.nameservers {
            let nameserver_config =
                NameServerConfigGroup::from_ips_clear(&nameservers.ips, nameservers.port, true);
            let resolver_config = ResolverConfig::from_parts(None, vec![], nameserver_config);

            Resolver::builder_with_config(resolver_config, TokioConnectionProvider::default())
        } else {
            Resolver::builder_tokio().expect("Successfully read system config")
        };

        let opts = builder.options_mut();

        if let Some(timeout) = self.timeout {
            opts.timeout = timeout;
        }

        if let Some(attempts) = self.attempts {
            opts.attempts = attempts;
        }

        if let Some(cache_size) = self.cache_size {
            opts.cache_size = cache_size;
        }

        if let Some(num_concurrent_reqs) = self.num_concurrent_reqs {
            opts.num_concurrent_reqs = num_concurrent_reqs;
        }

        HickoryDnsResolver {
            resolver: builder.build(),
        }
    }
}
