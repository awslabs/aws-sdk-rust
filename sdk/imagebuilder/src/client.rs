// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[derive(Debug)]
pub(crate) struct Handle {
    pub(crate) conf: crate::Config,
    #[allow(dead_code)] // unused when a service does not provide any operations
    pub(crate) runtime_plugins: ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
}

/// Client for EC2 Image Builder
///
/// Client for invoking operations on EC2 Image Builder. Each operation on EC2 Image Builder is a method on this
/// this struct. `.send()` MUST be invoked on the generated operations to dispatch the request to the service.
/// ## Constructing a `Client`
///
/// A [`Config`] is required to construct a client. For most use cases, the [`aws-config`]
/// crate should be used to automatically resolve this config using
/// [`aws_config::load_from_env()`], since this will resolve an [`SdkConfig`] which can be shared
/// across multiple different AWS SDK clients. This config resolution process can be customized
/// by calling [`aws_config::from_env()`] instead, which returns a [`ConfigLoader`] that uses
/// the [builder pattern] to customize the default config.
///
/// In the simplest case, creating a client looks as follows:
/// ```rust,no_run
/// # async fn wrapper() {
/// let config = aws_config::load_from_env().await;
/// let client = aws_sdk_imagebuilder::Client::new(&config);
/// # }
/// ```
///
/// Occasionally, SDKs may have additional service-specific values that can be set on the [`Config`] that
/// is absent from [`SdkConfig`], or slightly different settings for a specific client may be desired.
/// The [`Builder`](crate::config::Builder) struct implements `From<&SdkConfig>`, so setting these specific settings can be
/// done as follows:
///
/// ```rust,no_run
/// # async fn wrapper() {
/// let sdk_config = ::aws_config::load_from_env().await;
/// let config = aws_sdk_imagebuilder::config::Builder::from(&sdk_config)
/// # /*
///     .some_service_specific_setting("value")
/// # */
///     .build();
/// # }
/// ```
///
/// See the [`aws-config` docs] and [`Config`] for more information on customizing configuration.
///
/// _Note:_ Client construction is expensive due to connection thread pool initialization, and should
/// be done once at application start-up.
///
/// [`Config`]: crate::Config
/// [`ConfigLoader`]: https://docs.rs/aws-config/*/aws_config/struct.ConfigLoader.html
/// [`SdkConfig`]: https://docs.rs/aws-config/*/aws_config/struct.SdkConfig.html
/// [`aws-config` docs]: https://docs.rs/aws-config/*
/// [`aws-config`]: https://crates.io/crates/aws-config
/// [`aws_config::from_env()`]: https://docs.rs/aws-config/*/aws_config/fn.from_env.html
/// [`aws_config::load_from_env()`]: https://docs.rs/aws-config/*/aws_config/fn.load_from_env.html
/// [builder pattern]: https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder
/// # Using the `Client`
///
/// A client has a function for every operation that can be performed by the service.
/// For example, the [`CancelImageCreation`](crate::operation::cancel_image_creation) operation has
/// a [`Client::cancel_image_creation`], function which returns a builder for that operation.
/// The fluent builder ultimately has a `send()` function that returns an async future that
/// returns a result, as illustrated below:
///
/// ```rust,ignore
/// let result = client.cancel_image_creation()
///     .image_build_version_arn("example")
///     .send()
///     .await;
/// ```
///
/// The underlying HTTP requests that get made by this can be modified with the `customize_operation`
/// function on the fluent builder. See the [`customize`](crate::client::customize) module for more
/// information.
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct Client {
    handle: ::std::sync::Arc<Handle>,
}

impl Client {
    /// Creates a new client from the service [`Config`](crate::Config).
    ///
    /// # Panics
    ///
    /// This method will panic in the following cases:
    ///
    /// - Retries or timeouts are enabled without a `sleep_impl` configured.
    /// - Identity caching is enabled without a `sleep_impl` and `time_source` configured.
    /// - No `behavior_version` is provided.
    ///
    /// The panic message for each of these will have instructions on how to resolve them.
    #[track_caller]
    pub fn from_conf(conf: crate::Config) -> Self {
        let handle = Handle {
            conf: conf.clone(),
            runtime_plugins: crate::config::base_client_runtime_plugins(conf),
        };
        if let Err(err) = Self::validate_config(&handle) {
            panic!("Invalid client configuration: {err}");
        }
        Self {
            handle: ::std::sync::Arc::new(handle),
        }
    }

    /// Returns the client's configuration.
    pub fn config(&self) -> &crate::Config {
        &self.handle.conf
    }

    fn validate_config(handle: &Handle) -> ::std::result::Result<(), ::aws_smithy_runtime_api::box_error::BoxError> {
        let mut cfg = ::aws_smithy_types::config_bag::ConfigBag::base();
        handle
            .runtime_plugins
            .apply_client_configuration(&mut cfg)?
            .validate_base_client_config(&cfg)?;
        Ok(())
    }
}

impl Client {
    /// Creates a new client from an [SDK Config](::aws_types::sdk_config::SdkConfig).
    ///
    /// # Panics
    ///
    /// - This method will panic if the `sdk_config` is missing an async sleep implementation. If you experience this panic, set
    ///   the `sleep_impl` on the Config passed into this function to fix it.
    /// - This method will panic if the `sdk_config` is missing an HTTP connector. If you experience this panic, set the
    ///   `http_connector` on the Config passed into this function to fix it.
    /// - This method will panic if no `BehaviorVersion` is provided. If you experience this panic, set `behavior_version` on the Config or enable the `behavior-version-latest` Cargo feature.
    #[track_caller]
    pub fn new(sdk_config: &::aws_types::sdk_config::SdkConfig) -> Self {
        Self::from_conf(sdk_config.into())
    }
}

mod cancel_image_creation;

mod cancel_lifecycle_execution;

mod create_component;

mod create_container_recipe;

mod create_distribution_configuration;

mod create_image;

mod create_image_pipeline;

mod create_image_recipe;

mod create_infrastructure_configuration;

mod create_lifecycle_policy;

mod create_workflow;

/// Operation customization and supporting types.
///
/// The underlying HTTP requests made during an operation can be customized
/// by calling the `customize()` method on the builder returned from a client
/// operation call. For example, this can be used to add an additional HTTP header:
///
/// ```ignore
/// # async fn wrapper() -> ::std::result::Result<(), aws_sdk_imagebuilder::Error> {
/// # let client: aws_sdk_imagebuilder::Client = unimplemented!();
/// use ::http::header::{HeaderName, HeaderValue};
///
/// let result = client.cancel_image_creation()
///     .customize()
///     .mutate_request(|req| {
///         // Add `x-example-header` with value
///         req.headers_mut()
///             .insert(
///                 HeaderName::from_static("x-example-header"),
///                 HeaderValue::from_static("1"),
///             );
///     })
///     .send()
///     .await;
/// # }
/// ```
pub mod customize;

mod delete_component;

mod delete_container_recipe;

mod delete_distribution_configuration;

mod delete_image;

mod delete_image_pipeline;

mod delete_image_recipe;

mod delete_infrastructure_configuration;

mod delete_lifecycle_policy;

mod delete_workflow;

mod get_component;

mod get_component_policy;

mod get_container_recipe;

mod get_container_recipe_policy;

mod get_distribution_configuration;

mod get_image;

mod get_image_pipeline;

mod get_image_policy;

mod get_image_recipe;

mod get_image_recipe_policy;

mod get_infrastructure_configuration;

mod get_lifecycle_execution;

mod get_lifecycle_policy;

mod get_marketplace_resource;

mod get_workflow;

mod get_workflow_execution;

mod get_workflow_step_execution;

mod import_component;

mod import_disk_image;

mod import_vm_image;

mod list_component_build_versions;

mod list_components;

mod list_container_recipes;

mod list_distribution_configurations;

mod list_image_build_versions;

mod list_image_packages;

mod list_image_pipeline_images;

mod list_image_pipelines;

mod list_image_recipes;

mod list_image_scan_finding_aggregations;

mod list_image_scan_findings;

mod list_images;

mod list_infrastructure_configurations;

mod list_lifecycle_execution_resources;

mod list_lifecycle_executions;

mod list_lifecycle_policies;

mod list_tags_for_resource;

mod list_waiting_workflow_steps;

mod list_workflow_build_versions;

mod list_workflow_executions;

mod list_workflow_step_executions;

mod list_workflows;

mod put_component_policy;

mod put_container_recipe_policy;

mod put_image_policy;

mod put_image_recipe_policy;

mod send_workflow_step_action;

mod start_image_pipeline_execution;

mod start_resource_state_update;

mod tag_resource;

mod untag_resource;

mod update_distribution_configuration;

mod update_image_pipeline;

mod update_infrastructure_configuration;

mod update_lifecycle_policy;
