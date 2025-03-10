// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_export_task(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::ExportTask, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::ExportTask::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("ExportTaskIdentifier") /* ExportTaskIdentifier com.amazonaws.rds#ExportTask$ExportTaskIdentifier */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_export_task_identifier(var_1);
            }
            ,
            s if s.matches("SourceArn") /* SourceArn com.amazonaws.rds#ExportTask$SourceArn */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_source_arn(var_2);
            }
            ,
            s if s.matches("ExportOnly") /* ExportOnly com.amazonaws.rds#ExportTask$ExportOnly */ =>  {
                let var_3 =
                    Some(
                        crate::protocol_serde::shape_string_list::de_string_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_export_only(var_3);
            }
            ,
            s if s.matches("SnapshotTime") /* SnapshotTime com.amazonaws.rds#ExportTask$SnapshotTime */ =>  {
                let var_4 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.rds#TStamp`)"))
                        ?
                    )
                ;
                builder = builder.set_snapshot_time(var_4);
            }
            ,
            s if s.matches("TaskStartTime") /* TaskStartTime com.amazonaws.rds#ExportTask$TaskStartTime */ =>  {
                let var_5 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.rds#TStamp`)"))
                        ?
                    )
                ;
                builder = builder.set_task_start_time(var_5);
            }
            ,
            s if s.matches("TaskEndTime") /* TaskEndTime com.amazonaws.rds#ExportTask$TaskEndTime */ =>  {
                let var_6 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.rds#TStamp`)"))
                        ?
                    )
                ;
                builder = builder.set_task_end_time(var_6);
            }
            ,
            s if s.matches("S3Bucket") /* S3Bucket com.amazonaws.rds#ExportTask$S3Bucket */ =>  {
                let var_7 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_s3_bucket(var_7);
            }
            ,
            s if s.matches("S3Prefix") /* S3Prefix com.amazonaws.rds#ExportTask$S3Prefix */ =>  {
                let var_8 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_s3_prefix(var_8);
            }
            ,
            s if s.matches("IamRoleArn") /* IamRoleArn com.amazonaws.rds#ExportTask$IamRoleArn */ =>  {
                let var_9 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_iam_role_arn(var_9);
            }
            ,
            s if s.matches("KmsKeyId") /* KmsKeyId com.amazonaws.rds#ExportTask$KmsKeyId */ =>  {
                let var_10 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_kms_key_id(var_10);
            }
            ,
            s if s.matches("Status") /* Status com.amazonaws.rds#ExportTask$Status */ =>  {
                let var_11 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_status(var_11);
            }
            ,
            s if s.matches("PercentProgress") /* PercentProgress com.amazonaws.rds#ExportTask$PercentProgress */ =>  {
                let var_12 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.rds#Integer`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_percent_progress(var_12);
            }
            ,
            s if s.matches("TotalExtractedDataInGB") /* TotalExtractedDataInGB com.amazonaws.rds#ExportTask$TotalExtractedDataInGB */ =>  {
                let var_13 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.rds#Integer`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_total_extracted_data_in_gb(var_13);
            }
            ,
            s if s.matches("FailureCause") /* FailureCause com.amazonaws.rds#ExportTask$FailureCause */ =>  {
                let var_14 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_failure_cause(var_14);
            }
            ,
            s if s.matches("WarningMessage") /* WarningMessage com.amazonaws.rds#ExportTask$WarningMessage */ =>  {
                let var_15 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_warning_message(var_15);
            }
            ,
            s if s.matches("SourceType") /* SourceType com.amazonaws.rds#ExportTask$SourceType */ =>  {
                let var_16 =
                    Some(
                        Result::<crate::types::ExportSourceType, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::ExportSourceType::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_source_type(var_16);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}
