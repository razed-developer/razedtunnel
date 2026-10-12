#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct ArmSelfContinuationParams {
    /// Approved workspace whose current AI work should be recoverable.
    pub(crate) workspace_id: String,
    /// One specific AI-written sentence describing the exact remaining work and what is already complete.
    pub(crate) continuation_sentence: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SelfContinuationModeParam {
    Working,
    WaitingUser,
    Completed,
    Disabled,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct UpdateSelfContinuationParams {
    pub(crate) workspace_id: String,
    pub(crate) watch_id: String,
    /// Replace the one current sentence. Required when meaningful remaining work changes; omit when only changing state.
    pub(crate) continuation_sentence: Option<String>,
    pub(crate) state: SelfContinuationModeParam,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct SelfContinuationWatchParams {
    pub(crate) workspace_id: String,
    pub(crate) watch_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct SelfContinuationAttemptParams {
    pub(crate) workspace_id: String,
    pub(crate) watch_id: String,
    /// Stable random ID for this mounted MCP App instance. Used only to prevent concurrent duplicate sends.
    pub(crate) claimant_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct SelfContinuationDeliveryParams {
    pub(crate) workspace_id: String,
    pub(crate) watch_id: String,
    pub(crate) recovery_id: String,
    /// Stable random ID for this mounted MCP App instance. Used only to prevent concurrent duplicate sends.
    pub(crate) claimant_id: String,
}

