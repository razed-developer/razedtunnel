#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneTargetParams {
    /// Opaque phone ID returned by phone_status. Raw ADB serials, IP addresses, and ports are never accepted here.
    pub(crate) device_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneTapParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Horizontal position from 0..1 across the current phone display.
    pub(crate) x_ratio: f64,
    /// Vertical position from 0..1 across the current phone display.
    pub(crate) y_ratio: f64,
    /// Optional exact live frame ID from phone_fast_screen. When supplied, mutation fails closed if the phone has advanced to a different frame.
    pub(crate) expected_frame_id: Option<u64>,
    /// Optional display generation from Phone observation metadata. Mutation fails closed if display/orientation geometry changed.
    pub(crate) expected_display_generation: Option<u64>,
    /// Optional package expected to still be foreground when the tap is dispatched.
    pub(crate) expected_package: Option<String>,
    /// Optional activity component name expected to still be foreground when the tap is dispatched.
    pub(crate) expected_activity: Option<String>,
    /// Optional expected orientation: portrait or landscape.
    pub(crate) expected_orientation: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneSwipeParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Swipe start horizontal position from 0..1.
    pub(crate) start_x_ratio: f64,
    /// Swipe start vertical position from 0..1.
    pub(crate) start_y_ratio: f64,
    /// Swipe end horizontal position from 0..1.
    pub(crate) end_x_ratio: f64,
    /// Swipe end vertical position from 0..1.
    pub(crate) end_y_ratio: f64,
    /// Optional swipe duration in milliseconds. Defaults to 250 and is bounded to 50..3000.
    pub(crate) duration_ms: Option<u32>,
    /// Optional exact live frame ID from phone_fast_screen. When supplied, mutation fails closed if the frame changed.
    pub(crate) expected_frame_id: Option<u64>,
    /// Optional display generation from Phone observation metadata.
    pub(crate) expected_display_generation: Option<u64>,
    /// Optional package expected to still be foreground when the swipe is dispatched.
    pub(crate) expected_package: Option<String>,
    /// Optional activity component name expected to still be foreground when the swipe is dispatched.
    pub(crate) expected_activity: Option<String>,
    /// Optional expected orientation: portrait or landscape.
    pub(crate) expected_orientation: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneKeyParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// One of back, home, enter, recents, escape, tab, delete, dpad_up, dpad_down, dpad_left, or dpad_right.
    pub(crate) key: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneTextParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Text to type into the currently focused Android field. Control characters are rejected; use phone_key for Enter/Tab.
    pub(crate) text: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhonePackageParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Android package name such as com.android.settings.
    pub(crate) package_name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneShellParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Android-device shell command. This runs only inside the selected phone through ADB.
    pub(crate) command: String,
    /// Optional timeout in milliseconds, clamped to 1000..30000. Defaults to 10000.
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneLogsParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Maximum logcat lines to return. Defaults to 200 and is clamped to 1..1000.
    pub(crate) max_lines: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhonePingParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Hostname or IP address to ping from the Android phone.
    pub(crate) host: String,
    /// Optional echo request count. Defaults to 3 and is clamped to 1..5.
    pub(crate) count: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneSettingParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Android settings namespace: system, secure, or global.
    pub(crate) namespace: String,
    /// Android settings key.
    pub(crate) key: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneSettingWriteParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Android settings namespace: system, secure, or global.
    pub(crate) namespace: String,
    /// Android settings key.
    pub(crate) key: String,
    /// New settings value. The value is encoded before Android shell dispatch and is not echoed back.
    pub(crate) value: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhonePathParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Absolute Android path such as /sdcard/Download/file.txt.
    pub(crate) path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneFileWriteParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Absolute Android destination file path.
    pub(crate) path: String,
    /// Base64-encoded file bytes. Raw file data is limited to 8 MiB per request.
    data_base64: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub(crate) struct PhoneSequenceStepParam {
    /// One of tap, swipe, or wait.
    pub(crate) operation: String,
    /// Tap horizontal position from 0..1. Used only by tap.
    pub(crate) x_ratio: Option<f64>,
    /// Tap vertical position from 0..1. Used only by tap.
    pub(crate) y_ratio: Option<f64>,
    /// Swipe start horizontal position from 0..1.
    pub(crate) start_x_ratio: Option<f64>,
    /// Swipe start vertical position from 0..1.
    pub(crate) start_y_ratio: Option<f64>,
    /// Swipe end horizontal position from 0..1.
    pub(crate) end_x_ratio: Option<f64>,
    /// Swipe end vertical position from 0..1.
    pub(crate) end_y_ratio: Option<f64>,
    /// Swipe duration in milliseconds. Defaults to 250 for swipe.
    pub(crate) duration_ms: Option<u32>,
    /// Wait duration in milliseconds. Required for wait and bounded to 0..2000.
    pub(crate) wait_ms: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneSequenceParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Ordered already-grounded phone actions. RepoTunnel accepts 1..64 steps and at least one tap/swipe.
    pub(crate) steps: Vec<PhoneSequenceStepParam>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub(crate) struct PhoneFastSequenceStepParam {
    /// One of tap, swipe, key, type, launch_app, or wait.
    pub(crate) operation: String,
    /// Tap horizontal position from 0..1.
    pub(crate) x_ratio: Option<f64>,
    /// Tap vertical position from 0..1.
    pub(crate) y_ratio: Option<f64>,
    /// Swipe start horizontal position from 0..1.
    pub(crate) start_x_ratio: Option<f64>,
    /// Swipe start vertical position from 0..1.
    pub(crate) start_y_ratio: Option<f64>,
    /// Swipe end horizontal position from 0..1.
    pub(crate) end_x_ratio: Option<f64>,
    /// Swipe end vertical position from 0..1.
    pub(crate) end_y_ratio: Option<f64>,
    /// Swipe duration in milliseconds. Defaults to 180 and is bounded to 50..3000.
    pub(crate) duration_ms: Option<u32>,
    /// Android navigation/input key for operation=key.
    pub(crate) key: Option<String>,
    /// Text for operation=type. Text is not echoed back in the result.
    pub(crate) text: Option<String>,
    /// Exact Android package name for operation=launch_app.
    pub(crate) package_name: Option<String>,
    /// Wait duration for operation=wait. Bounded to 0..2000 ms.
    pub(crate) wait_ms: Option<u32>,
    /// State condition for operation=wait_until: foreground_package, frame_changed, frame_stable, keyboard_visible, or keyboard_hidden.
    pub(crate) condition: Option<String>,
    /// Condition timeout for operation=wait_until. Bounded to 50..10000 ms.
    pub(crate) timeout_ms: Option<u32>,
    /// Baseline live frame ID for condition=frame_changed.
    pub(crate) baseline_frame_id: Option<u64>,
    /// Consecutive equal-frame samples required for condition=frame_stable. Defaults to 2 and is bounded to 2..20.
    pub(crate) stable_count: Option<u32>,
    /// Sampling interval for condition=frame_stable. Defaults to 80 ms and is bounded to 20..500 ms.
    pub(crate) interval_ms: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneFastSequenceParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Ordered fast-path actions. RepoTunnel validates all steps before the first mutation and accepts 1..64 steps.
    pub(crate) steps: Vec<PhoneFastSequenceStepParam>,
    /// Return the newest live video frame in the same tool result. Defaults to true.
    pub(crate) return_screen: Option<bool>,
    /// When returning a screen, wait locally for a newer video frame after the actions. Defaults to 250 ms and is capped at 1500 ms.
    pub(crate) wait_for_frame_change_ms: Option<u32>,
    /// Optional extra local settle delay after the first changed frame. Defaults to 35 ms and is capped at 500 ms.
    pub(crate) settle_ms: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneFastScreenParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Previously observed capturedAt value. When supplied RepoTunnel can wait for a newer cached live-video frame.
    pub(crate) after_captured_at: Option<u64>,
    /// Local wait for a newer cached frame, in milliseconds. Defaults to 150 and is capped at 1000.
    pub(crate) wait_for_change_ms: Option<u32>,
    /// If true and the frame did not change after the optional wait, return compact metadata without retransmitting the same image.
    pub(crate) only_if_changed: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneSemanticSnapshotParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Maximum semantic Android nodes to return. Defaults to 400 and is clamped to 20..800.
    pub(crate) max_nodes: Option<usize>,
    /// Optional hash from a prior snapshot. When unchanged RepoTunnel may omit duplicate nodes.
    pub(crate) known_hash: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneSemanticFindParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Snapshot ID returned by phone_semantic_snapshot.
    pub(crate) snapshot_id: String,
    /// Optional free-text match across role/name/description/text/value.
    pub(crate) query: Option<String>,
    /// Optional role filter such as button, textbox, text, switch, or image.
    pub(crate) role: Option<String>,
    /// Optional accessible-name filter.
    pub(crate) name: Option<String>,
    /// Optional state filter such as enabled, focused, editable, or checked.
    pub(crate) state: Option<String>,
    /// Optional supported-action filter such as click or type.
    pub(crate) action: Option<String>,
    /// Maximum matching nodes to return. Defaults to 20 and is clamped to 1..100.
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneSemanticActionParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Snapshot ID returned by phone_semantic_snapshot.
    pub(crate) snapshot_id: String,
    /// Short-lived semantic ref such as e1.
    pub(crate) ref_id: String,
    /// One of click, type, or set_text.
    pub(crate) action: String,
    /// Text for type/set_text. Sensitive targets are blocked and the text is never echoed.
    pub(crate) text: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub(crate) struct PhoneTransactionStepParam {
    /// One of launch_app, find, click, set_text, key, wait_until, or verify.
    pub(crate) operation: String,
    /// Android package name for launch_app or wait_until foreground_package.
    pub(crate) package_name: Option<String>,
    /// Alias created by find, or "focused" for set_text.
    pub(crate) target: Option<String>,
    /// Alias to create for find.
    pub(crate) as_name: Option<String>,
    /// Optional semantic free-text query for find/wait/verify.
    pub(crate) query: Option<String>,
    /// Optional semantic role filter.
    pub(crate) role: Option<String>,
    /// Optional semantic accessible-name filter.
    pub(crate) name: Option<String>,
    /// Optional semantic state filter.
    pub(crate) state: Option<String>,
    /// Optional semantic supported-action filter.
    pub(crate) semantic_action: Option<String>,
    /// Text for set_text or text_equals/text_contains conditions. Never echoed in results.
    pub(crate) text: Option<String>,
    /// Android key for operation=key.
    pub(crate) key: Option<String>,
    /// wait_until/verify condition: foreground_package, semantic_exists, semantic_enabled, focused_editable, keyboard_visible, keyboard_hidden, text_equals, text_contains, frame_stable.
    pub(crate) condition: Option<String>,
    /// Condition timeout for wait_until, in milliseconds. Defaults to 5000 and is bounded to 50..10000.
    pub(crate) timeout_ms: Option<u32>,
    /// Stable-frame samples for frame_stable. Defaults to 2 and is bounded to 2..20.
    pub(crate) stable_count: Option<u32>,
    /// Stable-frame sampling interval. Defaults to 80 ms and is bounded to 20..500 ms.
    pub(crate) interval_ms: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PhoneTransactionParams {
    /// Opaque phone ID returned by phone_status.
    pub(crate) device_id: String,
    /// Ordered semantic/state-based steps. RepoTunnel validates every step before the first mutation. Accepts 1..64 steps.
    pub(crate) steps: Vec<PhoneTransactionStepParam>,
    /// Return a final semantic snapshot. Defaults true.
    pub(crate) return_semantic_snapshot: Option<bool>,
}

