use crate::phone;

pub(crate) fn parse_phone_key(value: &str) -> Result<phone::PhoneKey, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "back" => Ok(phone::PhoneKey::Back),
        "home" => Ok(phone::PhoneKey::Home),
        "enter" => Ok(phone::PhoneKey::Enter),
        "recents" => Ok(phone::PhoneKey::Recents),
        "escape" => Ok(phone::PhoneKey::Escape),
        "tab" => Ok(phone::PhoneKey::Tab),
        "delete" => Ok(phone::PhoneKey::Delete),
        "dpad_up" => Ok(phone::PhoneKey::DpadUp),
        "dpad_down" => Ok(phone::PhoneKey::DpadDown),
        "dpad_left" => Ok(phone::PhoneKey::DpadLeft),
        "dpad_right" => Ok(phone::PhoneKey::DpadRight),
        _ => Err(
            "Phone key must be one of back, home, enter, recents, escape, tab, delete, dpad_up, dpad_down, dpad_left, or dpad_right."
                .to_string(),
        ),
    }
}

pub(crate) fn parse_phone_settings_namespace(value: &str) -> Result<phone::PhoneSettingsNamespace, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "system" => Ok(phone::PhoneSettingsNamespace::System),
        "secure" => Ok(phone::PhoneSettingsNamespace::Secure),
        "global" => Ok(phone::PhoneSettingsNamespace::Global),
        _ => Err("Phone settings namespace must be system, secure, or global.".to_string()),
    }
}

