// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_create_access_point_for_object_lambda_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::create_access_point_for_object_lambda::CreateAccessPointForObjectLambdaOutput,
    crate::operation::create_access_point_for_object_lambda::CreateAccessPointForObjectLambdaError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::create_access_point_for_object_lambda::CreateAccessPointForObjectLambdaError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    Err(crate::operation::create_access_point_for_object_lambda::CreateAccessPointForObjectLambdaError::generic(generic))
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_create_access_point_for_object_lambda_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::create_access_point_for_object_lambda::CreateAccessPointForObjectLambdaOutput,
    crate::operation::create_access_point_for_object_lambda::CreateAccessPointForObjectLambdaError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::create_access_point_for_object_lambda::builders::CreateAccessPointForObjectLambdaOutputBuilder::default();
        output = crate::protocol_serde::shape_create_access_point_for_object_lambda::de_create_access_point_for_object_lambda(_response_body, output)
            .map_err(crate::operation::create_access_point_for_object_lambda::CreateAccessPointForObjectLambdaError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_create_access_point_for_object_lambda_headers(
    input: &crate::operation::create_access_point_for_object_lambda::CreateAccessPointForObjectLambdaInput,
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

pub fn ser_create_access_point_for_object_lambda_op_input(
    input: &crate::operation::create_access_point_for_object_lambda::CreateAccessPointForObjectLambdaInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    {
        let mut writer = ::aws_smithy_xml::encode::XmlWriter::new(&mut out);
        #[allow(unused_mut)]
        let mut root = writer
            .start_el("CreateAccessPointForObjectLambdaRequest")
            .write_ns("http://awss3control.amazonaws.com/doc/2018-08-20/", None);
        crate::protocol_serde::shape_create_access_point_for_object_lambda_input::ser_create_access_point_for_object_lambda_input_input_input(
            input, root,
        )?
    }
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

#[allow(unused_mut)]
pub fn de_create_access_point_for_object_lambda(
    inp: &[u8],
    mut builder: crate::operation::create_access_point_for_object_lambda::builders::CreateAccessPointForObjectLambdaOutputBuilder,
) -> std::result::Result<
    crate::operation::create_access_point_for_object_lambda::builders::CreateAccessPointForObjectLambdaOutputBuilder,
    ::aws_smithy_xml::decode::XmlDecodeError,
> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !start_el.matches("CreateAccessPointForObjectLambdaResult") {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "encountered invalid XML root: expected CreateAccessPointForObjectLambdaResult but got {:?}. This is likely a bug in the SDK.",
            start_el
        )));
    }
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("Alias") /* Alias com.amazonaws.s3control.synthetic#CreateAccessPointForObjectLambdaOutput$Alias */ =>  {
                let var_3 =
                    Some(
                        crate::protocol_serde::shape_object_lambda_access_point_alias::de_object_lambda_access_point_alias(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_alias(var_3);
            }
            ,
            s if s.matches("ObjectLambdaAccessPointArn") /* ObjectLambdaAccessPointArn com.amazonaws.s3control.synthetic#CreateAccessPointForObjectLambdaOutput$ObjectLambdaAccessPointArn */ =>  {
                let var_4 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_object_lambda_access_point_arn(var_4);
            }
            ,
            _ => {}
        }
    }
    Ok(builder)
}
