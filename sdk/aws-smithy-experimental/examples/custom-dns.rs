/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_experimental::hyper_1_0::{CryptoMode, HyperClientBuilder};
use aws_smithy_runtime_api::client::dns::{DnsFuture, ResolveDns};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone)]
struct StaticResolver;

impl ResolveDns for StaticResolver {
    fn resolve_dns<'a>(&'a self, _name: &'a str) -> DnsFuture<'a> {
        DnsFuture::ready(Ok(vec![IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))]))
    }
}

fn main() {
    let _client = HyperClientBuilder::new()
        .crypto_mode(CryptoMode::Ring)
        .build_with_resolver(StaticResolver);
}
