use axum::http::{request::Parts, HeaderMap};
use rmcp::{service::RequestContext, RoleServer};

pub(crate) fn valid_trace_hex(value: &str, len: usize) -> bool {
    value.len() == len && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn trace_edit_group_id(headers: &HeaderMap) -> Option<String> {
    let traceparent = headers.get("traceparent")?.to_str().ok()?.trim();
    let mut parts = traceparent.split('-');
    let version = parts.next()?;
    let trace_id = parts.next()?;
    let parent_id = parts.next()?;
    let flags = parts.next()?;

    if parts.next().is_some()
        || !valid_trace_hex(version, 2)
        || version.eq_ignore_ascii_case("ff")
        || !valid_trace_hex(trace_id, 32)
        || trace_id.bytes().all(|byte| byte == b'0')
        || !valid_trace_hex(parent_id, 16)
        || parent_id.bytes().all(|byte| byte == b'0')
        || !valid_trace_hex(flags, 2)
    {
        return None;
    }

    Some(format!("trace-{trace_id}"))
}

pub(crate) fn request_edit_group_id(parts: &Parts) -> Option<String> {
    trace_edit_group_id(&parts.headers)
}

pub(crate) fn openai_conversation_session(context: &RequestContext<RoleServer>) -> Option<&str> {
    context
        .meta
        .get("openai/session")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(crate) fn request_mcp_session_key(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get("mcp-session-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|session_id| {
            let bounded = session_id.chars().take(220).collect::<String>();
            format!("mcp-session:{bounded}")
        })
}

pub(crate) fn continuation_identity_keys_from(conversation_session: Option<&str>) -> Vec<String> {
    // Only the OpenAI conversation identity is narrow enough to bind a continuation
    // to one ChatGPT conversation. MCP connection IDs can be shared across chats,
    // so they must never be used as a continuation-routing fallback.
    openai_conversation_session_value(conversation_session)
        .into_iter()
        .collect()
}

pub(crate) fn openai_conversation_session_value(conversation_session: Option<&str>) -> Option<String> {
    conversation_session.map(|conversation_session| {
        let bounded = conversation_session.chars().take(220).collect::<String>();
        format!("openai-session:{bounded}")
    })
}

pub(crate) fn continuation_identity_keys(_parts: &Parts, context: &RequestContext<RoleServer>) -> Vec<String> {
    continuation_identity_keys_from(openai_conversation_session(context))
}

pub(crate) fn request_client_key(parts: &Parts) -> Option<String> {
    if let Some(session_key) = request_mcp_session_key(parts) {
        return Some(session_key);
    }
    request_edit_group_id(parts).map(|trace| format!("request:{trace}"))
}

pub(crate) fn request_resource_owner_key(
    parts: &Parts,
    context: &RequestContext<RoleServer>,
) -> Option<String> {
    if let Some(conversation_session) = openai_conversation_session(context) {
        let bounded = conversation_session.chars().take(220).collect::<String>();
        return Some(format!("openai-session:{bounded}"));
    }
    request_client_key(parts)
}

