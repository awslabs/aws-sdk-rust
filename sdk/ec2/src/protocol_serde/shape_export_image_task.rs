// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_export_image_task(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::ExportImageTask, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::ExportImageTask::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("description") /* Description com.amazonaws.ec2#ExportImageTask$Description */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_description(var_1);
            }
            ,
            s if s.matches("exportImageTaskId") /* ExportImageTaskId com.amazonaws.ec2#ExportImageTask$ExportImageTaskId */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_export_image_task_id(var_2);
            }
            ,
            s if s.matches("imageId") /* ImageId com.amazonaws.ec2#ExportImageTask$ImageId */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_image_id(var_3);
            }
            ,
            s if s.matches("progress") /* Progress com.amazonaws.ec2#ExportImageTask$Progress */ =>  {
                let var_4 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_progress(var_4);
            }
            ,
            s if s.matches("s3ExportLocation") /* S3ExportLocation com.amazonaws.ec2#ExportImageTask$S3ExportLocation */ =>  {
                let var_5 =
                    Some(
                        crate::protocol_serde::shape_export_task_s3_location::de_export_task_s3_location(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_s3_export_location(var_5);
            }
            ,
            s if s.matches("status") /* Status com.amazonaws.ec2#ExportImageTask$Status */ =>  {
                let var_6 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_status(var_6);
            }
            ,
            s if s.matches("statusMessage") /* StatusMessage com.amazonaws.ec2#ExportImageTask$StatusMessage */ =>  {
                let var_7 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_status_message(var_7);
            }
            ,
            s if s.matches("tagSet") /* Tags com.amazonaws.ec2#ExportImageTask$Tags */ =>  {
                let var_8 =
                    Some(
                        crate::protocol_serde::shape_tag_list::de_tag_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_tags(var_8);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}
