// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_get_merge_conflicts_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::get_merge_conflicts::GetMergeConflictsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.repository_name {
        object.key("repositoryName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.destination_commit_specifier {
        object.key("destinationCommitSpecifier").string(var_2.as_str());
    }
    if let Some(var_3) = &input.source_commit_specifier {
        object.key("sourceCommitSpecifier").string(var_3.as_str());
    }
    if let Some(var_4) = &input.merge_option {
        object.key("mergeOption").string(var_4.as_str());
    }
    if let Some(var_5) = &input.conflict_detail_level {
        object.key("conflictDetailLevel").string(var_5.as_str());
    }
    if let Some(var_6) = &input.max_conflict_files {
        object.key("maxConflictFiles").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_6).into()),
        );
    }
    if let Some(var_7) = &input.conflict_resolution_strategy {
        object.key("conflictResolutionStrategy").string(var_7.as_str());
    }
    if let Some(var_8) = &input.next_token {
        object.key("nextToken").string(var_8.as_str());
    }
    Ok(())
}
