// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_intent_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_intent::CreateIntentInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.description {
        object.key("description").string(var_1.as_str());
    }
    if let Some(var_2) = &input.dialog_code_hook {
        #[allow(unused_mut)]
        let mut object_3 = object.key("dialogCodeHook").start_object();
        crate::protocol_serde::shape_dialog_code_hook_settings::ser_dialog_code_hook_settings(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.fulfillment_code_hook {
        #[allow(unused_mut)]
        let mut object_5 = object.key("fulfillmentCodeHook").start_object();
        crate::protocol_serde::shape_fulfillment_code_hook_settings::ser_fulfillment_code_hook_settings(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.initial_response_setting {
        #[allow(unused_mut)]
        let mut object_7 = object.key("initialResponseSetting").start_object();
        crate::protocol_serde::shape_initial_response_setting::ser_initial_response_setting(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.input_contexts {
        let mut array_9 = object.key("inputContexts").start_array();
        for item_10 in var_8 {
            {
                #[allow(unused_mut)]
                let mut object_11 = array_9.value().start_object();
                crate::protocol_serde::shape_input_context::ser_input_context(&mut object_11, item_10)?;
                object_11.finish();
            }
        }
        array_9.finish();
    }
    if let Some(var_12) = &input.intent_closing_setting {
        #[allow(unused_mut)]
        let mut object_13 = object.key("intentClosingSetting").start_object();
        crate::protocol_serde::shape_intent_closing_setting::ser_intent_closing_setting(&mut object_13, var_12)?;
        object_13.finish();
    }
    if let Some(var_14) = &input.intent_confirmation_setting {
        #[allow(unused_mut)]
        let mut object_15 = object.key("intentConfirmationSetting").start_object();
        crate::protocol_serde::shape_intent_confirmation_setting::ser_intent_confirmation_setting(&mut object_15, var_14)?;
        object_15.finish();
    }
    if let Some(var_16) = &input.intent_name {
        object.key("intentName").string(var_16.as_str());
    }
    if let Some(var_17) = &input.kendra_configuration {
        #[allow(unused_mut)]
        let mut object_18 = object.key("kendraConfiguration").start_object();
        crate::protocol_serde::shape_kendra_configuration::ser_kendra_configuration(&mut object_18, var_17)?;
        object_18.finish();
    }
    if let Some(var_19) = &input.output_contexts {
        let mut array_20 = object.key("outputContexts").start_array();
        for item_21 in var_19 {
            {
                #[allow(unused_mut)]
                let mut object_22 = array_20.value().start_object();
                crate::protocol_serde::shape_output_context::ser_output_context(&mut object_22, item_21)?;
                object_22.finish();
            }
        }
        array_20.finish();
    }
    if let Some(var_23) = &input.parent_intent_signature {
        object.key("parentIntentSignature").string(var_23.as_str());
    }
    if let Some(var_24) = &input.q_in_connect_intent_configuration {
        #[allow(unused_mut)]
        let mut object_25 = object.key("qInConnectIntentConfiguration").start_object();
        crate::protocol_serde::shape_q_in_connect_intent_configuration::ser_q_in_connect_intent_configuration(&mut object_25, var_24)?;
        object_25.finish();
    }
    if let Some(var_26) = &input.qn_a_intent_configuration {
        #[allow(unused_mut)]
        let mut object_27 = object.key("qnAIntentConfiguration").start_object();
        crate::protocol_serde::shape_qn_a_intent_configuration::ser_qn_a_intent_configuration(&mut object_27, var_26)?;
        object_27.finish();
    }
    if let Some(var_28) = &input.sample_utterances {
        let mut array_29 = object.key("sampleUtterances").start_array();
        for item_30 in var_28 {
            {
                #[allow(unused_mut)]
                let mut object_31 = array_29.value().start_object();
                crate::protocol_serde::shape_sample_utterance::ser_sample_utterance(&mut object_31, item_30)?;
                object_31.finish();
            }
        }
        array_29.finish();
    }
    Ok(())
}
