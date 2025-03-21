// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn de_suspended_processes(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<::std::vec::Vec<crate::types::SuspendedProcess>, ::aws_smithy_xml::decode::XmlDecodeError> {
    let mut out = std::vec::Vec::new();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("member") /* member com.amazonaws.autoscaling#SuspendedProcesses$member */ =>  {
                out.push(
                    crate::protocol_serde::shape_suspended_process::de_suspended_process(&mut tag)
                    ?
                );
            }
            ,
            _ => {}
        }
    }
    Ok(out)
}
