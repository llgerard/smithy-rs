/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![deny(missing_docs)]

//! AWS Shared Config
//!
//! This module contains an shared configuration representation that is agnostic from a specific service.

use std::sync::Arc;

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_smithy_async::rt::sleep::AsyncSleep;
use aws_smithy_client::http_connector::HttpConnector;
use aws_smithy_types::retry::RetryConfig;
use aws_smithy_types::timeout::TimeoutConfig;

use crate::app_name::AppName;
use crate::endpoint::ResolveAwsEndpoint;
use crate::region::Region;

/// AWS Shared Configuration
#[derive(Debug, Clone)]
pub struct SdkConfig {
    app_name: Option<AppName>,
    credentials_provider: Option<SharedCredentialsProvider>,
    region: Option<Region>,
    endpoint_resolver: Option<Arc<dyn ResolveAwsEndpoint>>,
    endpoint_url: Option<String>,
    retry_config: Option<RetryConfig>,
    sleep_impl: Option<Arc<dyn AsyncSleep>>,
    timeout_config: Option<TimeoutConfig>,
    http_connector: Option<HttpConnector>,
}

/// Builder for AWS Shared Configuration
///
/// _Important:_ Using the `aws-config` crate to configure the SDK is preferred to invoking this
/// builder directly. Using this builder directly won't pull in any AWS recommended default
/// configuration values.
#[derive(Debug, Default)]
pub struct Builder {
    app_name: Option<AppName>,
    credentials_provider: Option<SharedCredentialsProvider>,
    region: Option<Region>,
    endpoint_resolver: Option<Arc<dyn ResolveAwsEndpoint>>,
    endpoint_url: Option<String>,
    retry_config: Option<RetryConfig>,
    sleep_impl: Option<Arc<dyn AsyncSleep>>,
    timeout_config: Option<TimeoutConfig>,
    http_connector: Option<HttpConnector>,
}

impl Builder {
    /// Set the region for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::SdkConfig;
    /// use aws_types::region::Region;
    /// let config = SdkConfig::builder().region(Region::new("us-east-1")).build();
    /// ```
    pub fn region(mut self, region: impl Into<Option<Region>>) -> Self {
        self.set_region(region);
        self
    }

    /// Set the region for the builder
    ///
    /// # Examples
    /// ```rust
    /// fn region_override() -> Option<Region> {
    ///     // ...
    ///     # None
    /// }
    /// use aws_types::SdkConfig;
    /// use aws_types::region::Region;
    /// let mut builder = SdkConfig::builder();
    /// if let Some(region) = region_override() {
    ///     builder.set_region(region);
    /// }
    /// let config = builder.build();
    /// ```
    pub fn set_region(&mut self, region: impl Into<Option<Region>>) -> &mut Self {
        self.region = region.into();
        self
    }

    /// Set the endpoint resolver to use when making requests
    ///
    /// This method is deprecated. Use [`Self::endpoint_url`] instead.
    ///
    /// # Examples
    /// ```
    /// # fn wrapper() -> Result<(), aws_smithy_http::endpoint::error::InvalidEndpointError> {
    /// use std::sync::Arc;
    /// use aws_types::SdkConfig;
    /// use aws_smithy_http::endpoint::Endpoint;
    /// let config = SdkConfig::builder().endpoint_resolver(
    ///     Endpoint::immutable("http://localhost:8080")?
    /// ).build();
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(note = "use `endpoint_url` instead")]
    pub fn endpoint_resolver(
        mut self,
        endpoint_resolver: impl ResolveAwsEndpoint + 'static,
    ) -> Self {
        self.set_endpoint_resolver(Some(Arc::new(endpoint_resolver)));
        self
    }

    /// Set the endpoint url to use when making requests.
    /// # Examples
    /// ```
    /// use aws_types::SdkConfig;
    /// let config = SdkConfig::builder().endpoint_url("http://localhost:8080").build();
    /// ```
    pub fn endpoint_url(mut self, endpoint_url: impl Into<String>) -> Self {
        self.set_endpoint_url(Some(endpoint_url.into()));
        self
    }

    /// Set the endpoint url to use when making requests.
    pub fn set_endpoint_url(&mut self, endpoint_url: Option<String>) -> &mut Self {
        self.endpoint_url = endpoint_url;
        self
    }

    /// Set the endpoint resolver to use when making requests
    ///
    /// # Examples
    /// ```
    /// use std::sync::Arc;
    /// use aws_types::SdkConfig;
    /// use aws_types::endpoint::ResolveAwsEndpoint;
    /// fn endpoint_resolver_override() -> Option<Arc<dyn ResolveAwsEndpoint>> {
    ///     // ...
    ///     # None
    /// }
    /// let mut config = SdkConfig::builder();
    /// config.set_endpoint_resolver(endpoint_resolver_override());
    /// config.build();
    /// ```
    pub fn set_endpoint_resolver(
        &mut self,
        endpoint_resolver: Option<Arc<dyn ResolveAwsEndpoint>>,
    ) -> &mut Self {
        self.endpoint_resolver = endpoint_resolver;
        self
    }

    /// Set the retry_config for the builder
    ///
    /// _Note:_ Retries require a sleep implementation in order to work. When enabling retry, make
    /// sure to set one with [Self::sleep_impl] or [Self::set_sleep_impl].
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::SdkConfig;
    /// use aws_smithy_types::retry::RetryConfig;
    ///
    /// let retry_config = RetryConfig::standard().with_max_attempts(5);
    /// let config = SdkConfig::builder().retry_config(retry_config).build();
    /// ```
    pub fn retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.set_retry_config(Some(retry_config));
        self
    }

    /// Set the retry_config for the builder
    ///
    /// _Note:_ Retries require a sleep implementation in order to work. When enabling retry, make
    /// sure to set one with [Self::sleep_impl] or [Self::set_sleep_impl].
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::sdk_config::{SdkConfig, Builder};
    /// use aws_smithy_types::retry::RetryConfig;
    ///
    /// fn disable_retries(builder: &mut Builder) {
    ///     let retry_config = RetryConfig::standard().with_max_attempts(1);
    ///     builder.set_retry_config(Some(retry_config));
    /// }
    ///
    /// let mut builder = SdkConfig::builder();
    /// disable_retries(&mut builder);
    /// let config = builder.build();
    /// ```
    pub fn set_retry_config(&mut self, retry_config: Option<RetryConfig>) -> &mut Self {
        self.retry_config = retry_config;
        self
    }

    /// Set the [`TimeoutConfig`] for the builder
    ///
    /// _Note:_ Timeouts require a sleep implementation in order to work.
    /// When enabling timeouts, be sure to set one with [Self::sleep_impl] or
    /// [Self::set_sleep_impl].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::time::Duration;
    /// use aws_types::SdkConfig;
    /// use aws_smithy_types::timeout::TimeoutConfig;
    ///
    /// let timeout_config = TimeoutConfig::builder()
    ///     .operation_attempt_timeout(Duration::from_secs(2))
    ///     .operation_timeout(Duration::from_secs(5))
    ///     .build();
    /// let config = SdkConfig::builder()
    ///     .timeout_config(timeout_config)
    ///     .build();
    /// ```
    pub fn timeout_config(mut self, timeout_config: TimeoutConfig) -> Self {
        self.set_timeout_config(Some(timeout_config));
        self
    }

    /// Set the [`TimeoutConfig`] for the builder
    ///
    /// _Note:_ Timeouts require a sleep implementation in order to work.
    /// When enabling timeouts, be sure to set one with [Self::sleep_impl] or
    /// [Self::set_sleep_impl].
    ///
    /// # Examples
    /// ```rust
    /// # use std::time::Duration;
    /// use aws_types::sdk_config::{SdkConfig, Builder};
    /// use aws_smithy_types::timeout::TimeoutConfig;
    ///
    /// fn set_preferred_timeouts(builder: &mut Builder) {
    ///     let timeout_config = TimeoutConfig::builder()
    ///         .operation_attempt_timeout(Duration::from_secs(2))
    ///         .operation_timeout(Duration::from_secs(5))
    ///         .build();
    ///     builder.set_timeout_config(Some(timeout_config));
    /// }
    ///
    /// let mut builder = SdkConfig::builder();
    /// set_preferred_timeouts(&mut builder);
    /// let config = builder.build();
    /// ```
    pub fn set_timeout_config(&mut self, timeout_config: Option<TimeoutConfig>) -> &mut Self {
        self.timeout_config = timeout_config;
        self
    }

    /// Set the sleep implementation for the builder. The sleep implementation is used to create
    /// timeout futures.
    ///
    /// _Note:_ If you're using the Tokio runtime, a `TokioSleep` implementation is available in
    /// the `aws-smithy-async` crate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use aws_smithy_async::rt::sleep::{AsyncSleep, Sleep};
    /// use aws_types::SdkConfig;
    ///
    /// ##[derive(Debug)]
    /// pub struct ForeverSleep;
    ///
    /// impl AsyncSleep for ForeverSleep {
    ///     fn sleep(&self, duration: std::time::Duration) -> Sleep {
    ///         Sleep::new(std::future::pending())
    ///     }
    /// }
    ///
    /// let sleep_impl = Arc::new(ForeverSleep);
    /// let config = SdkConfig::builder().sleep_impl(sleep_impl).build();
    /// ```
    pub fn sleep_impl(mut self, sleep_impl: Arc<dyn AsyncSleep>) -> Self {
        self.set_sleep_impl(Some(sleep_impl));
        self
    }

    /// Set the sleep implementation for the builder. The sleep implementation is used to create
    /// timeout futures.
    ///
    /// _Note:_ If you're using the Tokio runtime, a `TokioSleep` implementation is available in
    /// the `aws-smithy-async` crate.
    ///
    /// # Examples
    /// ```rust
    /// # use aws_smithy_async::rt::sleep::{AsyncSleep, Sleep};
    /// # use aws_types::sdk_config::{Builder, SdkConfig};
    /// #[derive(Debug)]
    /// pub struct ForeverSleep;
    ///
    /// impl AsyncSleep for ForeverSleep {
    ///     fn sleep(&self, duration: std::time::Duration) -> Sleep {
    ///         Sleep::new(std::future::pending())
    ///     }
    /// }
    ///
    /// fn set_never_ending_sleep_impl(builder: &mut Builder) {
    ///     let sleep_impl = std::sync::Arc::new(ForeverSleep);
    ///     builder.set_sleep_impl(Some(sleep_impl));
    /// }
    ///
    /// let mut builder = SdkConfig::builder();
    /// set_never_ending_sleep_impl(&mut builder);
    /// let config = builder.build();
    /// ```
    pub fn set_sleep_impl(&mut self, sleep_impl: Option<Arc<dyn AsyncSleep>>) -> &mut Self {
        self.sleep_impl = sleep_impl;
        self
    }

    /// Set the credentials provider for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_credential_types::provider::{ProvideCredentials, SharedCredentialsProvider};
    /// use aws_types::SdkConfig;
    /// fn make_provider() -> impl ProvideCredentials {
    ///   // ...
    ///   # use aws_credential_types::Credentials;
    ///   # Credentials::new("test", "test", None, None, "example")
    /// }
    ///
    /// let config = SdkConfig::builder()
    ///     .credentials_provider(SharedCredentialsProvider::new(make_provider()))
    ///     .build();
    /// ```
    pub fn credentials_provider(mut self, provider: SharedCredentialsProvider) -> Self {
        self.set_credentials_provider(Some(provider));
        self
    }

    /// Set the credentials provider for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_credential_types::provider::{ProvideCredentials, SharedCredentialsProvider};
    /// use aws_types::SdkConfig;
    /// fn make_provider() -> impl ProvideCredentials {
    ///   // ...
    ///   # use aws_credential_types::Credentials;
    ///   # Credentials::new("test", "test", None, None, "example")
    /// }
    ///
    /// fn override_provider() -> bool {
    ///   // ...
    ///   # true
    /// }
    ///
    /// let mut builder = SdkConfig::builder();
    /// if override_provider() {
    ///     builder.set_credentials_provider(Some(SharedCredentialsProvider::new(make_provider())));
    /// }
    /// let config = builder.build();
    /// ```
    pub fn set_credentials_provider(
        &mut self,
        provider: Option<SharedCredentialsProvider>,
    ) -> &mut Self {
        self.credentials_provider = provider;
        self
    }

    /// Sets the name of the app that is using the client.
    ///
    /// This _optional_ name is used to identify the application in the user agent that
    /// gets sent along with requests.
    pub fn app_name(mut self, app_name: AppName) -> Self {
        self.set_app_name(Some(app_name));
        self
    }

    /// Sets the name of the app that is using the client.
    ///
    /// This _optional_ name is used to identify the application in the user agent that
    /// gets sent along with requests.
    pub fn set_app_name(&mut self, app_name: Option<AppName>) -> &mut Self {
        self.app_name = app_name;
        self
    }

    /// Sets the HTTP connector to use when making requests.
    ///
    /// ## Examples
    /// ```no_run
    /// # #[cfg(feature = "examples")]
    /// # fn example() {
    /// use std::time::Duration;
    /// use aws_smithy_client::{Client, hyper_ext};
    /// use aws_smithy_client::erase::DynConnector;
    /// use aws_smithy_client::http_connector::ConnectorSettings;
    /// use aws_types::SdkConfig;
    ///
    /// let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
    ///     .with_webpki_roots()
    ///     .https_only()
    ///     .enable_http1()
    ///     .enable_http2()
    ///     .build();
    /// let smithy_connector = hyper_ext::Adapter::builder()
    ///     // Optionally set things like timeouts as well
    ///     .connector_settings(
    ///         ConnectorSettings::builder()
    ///             .connect_timeout(Duration::from_secs(5))
    ///             .build()
    ///     )
    ///     .build(https_connector);
    /// let sdk_config = SdkConfig::builder()
    ///     .http_connector(smithy_connector)
    ///     .build();
    /// # }
    /// ```
    pub fn http_connector(mut self, http_connector: impl Into<HttpConnector>) -> Self {
        self.set_http_connector(Some(http_connector));
        self
    }

    /// Sets the HTTP connector to use when making requests.
    ///
    /// ## Examples
    /// ```no_run
    /// # #[cfg(feature = "examples")]
    /// # fn example() {
    /// use std::time::Duration;
    /// use aws_smithy_client::hyper_ext;
    /// use aws_smithy_client::http_connector::ConnectorSettings;
    /// use aws_types::sdk_config::{SdkConfig, Builder};
    ///
    /// fn override_http_connector(builder: &mut Builder) {
    ///     let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
    ///         .with_webpki_roots()
    ///         .https_only()
    ///         .enable_http1()
    ///         .enable_http2()
    ///         .build();
    ///     let smithy_connector = hyper_ext::Adapter::builder()
    ///         // Optionally set things like timeouts as well
    ///         .connector_settings(
    ///             ConnectorSettings::builder()
    ///                 .connect_timeout(Duration::from_secs(5))
    ///                 .build()
    ///         )
    ///         .build(https_connector);
    ///     builder.set_http_connector(Some(smithy_connector));
    /// }
    ///
    /// let mut builder = SdkConfig::builder();
    /// override_http_connector(&mut builder);
    /// let config = builder.build();
    /// # }
    /// ```
    pub fn set_http_connector(
        &mut self,
        http_connector: Option<impl Into<HttpConnector>>,
    ) -> &mut Self {
        self.http_connector = http_connector.map(|inner| inner.into());
        self
    }

    /// Build a [`SdkConfig`](SdkConfig) from this builder
    pub fn build(self) -> SdkConfig {
        SdkConfig {
            app_name: self.app_name,
            credentials_provider: self.credentials_provider,
            region: self.region,
            endpoint_resolver: self.endpoint_resolver,
            endpoint_url: self.endpoint_url,
            retry_config: self.retry_config,
            sleep_impl: self.sleep_impl,
            timeout_config: self.timeout_config,
            http_connector: self.http_connector,
        }
    }
}

impl SdkConfig {
    /// Configured region
    pub fn region(&self) -> Option<&Region> {
        self.region.as_ref()
    }

    /// Configured endpoint resolver
    pub fn endpoint_resolver(&self) -> Option<Arc<dyn ResolveAwsEndpoint>> {
        self.endpoint_resolver.clone()
    }

    /// Configured endpoint URL
    pub fn endpoint_url(&self) -> Option<&str> {
        self.endpoint_url.as_deref()
    }

    /// Configured retry config
    pub fn retry_config(&self) -> Option<&RetryConfig> {
        self.retry_config.as_ref()
    }

    /// Configured timeout config
    pub fn timeout_config(&self) -> Option<&TimeoutConfig> {
        self.timeout_config.as_ref()
    }

    #[doc(hidden)]
    /// Configured sleep implementation
    pub fn sleep_impl(&self) -> Option<Arc<dyn AsyncSleep>> {
        self.sleep_impl.clone()
    }

    /// Configured credentials provider
    pub fn credentials_provider(&self) -> Option<&SharedCredentialsProvider> {
        self.credentials_provider.as_ref()
    }

    /// Configured app name
    pub fn app_name(&self) -> Option<&AppName> {
        self.app_name.as_ref()
    }

    /// Configured HTTP Connector
    pub fn http_connector(&self) -> Option<&HttpConnector> {
        self.http_connector.as_ref()
    }

    /// Config builder
    ///
    /// _Important:_ Using the `aws-config` crate to configure the SDK is preferred to invoking this
    /// builder directly. Using this builder directly won't pull in any AWS recommended default
    /// configuration values.
    pub fn builder() -> Builder {
        Builder::default()
    }
}
