/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg(all(feature = "hickory-dns", not(target_family = "wasm")))]

use aws_smithy_dns::HickoryDnsResolver;
use aws_smithy_runtime_api::client::dns::ResolveDns;
use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};
use tokio::test;

#[test]
async fn test_dns_caching() {
    let dns_server = test_dns_server::setup_dns_server().await;
    let (dns_ip, dns_port) = dns_server.addr();

    let resolver = HickoryDnsResolver::builder()
        .nameservers(&[dns_ip], dns_port)
        .cache_size(1)
        .build();

    let hostname = "example.com";

    // First resolution should hit the server
    let first_result = resolver.resolve_dns(hostname).await;
    let first_ips = first_result.unwrap();

    // Verify correct IP returned and server hit to get it
    assert_eq!(vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))], first_ips);
    assert!(dns_server.query_count() == 1);

    // Second resolution should hit the cache
    let second_result = resolver.resolve_dns(hostname).await;
    let second_ips = second_result.unwrap();

    // Verify second resolution hit cache, not server
    assert!(dns_server.query_count() == 1);

    // Verify same IPs returned
    assert_eq!(first_ips, second_ips);

    // Queries to explicitly configured nameservers carry no EDNS(0) record
    assert_eq!(0, dns_server.edns_query_count());
}

#[test]
async fn test_dns_cache_size_limit() {
    let dns_server = test_dns_server::setup_dns_server().await;
    let (dns_ip, dns_port) = dns_server.addr();

    let resolver = HickoryDnsResolver::builder()
        .nameservers(&[dns_ip], dns_port)
        .cache_size(1)
        .build();

    // First resolution should hit the server
    let _first_result = resolver.resolve_dns("example.com").await;

    // Verify server hit
    assert!(dns_server.query_count() == 1);

    // Second resolution should hit the server
    let _second_result = resolver.resolve_dns("aws.com").await;

    // Verify server hit
    assert!(dns_server.query_count() == 2);

    // Third resolution should hit the server
    let _third_result = resolver.resolve_dns("foo.com").await;

    // Verify server hit
    assert!(dns_server.query_count() == 3);

    // Third result should now be in cache
    let _third_result = resolver.resolve_dns("foo.com").await;

    // Verify server not hit in favor of the cache
    assert!(dns_server.query_count() == 3);

    // First result should have been removed from the cache, so querying it again should hit server
    let _first_result_again = resolver.resolve_dns("example.com").await;

    // Verify server hit
    assert!(dns_server.query_count() == 4);
}

#[test]
async fn test_dns_error_handling() {
    let dns_server = test_dns_server::setup_dns_server().await;
    let (dns_ip, dns_port) = dns_server.addr();

    let resolver = HickoryDnsResolver::builder()
        .nameservers(&[dns_ip], dns_port)
        .timeout(Duration::from_millis(100))
        .attempts(1)
        .build();

    // Try to resolve an invalid hostname
    let result = resolver
        .resolve_dns("invalid.nonexistent.domain.test")
        .await;
    assert!(result.is_err());
}

// Kind of janky minimal test utility for creating a local DNS server
#[cfg(test)]
mod test_dns_server {
    use std::{
        collections::{HashMap, HashSet},
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::{atomic::AtomicUsize, Arc},
        time::Duration,
    };
    use tokio::{net::UdpSocket, sync::Notify, task::JoinHandle};

    pub async fn setup_dns_server() -> TestDnsServer {
        let mut records = HashMap::new();
        records.insert(
            "example.com".to_string(),
            IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
        );
        records.insert("aws.com".to_string(), IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8)));
        records.insert(
            "foo.com".to_string(),
            IpAddr::V4(Ipv4Addr::new(9, 10, 11, 12)),
        );

        TestDnsServer::start(records).await.unwrap()
    }

    pub struct TestDnsServer {
        handle: JoinHandle<()>,
        addr: SocketAddr,
        shutdown: Arc<Notify>,
        query_count: Arc<AtomicUsize>,
        edns_query_count: Arc<AtomicUsize>,
    }

    impl TestDnsServer {
        pub async fn start(
            records: HashMap<String, IpAddr>,
        ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            // localhost, random port
            let socket = Arc::new(UdpSocket::bind("127.0.0.1:0").await?);
            let addr = socket.local_addr()?;

            let shutdown = Arc::new(Notify::new());
            let shutdown_clone = shutdown.clone();
            let query_count = Arc::new(AtomicUsize::new(0));
            let query_count_clone = query_count.clone();
            let edns_query_count = Arc::new(AtomicUsize::new(0));
            let edns_query_count_clone = edns_query_count.clone();
            let records = Arc::new(records);

            let handle = tokio::spawn(async move {
                let mut buf = [0; 512];
                // The resolver retransmits a UDP query under the same message ID while the
                // response is outstanding, so a lookup is counted once per distinct
                // (ID, name, type).
                let mut seen_queries = HashSet::new();
                loop {
                    tokio::select! {
                        _ = shutdown_clone.notified() => break,
                        result = socket.recv_from(&mut buf) => {
                            if let Ok((len, src)) = result {
                                let query = buf[..len].to_vec();
                                if let Some(parsed) = DnsQuery::parse(&query) {
                                    if seen_queries.insert((parsed.id, parsed.domain, parsed.query_type)) {
                                        query_count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    }
                                    if parsed.additional_count > 0 {
                                        edns_query_count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    }
                                }
                                // Each response is sent from its own task so that retransmitted
                                // queries do not queue behind the simulated latency.
                                let socket = socket.clone();
                                let records = records.clone();
                                tokio::spawn(async move {
                                    // Short sleep before returning DNS response to simulate network latency
                                    tokio::time::sleep(Duration::from_millis(1000)).await;
                                    let response = create_dns_response(&query, &records);
                                    let _ = socket.send_to(&response, src).await;
                                });
                            }
                        }
                    }
                }
            });

            Ok(TestDnsServer {
                handle,
                addr,
                shutdown,
                query_count,
                edns_query_count,
            })
        }

        pub fn addr(&self) -> (IpAddr, u16) {
            (self.addr.ip(), self.addr.port())
        }

        pub fn query_count(&self) -> usize {
            self.query_count.load(std::sync::atomic::Ordering::Relaxed)
        }

        /// Number of datagrams received with a non-empty additional section (an EDNS(0) OPT record).
        pub fn edns_query_count(&self) -> usize {
            self.edns_query_count
                .load(std::sync::atomic::Ordering::Relaxed)
        }
    }

    impl Drop for TestDnsServer {
        fn drop(&mut self) {
            self.shutdown.notify_one();
            self.handle.abort();
        }
    }

    fn create_dns_response(query: &[u8], records: &HashMap<String, IpAddr>) -> Vec<u8> {
        let parsed = DnsQuery::parse(query).unwrap_or_default();
        let ip = records.get(&parsed.domain).copied().unwrap();

        let response = DnsResponse {
            id: parsed.id,
            flags: 0x8180, // Standard response flags
            question: parsed.domain,
            answer_ip: ip,
            ttl: 300,
        };

        response.to_bytes()
    }

    #[derive(Debug, Default)]
    #[allow(dead_code)]
    struct DnsQuery {
        id: u16,
        flags: u16,
        question_count: u16,
        additional_count: u16,
        domain: String,
        query_type: u16,
        query_class: u16,
    }

    impl DnsQuery {
        fn parse(data: &[u8]) -> Option<Self> {
            if data.len() < 12 {
                return None;
            }

            let id = u16::from_be_bytes([data[0], data[1]]);
            let flags = u16::from_be_bytes([data[2], data[3]]);
            let question_count = u16::from_be_bytes([data[4], data[5]]);
            let additional_count = u16::from_be_bytes([data[10], data[11]]);

            if question_count == 0 {
                return None;
            }

            // Parse domain name starting at byte 12
            let mut pos = 12;
            let mut domain = String::new();

            while pos < data.len() {
                let len = data[pos] as usize;
                if len == 0 {
                    pos += 1;
                    break;
                }

                if !domain.is_empty() {
                    domain.push('.');
                }

                pos += 1;
                if pos + len > data.len() {
                    return None;
                }

                if let Ok(label) = std::str::from_utf8(&data[pos..pos + len]) {
                    domain.push_str(label);
                }
                pos += len;
            }

            if pos + 4 > data.len() {
                return None;
            }

            let query_type = u16::from_be_bytes([data[pos], data[pos + 1]]);
            let query_class = u16::from_be_bytes([data[pos + 2], data[pos + 3]]);

            Some(DnsQuery {
                id,
                flags,
                question_count,
                additional_count,
                domain,
                query_type,
                query_class,
            })
        }
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct DnsResponse {
        id: u16,
        flags: u16,
        question: String,
        answer_ip: IpAddr,
        ttl: u32,
    }

    impl DnsResponse {
        fn to_bytes(&self) -> Vec<u8> {
            // 30ish required bytes, 11 more added for the question section
            // since the longest domain we currently use is 11 bytes long
            let mut response = Vec::with_capacity(41);

            // Header (12 bytes) all values besides id/flags hardcoded
            response.extend_from_slice(&self.id.to_be_bytes());
            response.extend_from_slice(&self.flags.to_be_bytes());
            response.extend_from_slice(&1u16.to_be_bytes()); // Questions: 1
            response.extend_from_slice(&1u16.to_be_bytes()); // Answers: 1
            response.extend_from_slice(&0u16.to_be_bytes()); // Authority: 0
            response.extend_from_slice(&0u16.to_be_bytes()); // Additional: 0

            // Question section
            // In a more ideal world the DnsResponse would contain a ref to the
            // DnsQuery that triggered this response and recreate the question section
            // from that
            for label in self.question.split('.') {
                response.push(label.len() as u8);
                response.extend_from_slice(label.as_bytes());
            }
            response.push(0); // End of name
            response.extend_from_slice(&1u16.to_be_bytes()); // Type A
            response.extend_from_slice(&1u16.to_be_bytes()); // Class IN

            // Answer section
            response.extend_from_slice(&[0xc0, 0x0c]); // Name pointer
            response.extend_from_slice(&1u16.to_be_bytes()); // Type A
            response.extend_from_slice(&1u16.to_be_bytes()); // Class IN
            response.extend_from_slice(&self.ttl.to_be_bytes()); // TTL

            match self.answer_ip {
                IpAddr::V4(ipv4) => {
                    response.extend_from_slice(&4u16.to_be_bytes()); // Data length
                    response.extend_from_slice(&ipv4.octets());
                }
                IpAddr::V6(_) => {
                    // Unsupported, fallback to IPv4
                    response.extend_from_slice(&4u16.to_be_bytes());
                    response.extend_from_slice(&[127, 0, 0, 1]);
                }
            }

            response
        }
    }
}
