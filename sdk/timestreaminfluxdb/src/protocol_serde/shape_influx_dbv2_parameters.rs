// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_influx_dbv2_parameters<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::InfluxDBv2Parameters>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::InfluxDBv2ParametersBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "fluxLogEnabled" => {
                            builder = builder.set_flux_log_enabled(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "logLevel" => {
                            builder = builder.set_log_level(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::LogLevel::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "noTasks" => {
                            builder = builder.set_no_tasks(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "queryConcurrency" => {
                            builder = builder.set_query_concurrency(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "queryQueueSize" => {
                            builder = builder.set_query_queue_size(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "tracingType" => {
                            builder = builder.set_tracing_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::TracingType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "metricsDisabled" => {
                            builder = builder.set_metrics_disabled(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "httpIdleTimeout" => {
                            builder = builder.set_http_idle_timeout(crate::protocol_serde::shape_duration::de_duration(tokens)?);
                        }
                        "httpReadHeaderTimeout" => {
                            builder = builder.set_http_read_header_timeout(crate::protocol_serde::shape_duration::de_duration(tokens)?);
                        }
                        "httpReadTimeout" => {
                            builder = builder.set_http_read_timeout(crate::protocol_serde::shape_duration::de_duration(tokens)?);
                        }
                        "httpWriteTimeout" => {
                            builder = builder.set_http_write_timeout(crate::protocol_serde::shape_duration::de_duration(tokens)?);
                        }
                        "influxqlMaxSelectBuckets" => {
                            builder = builder.set_influxql_max_select_buckets(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "influxqlMaxSelectPoint" => {
                            builder = builder.set_influxql_max_select_point(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "influxqlMaxSelectSeries" => {
                            builder = builder.set_influxql_max_select_series(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "pprofDisabled" => {
                            builder = builder.set_pprof_disabled(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "queryInitialMemoryBytes" => {
                            builder = builder.set_query_initial_memory_bytes(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "queryMaxMemoryBytes" => {
                            builder = builder.set_query_max_memory_bytes(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "queryMemoryBytes" => {
                            builder = builder.set_query_memory_bytes(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "sessionLength" => {
                            builder = builder.set_session_length(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "sessionRenewDisabled" => {
                            builder = builder.set_session_renew_disabled(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "storageCacheMaxMemorySize" => {
                            builder = builder.set_storage_cache_max_memory_size(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "storageCacheSnapshotMemorySize" => {
                            builder = builder.set_storage_cache_snapshot_memory_size(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "storageCacheSnapshotWriteColdDuration" => {
                            builder =
                                builder.set_storage_cache_snapshot_write_cold_duration(crate::protocol_serde::shape_duration::de_duration(tokens)?);
                        }
                        "storageCompactFullWriteColdDuration" => {
                            builder =
                                builder.set_storage_compact_full_write_cold_duration(crate::protocol_serde::shape_duration::de_duration(tokens)?);
                        }
                        "storageCompactThroughputBurst" => {
                            builder = builder.set_storage_compact_throughput_burst(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "storageMaxConcurrentCompactions" => {
                            builder = builder.set_storage_max_concurrent_compactions(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "storageMaxIndexLogFileSize" => {
                            builder = builder.set_storage_max_index_log_file_size(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "storageNoValidateFieldSize" => {
                            builder = builder
                                .set_storage_no_validate_field_size(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "storageRetentionCheckInterval" => {
                            builder = builder.set_storage_retention_check_interval(crate::protocol_serde::shape_duration::de_duration(tokens)?);
                        }
                        "storageSeriesFileMaxConcurrentSnapshotCompactions" => {
                            builder = builder.set_storage_series_file_max_concurrent_snapshot_compactions(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "storageSeriesIdSetCacheSize" => {
                            builder = builder.set_storage_series_id_set_cache_size(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "storageWalMaxConcurrentWrites" => {
                            builder = builder.set_storage_wal_max_concurrent_writes(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "storageWalMaxWriteDelay" => {
                            builder = builder.set_storage_wal_max_write_delay(crate::protocol_serde::shape_duration::de_duration(tokens)?);
                        }
                        "uiDisabled" => {
                            builder = builder.set_ui_disabled(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        _ => ::aws_smithy_json::deserialize::token::skip_value(tokens)?,
                    },
                    other => {
                        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(format!(
                            "expected object key or end object, found: {:?}",
                            other
                        )))
                    }
                }
            }
            Ok(Some(builder.build()))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

pub fn ser_influx_dbv2_parameters(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::InfluxDBv2Parameters,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.flux_log_enabled {
        object.key("fluxLogEnabled").boolean(*var_1);
    }
    if let Some(var_2) = &input.log_level {
        object.key("logLevel").string(var_2.as_str());
    }
    if let Some(var_3) = &input.no_tasks {
        object.key("noTasks").boolean(*var_3);
    }
    if let Some(var_4) = &input.query_concurrency {
        object.key("queryConcurrency").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_4).into()),
        );
    }
    if let Some(var_5) = &input.query_queue_size {
        object.key("queryQueueSize").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_5).into()),
        );
    }
    if let Some(var_6) = &input.tracing_type {
        object.key("tracingType").string(var_6.as_str());
    }
    if let Some(var_7) = &input.metrics_disabled {
        object.key("metricsDisabled").boolean(*var_7);
    }
    if let Some(var_8) = &input.http_idle_timeout {
        #[allow(unused_mut)]
        let mut object_9 = object.key("httpIdleTimeout").start_object();
        crate::protocol_serde::shape_duration::ser_duration(&mut object_9, var_8)?;
        object_9.finish();
    }
    if let Some(var_10) = &input.http_read_header_timeout {
        #[allow(unused_mut)]
        let mut object_11 = object.key("httpReadHeaderTimeout").start_object();
        crate::protocol_serde::shape_duration::ser_duration(&mut object_11, var_10)?;
        object_11.finish();
    }
    if let Some(var_12) = &input.http_read_timeout {
        #[allow(unused_mut)]
        let mut object_13 = object.key("httpReadTimeout").start_object();
        crate::protocol_serde::shape_duration::ser_duration(&mut object_13, var_12)?;
        object_13.finish();
    }
    if let Some(var_14) = &input.http_write_timeout {
        #[allow(unused_mut)]
        let mut object_15 = object.key("httpWriteTimeout").start_object();
        crate::protocol_serde::shape_duration::ser_duration(&mut object_15, var_14)?;
        object_15.finish();
    }
    if let Some(var_16) = &input.influxql_max_select_buckets {
        object.key("influxqlMaxSelectBuckets").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_16).into()),
        );
    }
    if let Some(var_17) = &input.influxql_max_select_point {
        object.key("influxqlMaxSelectPoint").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_17).into()),
        );
    }
    if let Some(var_18) = &input.influxql_max_select_series {
        object.key("influxqlMaxSelectSeries").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_18).into()),
        );
    }
    if let Some(var_19) = &input.pprof_disabled {
        object.key("pprofDisabled").boolean(*var_19);
    }
    if let Some(var_20) = &input.query_initial_memory_bytes {
        object.key("queryInitialMemoryBytes").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_20).into()),
        );
    }
    if let Some(var_21) = &input.query_max_memory_bytes {
        object.key("queryMaxMemoryBytes").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_21).into()),
        );
    }
    if let Some(var_22) = &input.query_memory_bytes {
        object.key("queryMemoryBytes").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_22).into()),
        );
    }
    if let Some(var_23) = &input.session_length {
        object.key("sessionLength").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_23).into()),
        );
    }
    if let Some(var_24) = &input.session_renew_disabled {
        object.key("sessionRenewDisabled").boolean(*var_24);
    }
    if let Some(var_25) = &input.storage_cache_max_memory_size {
        object.key("storageCacheMaxMemorySize").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_25).into()),
        );
    }
    if let Some(var_26) = &input.storage_cache_snapshot_memory_size {
        object.key("storageCacheSnapshotMemorySize").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_26).into()),
        );
    }
    if let Some(var_27) = &input.storage_cache_snapshot_write_cold_duration {
        #[allow(unused_mut)]
        let mut object_28 = object.key("storageCacheSnapshotWriteColdDuration").start_object();
        crate::protocol_serde::shape_duration::ser_duration(&mut object_28, var_27)?;
        object_28.finish();
    }
    if let Some(var_29) = &input.storage_compact_full_write_cold_duration {
        #[allow(unused_mut)]
        let mut object_30 = object.key("storageCompactFullWriteColdDuration").start_object();
        crate::protocol_serde::shape_duration::ser_duration(&mut object_30, var_29)?;
        object_30.finish();
    }
    if let Some(var_31) = &input.storage_compact_throughput_burst {
        object.key("storageCompactThroughputBurst").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_31).into()),
        );
    }
    if let Some(var_32) = &input.storage_max_concurrent_compactions {
        object.key("storageMaxConcurrentCompactions").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_32).into()),
        );
    }
    if let Some(var_33) = &input.storage_max_index_log_file_size {
        object.key("storageMaxIndexLogFileSize").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_33).into()),
        );
    }
    if let Some(var_34) = &input.storage_no_validate_field_size {
        object.key("storageNoValidateFieldSize").boolean(*var_34);
    }
    if let Some(var_35) = &input.storage_retention_check_interval {
        #[allow(unused_mut)]
        let mut object_36 = object.key("storageRetentionCheckInterval").start_object();
        crate::protocol_serde::shape_duration::ser_duration(&mut object_36, var_35)?;
        object_36.finish();
    }
    if let Some(var_37) = &input.storage_series_file_max_concurrent_snapshot_compactions {
        object.key("storageSeriesFileMaxConcurrentSnapshotCompactions").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_37).into()),
        );
    }
    if let Some(var_38) = &input.storage_series_id_set_cache_size {
        object.key("storageSeriesIdSetCacheSize").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_38).into()),
        );
    }
    if let Some(var_39) = &input.storage_wal_max_concurrent_writes {
        object.key("storageWalMaxConcurrentWrites").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_39).into()),
        );
    }
    if let Some(var_40) = &input.storage_wal_max_write_delay {
        #[allow(unused_mut)]
        let mut object_41 = object.key("storageWalMaxWriteDelay").start_object();
        crate::protocol_serde::shape_duration::ser_duration(&mut object_41, var_40)?;
        object_41.finish();
    }
    if let Some(var_42) = &input.ui_disabled {
        object.key("uiDisabled").boolean(*var_42);
    }
    Ok(())
}
