// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_restore_volume_from_snapshot_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotOutput,
    crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => {
            return Err(crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotError::unhandled(
                generic,
            ))
        }
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "BadRequest" => crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotError::BadRequest({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::BadRequestBuilder::default();
                output = crate::protocol_serde::shape_bad_request::de_bad_request_json_err(_response_body, output)
                    .map_err(crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalServerError" => crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotError::InternalServerError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerErrorBuilder::default();
                output = crate::protocol_serde::shape_internal_server_error::de_internal_server_error_json_err(_response_body, output)
                    .map_err(crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "VolumeNotFound" => crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotError::VolumeNotFound({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::VolumeNotFoundBuilder::default();
                output = crate::protocol_serde::shape_volume_not_found::de_volume_not_found_json_err(_response_body, output)
                    .map_err(crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_restore_volume_from_snapshot_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotOutput,
    crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::restore_volume_from_snapshot::builders::RestoreVolumeFromSnapshotOutputBuilder::default();
        output = crate::protocol_serde::shape_restore_volume_from_snapshot::de_restore_volume_from_snapshot(_response_body, output)
            .map_err(crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_restore_volume_from_snapshot_input(
    input: &crate::operation::restore_volume_from_snapshot::RestoreVolumeFromSnapshotInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_restore_volume_from_snapshot_input::ser_restore_volume_from_snapshot_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_restore_volume_from_snapshot(
    value: &[u8],
    mut builder: crate::operation::restore_volume_from_snapshot::builders::RestoreVolumeFromSnapshotOutputBuilder,
) -> ::std::result::Result<
    crate::operation::restore_volume_from_snapshot::builders::RestoreVolumeFromSnapshotOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "VolumeId" => {
                    builder = builder.set_volume_id(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "Lifecycle" => {
                    builder = builder.set_lifecycle(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| crate::types::VolumeLifecycle::from(u.as_ref())))
                            .transpose()?,
                    );
                }
                "AdministrativeActions" => {
                    builder =
                        builder.set_administrative_actions(crate::protocol_serde::shape_administrative_actions::de_administrative_actions(tokens)?);
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
    if tokens.next().is_some() {
        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "found more JSON tokens after completing parsing",
        ));
    }
    Ok(builder)
}
