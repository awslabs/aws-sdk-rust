// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_put_bucket_policy_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::put_bucket_policy::PutBucketPolicyOutput, crate::operation::put_bucket_policy::PutBucketPolicyError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::put_bucket_policy::PutBucketPolicyError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    Err(crate::operation::put_bucket_policy::PutBucketPolicyError::generic(generic))
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_put_bucket_policy_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::put_bucket_policy::PutBucketPolicyOutput, crate::operation::put_bucket_policy::PutBucketPolicyError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::put_bucket_policy::builders::PutBucketPolicyOutputBuilder::default();
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_put_bucket_policy_headers(
    input: &crate::operation::put_bucket_policy::PutBucketPolicyInput,
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
    if let ::std::option::Option::Some(inner_3) = &input.confirm_remove_self_bucket_access {
        let mut encoder = ::aws_smithy_types::primitive::Encoder::from(*inner_3);
        let formatted_4 = encoder.encode();
        let header_value = formatted_4;
        let header_value: ::http::HeaderValue = header_value.parse().map_err(|err| {
            ::aws_smithy_types::error::operation::BuildError::invalid_field(
                "confirm_remove_self_bucket_access",
                format!("`{}` cannot be used as a header value: {}", &header_value, err),
            )
        })?;
        builder = builder.header("x-amz-confirm-remove-self-bucket-access", header_value);
    }
    Ok(builder)
}

pub fn ser_put_bucket_policy_op_input(
    input: &crate::operation::put_bucket_policy::PutBucketPolicyInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    {
        let mut writer = ::aws_smithy_xml::encode::XmlWriter::new(&mut out);
        #[allow(unused_mut)]
        let mut root = writer
            .start_el("PutBucketPolicyRequest")
            .write_ns("http://awss3control.amazonaws.com/doc/2018-08-20/", None);
        crate::protocol_serde::shape_put_bucket_policy_input::ser_put_bucket_policy_input_input_input(input, root)?
    }
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}
