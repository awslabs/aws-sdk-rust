// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub use crate::operation::create_simulation_application::_create_simulation_application_output::CreateSimulationApplicationOutputBuilder;

pub use crate::operation::create_simulation_application::_create_simulation_application_input::CreateSimulationApplicationInputBuilder;

impl crate::operation::create_simulation_application::builders::CreateSimulationApplicationInputBuilder {
    /// Sends a request with this input using the given client.
    pub async fn send_with(
        self,
        client: &crate::Client,
    ) -> ::std::result::Result<
        crate::operation::create_simulation_application::CreateSimulationApplicationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_simulation_application::CreateSimulationApplicationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let mut fluent_builder = client.create_simulation_application();
        fluent_builder.inner = self;
        fluent_builder.send().await
    }
}
/// Fluent builder constructing a request to `CreateSimulationApplication`.
///
/// <important>
/// <p>End of support notice: On September 10, 2025, Amazon Web Services will discontinue support for Amazon Web Services RoboMaker. After September 10, 2025, you will no longer be able to access the Amazon Web Services RoboMaker console or Amazon Web Services RoboMaker resources. For more information on transitioning to Batch to help run containerized simulations, visit <a href="https://aws.amazon.com/blogs/hpc/run-simulations-using-multiple-containers-in-a-single-aws-batch-job/">https://aws.amazon.com/blogs/hpc/run-simulations-using-multiple-containers-in-a-single-aws-batch-job/</a>.</p>
/// </important>
/// <p>Creates a simulation application.</p>
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct CreateSimulationApplicationFluentBuilder {
    handle: ::std::sync::Arc<crate::client::Handle>,
    inner: crate::operation::create_simulation_application::builders::CreateSimulationApplicationInputBuilder,
    config_override: ::std::option::Option<crate::config::Builder>,
}
impl
    crate::client::customize::internal::CustomizableSend<
        crate::operation::create_simulation_application::CreateSimulationApplicationOutput,
        crate::operation::create_simulation_application::CreateSimulationApplicationError,
    > for CreateSimulationApplicationFluentBuilder
{
    fn send(
        self,
        config_override: crate::config::Builder,
    ) -> crate::client::customize::internal::BoxFuture<
        crate::client::customize::internal::SendResult<
            crate::operation::create_simulation_application::CreateSimulationApplicationOutput,
            crate::operation::create_simulation_application::CreateSimulationApplicationError,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.config_override(config_override).send().await })
    }
}
impl CreateSimulationApplicationFluentBuilder {
    /// Creates a new `CreateSimulationApplicationFluentBuilder`.
    pub(crate) fn new(handle: ::std::sync::Arc<crate::client::Handle>) -> Self {
        Self {
            handle,
            inner: ::std::default::Default::default(),
            config_override: ::std::option::Option::None,
        }
    }
    /// Access the CreateSimulationApplication as a reference.
    pub fn as_input(&self) -> &crate::operation::create_simulation_application::builders::CreateSimulationApplicationInputBuilder {
        &self.inner
    }
    /// Sends the request and returns the response.
    ///
    /// If an error occurs, an `SdkError` will be returned with additional details that
    /// can be matched against.
    ///
    /// By default, any retryable failures will be retried twice. Retry behavior
    /// is configurable with the [RetryConfig](aws_smithy_types::retry::RetryConfig), which can be
    /// set when configuring the client.
    pub async fn send(
        self,
    ) -> ::std::result::Result<
        crate::operation::create_simulation_application::CreateSimulationApplicationOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::create_simulation_application::CreateSimulationApplicationError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = self
            .inner
            .build()
            .map_err(::aws_smithy_runtime_api::client::result::SdkError::construction_failure)?;
        let runtime_plugins = crate::operation::create_simulation_application::CreateSimulationApplication::operation_runtime_plugins(
            self.handle.runtime_plugins.clone(),
            &self.handle.conf,
            self.config_override,
        );
        crate::operation::create_simulation_application::CreateSimulationApplication::orchestrate(&runtime_plugins, input).await
    }

    /// Consumes this builder, creating a customizable operation that can be modified before being sent.
    pub fn customize(
        self,
    ) -> crate::client::customize::CustomizableOperation<
        crate::operation::create_simulation_application::CreateSimulationApplicationOutput,
        crate::operation::create_simulation_application::CreateSimulationApplicationError,
        Self,
    > {
        crate::client::customize::CustomizableOperation::new(self)
    }
    pub(crate) fn config_override(mut self, config_override: impl ::std::convert::Into<crate::config::Builder>) -> Self {
        self.set_config_override(::std::option::Option::Some(config_override.into()));
        self
    }

    pub(crate) fn set_config_override(&mut self, config_override: ::std::option::Option<crate::config::Builder>) -> &mut Self {
        self.config_override = config_override;
        self
    }
    /// <p>The name of the simulation application.</p>
    pub fn name(mut self, input: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.name(input.into());
        self
    }
    /// <p>The name of the simulation application.</p>
    pub fn set_name(mut self, input: ::std::option::Option<::std::string::String>) -> Self {
        self.inner = self.inner.set_name(input);
        self
    }
    /// <p>The name of the simulation application.</p>
    pub fn get_name(&self) -> &::std::option::Option<::std::string::String> {
        self.inner.get_name()
    }
    ///
    /// Appends an item to `sources`.
    ///
    /// To override the contents of this collection use [`set_sources`](Self::set_sources).
    ///
    /// <p>The sources of the simulation application.</p>
    pub fn sources(mut self, input: crate::types::SourceConfig) -> Self {
        self.inner = self.inner.sources(input);
        self
    }
    /// <p>The sources of the simulation application.</p>
    pub fn set_sources(mut self, input: ::std::option::Option<::std::vec::Vec<crate::types::SourceConfig>>) -> Self {
        self.inner = self.inner.set_sources(input);
        self
    }
    /// <p>The sources of the simulation application.</p>
    pub fn get_sources(&self) -> &::std::option::Option<::std::vec::Vec<crate::types::SourceConfig>> {
        self.inner.get_sources()
    }
    /// <p>The simulation software suite used by the simulation application.</p>
    pub fn simulation_software_suite(mut self, input: crate::types::SimulationSoftwareSuite) -> Self {
        self.inner = self.inner.simulation_software_suite(input);
        self
    }
    /// <p>The simulation software suite used by the simulation application.</p>
    pub fn set_simulation_software_suite(mut self, input: ::std::option::Option<crate::types::SimulationSoftwareSuite>) -> Self {
        self.inner = self.inner.set_simulation_software_suite(input);
        self
    }
    /// <p>The simulation software suite used by the simulation application.</p>
    pub fn get_simulation_software_suite(&self) -> &::std::option::Option<crate::types::SimulationSoftwareSuite> {
        self.inner.get_simulation_software_suite()
    }
    /// <p>The robot software suite used by the simulation application.</p>
    pub fn robot_software_suite(mut self, input: crate::types::RobotSoftwareSuite) -> Self {
        self.inner = self.inner.robot_software_suite(input);
        self
    }
    /// <p>The robot software suite used by the simulation application.</p>
    pub fn set_robot_software_suite(mut self, input: ::std::option::Option<crate::types::RobotSoftwareSuite>) -> Self {
        self.inner = self.inner.set_robot_software_suite(input);
        self
    }
    /// <p>The robot software suite used by the simulation application.</p>
    pub fn get_robot_software_suite(&self) -> &::std::option::Option<crate::types::RobotSoftwareSuite> {
        self.inner.get_robot_software_suite()
    }
    /// <p>The rendering engine for the simulation application.</p>
    pub fn rendering_engine(mut self, input: crate::types::RenderingEngine) -> Self {
        self.inner = self.inner.rendering_engine(input);
        self
    }
    /// <p>The rendering engine for the simulation application.</p>
    pub fn set_rendering_engine(mut self, input: ::std::option::Option<crate::types::RenderingEngine>) -> Self {
        self.inner = self.inner.set_rendering_engine(input);
        self
    }
    /// <p>The rendering engine for the simulation application.</p>
    pub fn get_rendering_engine(&self) -> &::std::option::Option<crate::types::RenderingEngine> {
        self.inner.get_rendering_engine()
    }
    ///
    /// Adds a key-value pair to `tags`.
    ///
    /// To override the contents of this collection use [`set_tags`](Self::set_tags).
    ///
    /// <p>A map that contains tag keys and tag values that are attached to the simulation application.</p>
    pub fn tags(mut self, k: impl ::std::convert::Into<::std::string::String>, v: impl ::std::convert::Into<::std::string::String>) -> Self {
        self.inner = self.inner.tags(k.into(), v.into());
        self
    }
    /// <p>A map that contains tag keys and tag values that are attached to the simulation application.</p>
    pub fn set_tags(mut self, input: ::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>>) -> Self {
        self.inner = self.inner.set_tags(input);
        self
    }
    /// <p>A map that contains tag keys and tag values that are attached to the simulation application.</p>
    pub fn get_tags(&self) -> &::std::option::Option<::std::collections::HashMap<::std::string::String, ::std::string::String>> {
        self.inner.get_tags()
    }
    /// <p>The object that contains the Docker image URI used to create your simulation application.</p>
    pub fn environment(mut self, input: crate::types::Environment) -> Self {
        self.inner = self.inner.environment(input);
        self
    }
    /// <p>The object that contains the Docker image URI used to create your simulation application.</p>
    pub fn set_environment(mut self, input: ::std::option::Option<crate::types::Environment>) -> Self {
        self.inner = self.inner.set_environment(input);
        self
    }
    /// <p>The object that contains the Docker image URI used to create your simulation application.</p>
    pub fn get_environment(&self) -> &::std::option::Option<crate::types::Environment> {
        self.inner.get_environment()
    }
}
