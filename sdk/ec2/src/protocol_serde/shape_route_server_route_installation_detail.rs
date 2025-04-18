// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_route_server_route_installation_detail(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::RouteServerRouteInstallationDetail, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::RouteServerRouteInstallationDetail::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("routeTableId") /* RouteTableId com.amazonaws.ec2#RouteServerRouteInstallationDetail$RouteTableId */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_route_table_id(var_1);
            }
            ,
            s if s.matches("routeInstallationStatus") /* RouteInstallationStatus com.amazonaws.ec2#RouteServerRouteInstallationDetail$RouteInstallationStatus */ =>  {
                let var_2 =
                    Some(
                        Result::<crate::types::RouteServerRouteInstallationStatus, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::RouteServerRouteInstallationStatus::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_route_installation_status(var_2);
            }
            ,
            s if s.matches("routeInstallationStatusReason") /* RouteInstallationStatusReason com.amazonaws.ec2#RouteServerRouteInstallationDetail$RouteInstallationStatusReason */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_route_installation_status_reason(var_3);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}
