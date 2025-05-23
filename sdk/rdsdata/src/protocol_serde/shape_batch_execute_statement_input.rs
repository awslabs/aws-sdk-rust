// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_batch_execute_statement_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::batch_execute_statement::BatchExecuteStatementInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.database {
        object.key("database").string(var_1.as_str());
    }
    if let Some(var_2) = &input.parameter_sets {
        let mut array_3 = object.key("parameterSets").start_array();
        for item_4 in var_2 {
            {
                let mut array_5 = array_3.value().start_array();
                for item_6 in item_4 {
                    {
                        #[allow(unused_mut)]
                        let mut object_7 = array_5.value().start_object();
                        crate::protocol_serde::shape_sql_parameter::ser_sql_parameter(&mut object_7, item_6)?;
                        object_7.finish();
                    }
                }
                array_5.finish();
            }
        }
        array_3.finish();
    }
    if let Some(var_8) = &input.resource_arn {
        object.key("resourceArn").string(var_8.as_str());
    }
    if let Some(var_9) = &input.schema {
        object.key("schema").string(var_9.as_str());
    }
    if let Some(var_10) = &input.secret_arn {
        object.key("secretArn").string(var_10.as_str());
    }
    if let Some(var_11) = &input.sql {
        object.key("sql").string(var_11.as_str());
    }
    if let Some(var_12) = &input.transaction_id {
        object.key("transactionId").string(var_12.as_str());
    }
    Ok(())
}
