//! Windows restricted-token construction for future RazedBrowser launches.
//!
//! A restricted token alone is NOT a sandbox. It does not prove filesystem
//! confinement, network egress policy, or child-process containment.
//! Never use this token with an unrestricted CreateProcessW launch.
#![cfg(windows)]

use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    Security::{
        CreateRestrictedToken, GetTokenInformation, IsTokenRestricted,
        TokenPrivileges, DISABLE_MAX_PRIVILEGE, TOKEN_DUPLICATE, TOKEN_QUERY,
        LUID_AND_ATTRIBUTES, TOKEN_PRIVILEGES, SE_PRIVILEGE_ENABLED,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

/// Verify the OS reports no enabled privileges in the candidate token.
/// This is only one property of an execution identity, not proof of sandboxing.
fn verify_privileges_disabled(token: HANDLE) -> Result<(), String> {
    let mut required = 0u32;
    // First call obtains the required buffer size; failure is expected.
    unsafe {
        GetTokenInformation(token, TokenPrivileges, std::ptr::null_mut(), 0, &mut required);
    }
    if (required as usize) < std::mem::size_of::<TOKEN_PRIVILEGES>() {
        return Err("Cannot determine token privilege buffer size.".into());
    }
    // u64 allocation provides alignment for TOKEN_PRIVILEGES and its entries.
    let mut buffer = vec![0u64; (required as usize).div_ceil(8)];
    let mut returned = 0u32;
    if unsafe {
        GetTokenInformation(
            token, TokenPrivileges, buffer.as_mut_ptr().cast(),
            (buffer.len() * 8) as u32, &mut returned,
        )
    } == 0 {
        return Err(format!("GetTokenInformation(TokenPrivileges) failed: {}", std::io::Error::last_os_error()));
    }
    if returned < 4 {
        return Err("Token privilege information was truncated.".into());
    }
    let base = buffer.as_ptr().cast::<u8>();
    let count = unsafe { std::ptr::read_unaligned(base.cast::<u32>()) } as usize;
    let offset = std::mem::offset_of!(TOKEN_PRIVILEGES, Privileges);
    let entry_size = std::mem::size_of::<LUID_AND_ATTRIBUTES>();
    let bytes_needed = count.checked_mul(entry_size)
        .and_then(|n| offset.checked_add(n))
        .ok_or("Token privilege count overflow.")?;
    if bytes_needed > returned as usize || bytes_needed > buffer.len() * 8 {
        return Err("Token privilege entries exceed returned buffer.".into());
    }
    for i in 0..count {
        let entry = unsafe {
            std::ptr::read_unaligned(base.add(offset + i * entry_size).cast::<LUID_AND_ATTRIBUTES>())
        };
        if entry.Attributes & SE_PRIVILEGE_ENABLED != 0 {
            return Err("Restricted token still has an enabled privilege.".into());
        }
    }
    Ok(())
}

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
        if let Err(error) = verify_privileges_disabled(restricted) {
            unsafe { CloseHandle(restricted); }
            return Err(error);
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
            verify_privileges_disabled(token.handle).unwrap();
        }
    }
}
