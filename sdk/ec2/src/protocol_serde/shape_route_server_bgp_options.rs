// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_route_server_bgp_options(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::RouteServerBgpOptions, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::RouteServerBgpOptions::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("peerAsn") /* PeerAsn com.amazonaws.ec2#RouteServerBgpOptions$PeerAsn */ =>  {
                let var_1 =
                    Some(
                         {
                            <i64 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (long: `com.amazonaws.ec2#Long`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_peer_asn(var_1);
            }
            ,
            s if s.matches("peerLivenessDetection") /* PeerLivenessDetection com.amazonaws.ec2#RouteServerBgpOptions$PeerLivenessDetection */ =>  {
                let var_2 =
                    Some(
                        Result::<crate::types::RouteServerPeerLivenessMode, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::RouteServerPeerLivenessMode::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_peer_liveness_detection(var_2);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}
