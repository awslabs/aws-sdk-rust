// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_create_access_grants_location_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::create_access_grants_location::CreateAccessGrantsLocationOutput,
    crate::operation::create_access_grants_location::CreateAccessGrantsLocationError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::create_access_grants_location::CreateAccessGrantsLocationError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    Err(crate::operation::create_access_grants_location::CreateAccessGrantsLocationError::generic(
        generic,
    ))
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_create_access_grants_location_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::create_access_grants_location::CreateAccessGrantsLocationOutput,
    crate::operation::create_access_grants_location::CreateAccessGrantsLocationError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::create_access_grants_location::builders::CreateAccessGrantsLocationOutputBuilder::default();
        output = crate::protocol_serde::shape_create_access_grants_location::de_create_access_grants_location(_response_body, output)
            .map_err(crate::operation::create_access_grants_location::CreateAccessGrantsLocationError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_create_access_grants_location_headers(
    input: &crate::operation::create_access_grants_location::CreateAccessGrantsLocationInput,
    mut builder: ::http::request::Builder,
) -> std::result::Result<::http::request::Builder, ::aws_smithy_types::error::operation::BuildError> {
    if let ::std::option::Option::Some(inner_1) = &input.account_id {
        let formatted_2 = inner_1.as_str();
        let header_value = formatted_2;
        let header_value: ::http::HeaderValue = header_value.parse().map_err(|err| {
            ::aws_smithy_types::error::operation::BuildError::invalid_field(
                "account_id",
                format!("`{}` cannot be used as a header value: {}", &header_value, err),
            )
        })?;
        builder = builder.header("x-amz-account-id", header_value);
    }
    Ok(builder)
}

pub fn ser_create_access_grants_location_op_input(
    input: &crate::operation::create_access_grants_location::CreateAccessGrantsLocationInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    {
        let mut writer = ::aws_smithy_xml::encode::XmlWriter::new(&mut out);
        #[allow(unused_mut)]
        let mut root = writer
            .start_el("CreateAccessGrantsLocationRequest")
            .write_ns("http://awss3control.amazonaws.com/doc/2018-08-20/", None);
        crate::protocol_serde::shape_create_access_grants_location_input::ser_create_access_grants_location_input_input_input(input, root)?
    }
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

#[allow(unused_mut)]
pub fn de_create_access_grants_location(
    inp: &[u8],
    mut builder: crate::operation::create_access_grants_location::builders::CreateAccessGrantsLocationOutputBuilder,
) -> std::result::Result<
    crate::operation::create_access_grants_location::builders::CreateAccessGrantsLocationOutputBuilder,
    ::aws_smithy_xml::decode::XmlDecodeError,
> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !start_el.matches("CreateAccessGrantsLocationResult") {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "encountered invalid XML root: expected CreateAccessGrantsLocationResult but got {:?}. This is likely a bug in the SDK.",
            start_el
        )));
    }
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("CreatedAt") /* CreatedAt com.amazonaws.s3control.synthetic#CreateAccessGrantsLocationOutput$CreatedAt */ =>  {
                let var_3 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.s3control#CreationTimestamp`)"))
                        ?
                    )
                ;
                builder = builder.set_created_at(var_3);
            }
            ,
            s if s.matches("LocationScope") /* LocationScope com.amazonaws.s3control.synthetic#CreateAccessGrantsLocationOutput$LocationScope */ =>  {
                let var_4 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_location_scope(var_4);
            }
            ,
            s if s.matches("IAMRoleArn") /* IAMRoleArn com.amazonaws.s3control.synthetic#CreateAccessGrantsLocationOutput$IAMRoleArn */ =>  {
                let var_5 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_iam_role_arn(var_5);
            }
            ,
            s if s.matches("AccessGrantsLocationArn") /* AccessGrantsLocationArn com.amazonaws.s3control.synthetic#CreateAccessGrantsLocationOutput$AccessGrantsLocationArn */ =>  {
                let var_6 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_access_grants_location_arn(var_6);
            }
            ,
            s if s.matches("AccessGrantsLocationId") /* AccessGrantsLocationId com.amazonaws.s3control.synthetic#CreateAccessGrantsLocationOutput$AccessGrantsLocationId */ =>  {
                let var_7 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_access_grants_location_id(var_7);
            }
            ,
            _ => {}
        }
    }
    Ok(builder)
}
