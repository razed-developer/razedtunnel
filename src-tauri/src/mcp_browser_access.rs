use tauri::AppHandle;
use crate::{ai_resources, browser, gmail_access, models::Workspace};

pub(crate) fn browser_scope_for_request(
    workspace: &Workspace,
    client_key: Option<&str>,
) -> browser::BrowserScope {
    let owner_id = client_key.map(ai_resources::opaque_owner_id);
    browser::BrowserScope::new(workspace, owner_id.as_deref())
}

pub(crate) fn require_google_tab_access(
    app: &AppHandle,
    scope: &browser::BrowserScope,
    tab_id: &str,
) -> Result<(), String> {
    let tab = browser::list_tabs(app, scope)?
        .into_iter()
        .find(|tab| tab.id == tab_id)
        .ok_or_else(|| {
            "That managed browser tab is not available to this AI browser session.".to_string()
        })?;
    gmail_access::require_url_access(app, &tab.url)
}

pub(crate) fn require_ai_browser_tab_access(
    workspace: &Workspace,
    client_key: Option<&str>,
    tab_id: &str,
) -> Result<(), String> {
    if let Some(client_key) = client_key {
        ai_resources::claim_or_assert_browser_tab(&workspace.id, client_key, tab_id)?;
    }
    Ok(())
}

