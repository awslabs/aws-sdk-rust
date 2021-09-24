//! IMDS Region Provider
//!
//! Load region from IMDS from `/latest/meta-data/placement/region`
//! This provider has a 5 second timeout.

use crate::imds;
use crate::imds::client::LazyClient;
use crate::meta::region::{future, ProvideRegion};
use crate::provider_config::ProviderConfig;
use aws_types::os_shim_internal::Env;
use aws_types::region::Region;
use smithy_async::future::timeout::Timeout;
use smithy_async::rt::sleep::AsyncSleep;
use std::sync::Arc;
use std::time::Duration;
use tracing::Instrument;

/// IMDSv2 Region Provider
///
/// This provider is included in the default region chain, so it does not need to be used manually.
///
/// This provider has a 5 second timeout.
#[derive(Debug)]
pub struct ImdsRegionProvider {
    client: LazyClient,
    sleep: Arc<dyn AsyncSleep>,
    env: Env,
}

const REGION_PATH: &str = "/latest/meta-data/placement/region";

impl ImdsRegionProvider {
    /// Builder for [`ImdsRegionProvider`]
    pub fn builder() -> Builder {
        Builder::default()
    }

    fn imds_disabled(&self) -> bool {
        match self.env.get(super::env::EC2_METADATA_DISABLED) {
            Ok(value) => value.eq_ignore_ascii_case("true"),
            _ => false,
        }
    }

    /// Load a region from IMDS
    ///
    /// This provider uses the API `/latest/meta-data/placement/region`
    pub async fn region(&self) -> Option<Region> {
        if self.imds_disabled() {
            return None;
        }
        let client = self.client.client().await.ok()?;
        // TODO: IMDS clients should use a 1 second connect timeout, we shouldn't add an external timeout.
        // There isn't a generalized way to know when you are inside of EC2 and IMDS will work. In
        // the case where a customer doesn't have a region provider configured, we don't want the
        // SDK to hang forever trying to load configuration, so we need a timeout to account for
        // the IMDS provider running when IMDS isn't available. 5 seconds is a compromise since we
        // need to make multiple e2e requests to IMDS to actually load a region.
        let timeout_fut = Timeout::new(
            client.get(REGION_PATH),
            self.sleep.sleep(Duration::from_secs(5)),
        );
        let imds_result = match timeout_fut.await {
            Ok(res) => res,
            Err(_) => {
                tracing::warn!("imds timed out after 5 seconds");
                return None;
            }
        };
        match imds_result {
            Ok(region) => {
                tracing::info!(region = % region, "loaded region from IMDS");
                Some(Region::new(region))
            }
            Err(err) => {
                tracing::warn!(err = % err, "failed to load region from IMDS");
                None
            }
        }
    }
}

impl ProvideRegion for ImdsRegionProvider {
    fn region(&self) -> future::ProvideRegion {
        future::ProvideRegion::new(
            self.region()
                .instrument(tracing::info_span!("imds_load_region")),
        )
    }
}

/// Builder for [`ImdsRegionProvider`]
#[derive(Default)]
pub struct Builder {
    provider_config: Option<ProviderConfig>,
    imds_client_override: Option<imds::Client>,
}

impl Builder {
    /// Set configuration options of the [`Builder`]
    pub fn configure(self, provider_config: &ProviderConfig) -> Self {
        Self {
            provider_config: Some(provider_config.clone()),
            ..self
        }
    }

    /// Override the IMDS client used to load the region
    pub fn imds_client(mut self, imds_client: imds::Client) -> Self {
        self.imds_client_override = Some(imds_client);
        self
    }

    /// Create an [`ImdsRegionProvider`] from this builder
    pub fn build(self) -> ImdsRegionProvider {
        let provider_config = self.provider_config.unwrap_or_default();
        let client = self
            .imds_client_override
            .map(LazyClient::from_ready_client)
            .unwrap_or_else(|| {
                imds::Client::builder()
                    .configure(&provider_config)
                    .build_lazy()
            });
        ImdsRegionProvider {
            client,
            env: provider_config.env(),
            sleep: provider_config
                .sleep()
                .expect("no default sleep implementation provided"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::imds::client::test::{imds_request, imds_response, token_request, token_response};
    use crate::imds::region::ImdsRegionProvider;
    use crate::provider_config::ProviderConfig;
    use aws_hyper::DynConnector;
    use aws_sdk_sts::Region;
    use smithy_async::rt::sleep::TokioSleep;
    use smithy_client::test_connection::TestConnection;
    use smithy_http::body::SdkBody;
    use tracing_test::traced_test;

    #[tokio::test]
    async fn load_region() {
        let conn = TestConnection::new(vec![
            (
                token_request("http://169.254.169.254", 21600),
                token_response(21600, "token"),
            ),
            (
                imds_request(
                    "http://169.254.169.254/latest/meta-data/placement/region",
                    "token",
                ),
                imds_response("eu-west-1"),
            ),
        ]);
        let provider = ImdsRegionProvider::builder()
            .configure(
                &ProviderConfig::no_configuration()
                    .with_connector(DynConnector::new(conn))
                    .with_sleep(TokioSleep::new()),
            )
            .build();
        assert_eq!(
            provider.region().await.expect("returns region"),
            Region::new("eu-west-1")
        );
    }

    #[traced_test]
    #[tokio::test]
    async fn no_region_imds_disabled() {
        let conn = TestConnection::new(vec![(
            token_request("http://169.254.169.254", 21600),
            http::Response::builder()
                .status(403)
                .body(SdkBody::empty())
                .unwrap(),
        )]);
        let provider = ImdsRegionProvider::builder()
            .configure(
                &ProviderConfig::no_configuration()
                    .with_connector(DynConnector::new(conn))
                    .with_sleep(TokioSleep::new()),
            )
            .build();
        assert_eq!(provider.region().await, None);
        assert!(logs_contain("failed to load region from IMDS"));
        assert!(logs_contain("IMDS is disabled"));
    }
}
