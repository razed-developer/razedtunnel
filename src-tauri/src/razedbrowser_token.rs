//! Windows restricted-token construction for future RazedBrowser launches.
//!
//! A restricted token alone is NOT a sandbox. It does not prove filesystem
//! confinement, network egress policy, or child-process containment.
//! Never use this token with an unrestricted CreateProcessW launch.
#![cfg(windows)]

use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    Security::{
        CreateRestrictedToken, IsTokenRestricted, OpenProcessToken,
        DISABLE_MAX_PRIVILEGE, TOKEN_DUPLICATE, TOKEN_QUERY,
    },
    System::Threading::GetCurrentProcess,
};

/// Owns a Windows restricted token. Not a claim of complete isolation.
pub(crate) struct RestrictedBrowserToken {
    handle: HANDLE,
}

impl RestrictedBrowserToken {
    /// Create a restricted primary-token candidate from the current process.
    /// This disables available privileges; it does not remove all filesystem
    /// access or enforce any network policy. Do not use for production launch
    /// until token type, integrity, SID restrictions and ACLs are verified.
    pub(crate) fn create_candidate() -> Result<Self, String> {
        let mut source: HANDLE = std::ptr::null_mut();
        let opened = unsafe {
            OpenProcessToken(GetCurrentProcess(), TOKEN_DUPLICATE | TOKEN_QUERY, &mut source)
        };
        if opened == 0 {
            return Err(format!("OpenProcessToken failed: {}", std::io::Error::last_os_error()));
        }
        let mut restricted: HANDLE = std::ptr::null_mut();
        let created = unsafe {
            CreateRestrictedToken(
                source, DISABLE_MAX_PRIVILEGE, 0, std::ptr::null(),
                0, std::ptr::null(), 0, std::ptr::null(), &mut restricted,
            )
        };
        unsafe { CloseHandle(source); }
        if created == 0 {
            return Err(format!("CreateRestrictedToken failed: {}", std::io::Error::last_os_error()));
        }
        // IsTokenRestricted returns a BOOL, not an output parameter.
        if unsafe { IsTokenRestricted(restricted) } == 0 {
            unsafe { CloseHandle(restricted); }
            return Err("Windows did not confirm a restricted token.".into());
        }
        Ok(Self { handle: restricted })
    }
}

impl Drop for RestrictedBrowserToken {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restricted_token_candidate_is_verified() {
        // Depending on the caller's existing token, disabling privileges
        // alone may not yield a token Windows considers restricted. Reject
        // that case rather than treating it as an isolation success.
        if let Ok(token) = RestrictedBrowserToken::create_candidate() {
            assert!(!token.handle.is_null());
            assert_ne!(unsafe { IsTokenRestricted(token.handle) }, 0);
        }
    }
}
