/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod with_sdk_config {
    use aws_config::SdkConfig;
    use aws_sdk_s3 as s3;
    use s3::config::StalledStreamProtectionConfig;

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
        let config = SdkConfig::builder()
            .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
            .build();
        assert!(config.timeout_config().is_none());
        assert!(config.retry_config().is_none());
        let _s3 = s3::Client::new(&config);
    }

    #[test]
    fn bytestream_from_path_exists() {
        let _ = aws_sdk_s3::primitives::ByteStream::from_path("a/b.txt");
    }
}

mod with_service_config {
    use aws_sdk_s3 as s3;

    #[test]
    fn manual_config_construction_all_defaults() {
        // When manually constructing `Config` with everything unset,
        // it should work since there will be no timeouts or retries enabled,
        // and thus, no sleep impl is required.
        let config = s3::Config::builder().build();
        let _s3 = s3::Client::from_conf(config);
    }
}
