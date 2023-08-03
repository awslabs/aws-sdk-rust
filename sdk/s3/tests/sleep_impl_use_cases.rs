/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod with_sdk_config {
    use aws_config::retry::RetryConfig;
    use aws_config::timeout::TimeoutConfig;
    use aws_config::SdkConfig;
    use aws_sdk_s3 as s3;
    use aws_smithy_async::rt::sleep::SharedAsyncSleep;
    use std::time::Duration;

    #[tokio::test]
    async fn using_config_loader() {
        // When using `aws_config::load_from_env`, things should just work
        let config = aws_config::load_from_env().await;
        assert!(config.timeout_config().unwrap().has_timeouts());
        assert!(config.retry_config().unwrap().has_retry());
        let _s3 = s3::Client::new(&config);
    }

    #[test]
    fn manual_config_construction_all_defaults() {
        // When manually constructing `SdkConfig` with everything unset,
        // it should work since there will be no timeouts or retries enabled,
        // and thus, no sleep impl is required.
        let config = SdkConfig::builder().build();
        assert!(config.timeout_config().is_none());
        assert!(config.retry_config().is_none());
        let _s3 = s3::Client::new(&config);
    }

    #[test]
    fn no_sleep_no_timeouts_no_retries() {
        // When explicitly setting timeouts and retries to their disabled
        // states, it should work since no sleep impl is required.
        let config = SdkConfig::builder()
            .timeout_config(TimeoutConfig::disabled())
            .retry_config(RetryConfig::disabled())
            .build();
        assert!(!config.timeout_config().unwrap().has_timeouts());
        assert!(!config.retry_config().unwrap().has_retry());
        let _s3 = s3::Client::new(&config);
    }

    #[test]
    #[should_panic]
    fn no_sleep_no_timeouts_yes_retries() {
        // When retries are enabled and a sleep impl isn't given, it should panic
        let config = SdkConfig::builder()
            .timeout_config(TimeoutConfig::disabled())
            .retry_config(RetryConfig::standard())
            .build();
        assert!(!config.timeout_config().unwrap().has_timeouts());
        assert!(config.retry_config().unwrap().has_retry());
        let _s3 = s3::Client::new(&config);
    }

    #[test]
    #[should_panic]
    fn no_sleep_yes_timeouts_no_retries() {
        // When timeouts are enabled and a sleep impl isn't given, it should panic
        let config = SdkConfig::builder()
            .timeout_config(
                TimeoutConfig::builder()
                    .operation_timeout(Duration::from_millis(100))
                    .build(),
            )
            .retry_config(RetryConfig::disabled())
            .build();
        assert!(config.timeout_config().unwrap().has_timeouts());
        assert!(!config.retry_config().unwrap().has_retry());
        let _s3 = s3::Client::new(&config);
    }

    #[test]
    #[should_panic]
    fn no_sleep_yes_timeouts_yes_retries() {
        // When timeouts and retries are enabled but a sleep impl isn't given, it should panic
        let config = SdkConfig::builder()
            .timeout_config(
                TimeoutConfig::builder()
                    .operation_timeout(Duration::from_millis(100))
                    .build(),
            )
            .retry_config(RetryConfig::standard().with_max_attempts(2))
            .build();
        assert!(config.timeout_config().unwrap().has_timeouts());
        assert!(config.retry_config().unwrap().has_retry());
        let _s3 = s3::Client::new(&config);
    }

    #[test]
    fn yes_sleep_yes_timeouts_yes_retries() {
        // When a sleep impl is given, enabling timeouts/retries should work
        let config = SdkConfig::builder()
            .timeout_config(
                TimeoutConfig::builder()
                    .operation_timeout(Duration::from_millis(100))
                    .build(),
            )
            .retry_config(RetryConfig::standard().with_max_attempts(2))
            .sleep_impl(SharedAsyncSleep::new(
                aws_smithy_async::rt::sleep::TokioSleep::new(),
            ))
            .build();
        assert!(config.timeout_config().unwrap().has_timeouts());
        assert!(config.retry_config().unwrap().has_retry());
        let _s3 = s3::Client::new(&config);
    }
}

mod with_service_config {
    use aws_config::retry::RetryConfig;
    use aws_config::timeout::TimeoutConfig;
    use aws_config::SdkConfig;
    use aws_sdk_s3 as s3;
    use aws_smithy_async::rt::sleep::SharedAsyncSleep;
    use std::time::Duration;

    #[test]
    fn manual_config_construction_all_defaults() {
        // When manually constructing `Config` with everything unset,
        // it should work since there will be no timeouts or retries enabled,
        // and thus, no sleep impl is required.
        let config = s3::Config::builder().build();
        let _s3 = s3::Client::from_conf(config);
    }

    #[test]
    fn no_sleep_no_timeouts_no_retries() {
        // When explicitly setting timeouts and retries to their disabled
        // states, it should work since no sleep impl is required.
        let config = s3::Config::builder()
            .timeout_config(TimeoutConfig::disabled())
            .retry_config(RetryConfig::disabled())
            .build();
        let _s3 = s3::Client::from_conf(config);
    }

    #[test]
    #[should_panic]
    fn no_sleep_no_timeouts_yes_retries() {
        // When retries are enabled and a sleep impl isn't given, it should panic
        let config = s3::Config::builder()
            .timeout_config(TimeoutConfig::disabled())
            .retry_config(RetryConfig::standard())
            .build();
        let _s3 = s3::Client::from_conf(config);
    }

    #[test]
    #[should_panic]
    fn no_sleep_yes_timeouts_no_retries() {
        // When timeouts are enabled and a sleep impl isn't given, it should panic
        let config = s3::Config::builder()
            .timeout_config(
                TimeoutConfig::builder()
                    .operation_timeout(Duration::from_millis(100))
                    .build(),
            )
            .retry_config(RetryConfig::disabled())
            .build();
        let _s3 = s3::Client::from_conf(config);
    }

    #[test]
    #[should_panic]
    fn no_sleep_yes_timeouts_yes_retries() {
        // When retries and timeouts are enabled and a sleep impl isn't given, it should panic
        let config = s3::Config::builder()
            .timeout_config(
                TimeoutConfig::builder()
                    .operation_timeout(Duration::from_millis(100))
                    .build(),
            )
            .retry_config(RetryConfig::standard().with_max_attempts(2))
            .build();
        let _s3 = s3::Client::from_conf(config);
    }

    #[test]
    fn yes_sleep_yes_timeouts_yes_retries() {
        // When a sleep impl is given, enabling timeouts/retries should work
        let config = SdkConfig::builder()
            .timeout_config(
                TimeoutConfig::builder()
                    .operation_timeout(Duration::from_millis(100))
                    .build(),
            )
            .retry_config(RetryConfig::standard().with_max_attempts(2))
            .sleep_impl(SharedAsyncSleep::new(
                aws_smithy_async::rt::sleep::TokioSleep::new(),
            ))
            .build();
        let _s3 = s3::Client::new(&config);
    }
}
