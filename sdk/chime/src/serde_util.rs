// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn account_correct_errors(mut builder: crate::types::builders::AccountBuilder) -> crate::types::builders::AccountBuilder {
    if builder.aws_account_id.is_none() {
        builder.aws_account_id = Some(Default::default())
    }
    if builder.account_id.is_none() {
        builder.account_id = Some(Default::default())
    }
    if builder.name.is_none() {
        builder.name = Some(Default::default())
    }
    builder
}

pub(crate) fn user_correct_errors(mut builder: crate::types::builders::UserBuilder) -> crate::types::builders::UserBuilder {
    if builder.user_id.is_none() {
        builder.user_id = Some(Default::default())
    }
    builder
}

pub(crate) fn user_settings_correct_errors(mut builder: crate::types::builders::UserSettingsBuilder) -> crate::types::builders::UserSettingsBuilder {
    if builder.telephony.is_none() {
        builder.telephony = {
            let builder = crate::types::builders::TelephonySettingsBuilder::default();
            crate::serde_util::telephony_settings_correct_errors(builder).build().ok()
        }
    }
    builder
}

pub(crate) fn telephony_settings_correct_errors(
    mut builder: crate::types::builders::TelephonySettingsBuilder,
) -> crate::types::builders::TelephonySettingsBuilder {
    if builder.inbound_calling.is_none() {
        builder.inbound_calling = Some(Default::default())
    }
    if builder.outbound_calling.is_none() {
        builder.outbound_calling = Some(Default::default())
    }
    if builder.sms.is_none() {
        builder.sms = Some(Default::default())
    }
    builder
}
