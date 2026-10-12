use tauri::AppHandle;
use std::collections::BTreeMap;
use crate::mcp_phone_helpers::parse_phone_key;
use crate::{app_state::AppState, phone, semantic, mcp_phone_params::PhoneTransactionStepParam};

pub(crate) fn valid_phone_transaction_alias(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

pub(crate) fn phone_transaction_query(step: &PhoneTransactionStepParam) -> semantic::SemanticFindQuery {
    semantic::SemanticFindQuery {
        query: step.query.clone(),
        role: step.role.clone(),
        name: step.name.clone(),
        state: step.state.clone(),
        action: step.semantic_action.clone(),
        limit: 3,
    }
}

pub(crate) fn phone_transaction_focused_editable(
    snapshot: &phone::PhoneSemanticSnapshotResult,
) -> Option<&semantic::SemanticNode> {
    snapshot.snapshot.nodes.iter().find(|node| {
        node.role == "textbox"
            && !node.sensitive
            && node
                .states
                .iter()
                .any(|state| state.eq_ignore_ascii_case("focused"))
            && node
                .states
                .iter()
                .any(|state| state.eq_ignore_ascii_case("editable"))
    })
}

pub(crate) fn phone_transaction_has_selector(step: &PhoneTransactionStepParam) -> bool {
    [
        step.query.as_deref(),
        step.role.as_deref(),
        step.name.as_deref(),
        step.state.as_deref(),
        step.semantic_action.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|value| !value.trim().is_empty())
}

#[derive(Debug, Clone)]
pub(crate) struct PhoneTransactionAliasTarget {
    pub(crate) snapshot_id: String,
    pub(crate) ref_id: String,
}

pub(crate) fn phone_transaction_find_unique(
    app: &AppHandle,
    state: &AppState,
    device_id: &str,
    step: &PhoneTransactionStepParam,
) -> Result<(phone::PhoneSemanticSnapshotResult, semantic::SemanticNode), String> {
    let snapshot = state.phone.semantic_snapshot(app, device_id, 800, None)?;
    let mut matches = state.phone.semantic_find(
        device_id,
        &snapshot.snapshot.snapshot_id,
        phone_transaction_query(step),
    )?;
    match matches.len() {
        0 => Err("TARGET_NOT_FOUND: No Android semantic node matched the requested selector."
            .to_string()),
        1 => Ok((snapshot, matches.remove(0))),
        count => Err(format!(
            "AMBIGUOUS_TARGET: {count} Android semantic nodes matched the selector; refine the query before mutation."
        )),
    }
}

pub(crate) fn phone_transaction_condition_met(
    app: &AppHandle,
    state: &AppState,
    device_id: &str,
    step: &PhoneTransactionStepParam,
) -> Result<bool, String> {
    let condition = step
        .condition
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    if matches!(
        condition.as_str(),
        "foreground_package" | "foregroundpackage"
    ) {
        let package_name = step.package_name.as_deref().unwrap_or_default().trim();
        return state
            .phone
            .foreground_package_is(app, device_id, package_name);
    }

    let snapshot = state.phone.semantic_snapshot(app, device_id, 800, None)?;

    match condition.as_str() {
        "semantic_exists" | "semanticexists" => Ok(!state
            .phone
            .semantic_find(
                device_id,
                &snapshot.snapshot.snapshot_id,
                phone_transaction_query(step),
            )?
            .is_empty()),
        "semantic_enabled" | "semanticenabled" => Ok(state
            .phone
            .semantic_find(
                device_id,
                &snapshot.snapshot.snapshot_id,
                phone_transaction_query(step),
            )?
            .iter()
            .any(|node| {
                node.states
                    .iter()
                    .any(|state| state.eq_ignore_ascii_case("enabled"))
            })),
        "focused_editable" | "focusededitable" => {
            Ok(phone_transaction_focused_editable(&snapshot).is_some())
        }
        "keyboard_visible" | "keyboardvisible" => Ok(snapshot.keyboard_visible == Some(true)),
        "keyboard_hidden" | "keyboardhidden" => Ok(snapshot.keyboard_visible == Some(false)),
        "text_equals" | "textequals" | "text_contains" | "textcontains" => {
            let expected = step.text.as_deref().unwrap_or_default();
            let candidates = if phone_transaction_has_selector(step) {
                state.phone.semantic_find(
                    device_id,
                    &snapshot.snapshot.snapshot_id,
                    phone_transaction_query(step),
                )?
            } else {
                phone_transaction_focused_editable(&snapshot)
                    .cloned()
                    .into_iter()
                    .collect()
            };
            Ok(candidates.iter().any(|node| {
                if node.sensitive {
                    return false;
                }
                let actual = node.value.as_deref().or(node.text.as_deref()).unwrap_or("");
                if matches!(condition.as_str(), "text_equals" | "textequals") {
                    actual == expected
                } else {
                    actual.contains(expected)
                }
            }))
        }
        "frame_stable" | "framestable" => Err(
            "INTERNAL_CONDITION: frame_stable must use the bounded live-frame waiter.".to_string(),
        ),
        _ => Err("INVALID_ARGUMENT: Unsupported Phone transaction condition.".to_string()),
    }
}

pub(crate) fn validate_phone_transaction_steps(steps: &[PhoneTransactionStepParam]) -> Result<(), String> {
    if steps.is_empty() || steps.len() > 64 {
        return Err("Phone transaction must contain between 1 and 64 steps.".to_string());
    }

    let mut aliases = BTreeMap::<String, ()>::new();
    for (index, step) in steps.iter().enumerate() {
        let step_number = index + 1;
        let operation = step.operation.trim().to_ascii_lowercase();
        match operation.as_str() {
            "launch_app" | "launchapp" => {
                let package_name = step
                    .package_name
                    .as_deref()
                    .ok_or_else(|| {
                        format!(
                            "Phone transaction step {step_number} launch_app requires package_name."
                        )
                    })?
                    .trim();
                if !phone::valid_android_package_name(package_name) {
                    return Err(format!(
                        "Phone transaction step {step_number} has an invalid Android package name."
                    ));
                }
            }
            "find" => {
                let alias = step
                    .as_name
                    .as_deref()
                    .ok_or_else(|| {
                        format!("Phone transaction step {step_number} find requires as_name.")
                    })?
                    .trim();
                if !valid_phone_transaction_alias(alias) {
                    return Err(format!(
                        "Phone transaction step {step_number} find has an invalid alias."
                    ));
                }
                if aliases.contains_key(alias) {
                    return Err(format!(
                        "Phone transaction step {step_number} reuses alias '{alias}'."
                    ));
                }
                if !phone_transaction_has_selector(step) {
                    return Err(format!(
                        "Phone transaction step {step_number} find requires at least one semantic selector."
                    ));
                }
                aliases.insert(alias.to_string(), ());
            }
            "click" => {
                let alias = step
                    .target
                    .as_deref()
                    .ok_or_else(|| {
                        format!("Phone transaction step {step_number} click requires target.")
                    })?
                    .trim();
                if !aliases.contains_key(alias) {
                    return Err(format!(
                        "Phone transaction step {step_number} click references unknown alias '{alias}'."
                    ));
                }
            }
            "set_text" | "settext" | "type" => {
                let target = step.target.as_deref().unwrap_or("focused").trim();
                if target != "focused" && !aliases.contains_key(target) {
                    return Err(format!(
                        "Phone transaction step {step_number} set_text references unknown alias '{target}'."
                    ));
                }
                let text = step.text.as_deref().ok_or_else(|| {
                    format!("Phone transaction step {step_number} set_text requires text.")
                })?;
                if text.is_empty()
                    || text.chars().count() > 2_000
                    || text.chars().any(char::is_control)
                {
                    return Err(format!(
                        "Phone transaction step {step_number} text must contain 1..2000 non-control characters."
                    ));
                }
            }
            "key" => {
                let key = step.key.as_deref().ok_or_else(|| {
                    format!("Phone transaction step {step_number} key requires key.")
                })?;
                parse_phone_key(key).map_err(|error| {
                    format!("Phone transaction step {step_number}: {error}")
                })?;
            }
            "wait_until" | "waituntil" | "verify" => {
                let condition = step
                    .condition
                    .as_deref()
                    .ok_or_else(|| {
                        format!(
                            "Phone transaction step {step_number} {operation} requires condition."
                        )
                    })?
                    .trim()
                    .to_ascii_lowercase();
                match condition.as_str() {
                    "foreground_package" | "foregroundpackage" => {
                        let package_name = step.package_name.as_deref().ok_or_else(|| {
                            format!(
                                "Phone transaction step {step_number} foreground_package requires package_name."
                            )
                        })?;
                        if !phone::valid_android_package_name(package_name.trim()) {
                            return Err(format!(
                                "Phone transaction step {step_number} has an invalid Android package name."
                            ));
                        }
                    }
                    "semantic_exists" | "semanticexists" | "semantic_enabled"
                    | "semanticenabled" => {
                        if !phone_transaction_has_selector(step) {
                            return Err(format!(
                                "Phone transaction step {step_number} {condition} requires a semantic selector."
                            ));
                        }
                    }
                    "focused_editable" | "focusededitable" | "keyboard_visible"
                    | "keyboardvisible" | "keyboard_hidden" | "keyboardhidden" => {}
                    "text_equals" | "textequals" | "text_contains" | "textcontains" => {
                        let text = step.text.as_deref().ok_or_else(|| {
                            format!(
                                "Phone transaction step {step_number} {condition} requires text."
                            )
                        })?;
                        if text.is_empty()
                            || text.chars().count() > 2_000
                            || text.chars().any(char::is_control)
                        {
                            return Err(format!(
                                "Phone transaction step {step_number} condition text must contain 1..2000 non-control characters."
                            ));
                        }
                    }
                    "frame_stable" | "framestable" => {
                        let stable_count = step.stable_count.unwrap_or(2);
                        let interval_ms = step.interval_ms.unwrap_or(80);
                        if !(2..=20).contains(&stable_count) {
                            return Err(format!(
                                "Phone transaction step {step_number} frame_stable count must be 2..20."
                            ));
                        }
                        if !(20..=500).contains(&interval_ms) {
                            return Err(format!(
                                "Phone transaction step {step_number} frame_stable interval must be 20..500 ms."
                            ));
                        }
                    }
                    _ => {
                        return Err(format!(
                            "Phone transaction step {step_number} condition must be foreground_package, semantic_exists, semantic_enabled, focused_editable, keyboard_visible, keyboard_hidden, text_equals, text_contains, or frame_stable."
                        ))
                    }
                }

                if operation != "verify" {
                    let timeout_ms = step.timeout_ms.unwrap_or(5_000);
                    if !(50..=10_000).contains(&timeout_ms) {
                        return Err(format!(
                            "Phone transaction step {step_number} timeout must be 50..10000 ms."
                        ));
                    }
                }
            }
            _ => {
                return Err(format!(
                    "Phone transaction step {step_number} operation must be launch_app, find, click, set_text, key, wait_until, or verify."
                ))
            }
        }
    }

    Ok(())
}

