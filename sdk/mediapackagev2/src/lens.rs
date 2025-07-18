// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn reflens_list_channel_groups_output_output_next_token(
    input: &crate::operation::list_channel_groups::ListChannelGroupsOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_list_channels_output_output_next_token(
    input: &crate::operation::list_channels::ListChannelsOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_list_harvest_jobs_output_output_next_token(
    input: &crate::operation::list_harvest_jobs::ListHarvestJobsOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_list_origin_endpoints_output_output_next_token(
    input: &crate::operation::list_origin_endpoints::ListOriginEndpointsOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_list_channel_groups_output_output_items(
    input: crate::operation::list_channel_groups::ListChannelGroupsOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::ChannelGroupListConfiguration>> {
    let input = input.items?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_list_channels_output_output_items(
    input: crate::operation::list_channels::ListChannelsOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::ChannelListConfiguration>> {
    let input = input.items?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_list_harvest_jobs_output_output_items(
    input: crate::operation::list_harvest_jobs::ListHarvestJobsOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::HarvestJob>> {
    let input = input.items?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_list_origin_endpoints_output_output_items(
    input: crate::operation::list_origin_endpoints::ListOriginEndpointsOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::OriginEndpointListConfiguration>> {
    let input = input.items?;
    ::std::option::Option::Some(input)
}
