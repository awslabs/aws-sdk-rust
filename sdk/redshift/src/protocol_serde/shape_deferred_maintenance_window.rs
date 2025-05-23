// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_deferred_maintenance_window(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::DeferredMaintenanceWindow, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::DeferredMaintenanceWindow::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("DeferMaintenanceIdentifier") /* DeferMaintenanceIdentifier com.amazonaws.redshift#DeferredMaintenanceWindow$DeferMaintenanceIdentifier */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_defer_maintenance_identifier(var_1);
            }
            ,
            s if s.matches("DeferMaintenanceStartTime") /* DeferMaintenanceStartTime com.amazonaws.redshift#DeferredMaintenanceWindow$DeferMaintenanceStartTime */ =>  {
                let var_2 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.redshift#TStamp`)"))
                        ?
                    )
                ;
                builder = builder.set_defer_maintenance_start_time(var_2);
            }
            ,
            s if s.matches("DeferMaintenanceEndTime") /* DeferMaintenanceEndTime com.amazonaws.redshift#DeferredMaintenanceWindow$DeferMaintenanceEndTime */ =>  {
                let var_3 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.redshift#TStamp`)"))
                        ?
                    )
                ;
                builder = builder.set_defer_maintenance_end_time(var_3);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}
