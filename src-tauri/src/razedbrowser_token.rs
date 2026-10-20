//! Windows restricted-token construction for future RazedBrowser launches.
//!
//! A restricted token alone is NOT a sandbox. It does not prove filesystem
//! confinement, network egress policy, or child-process containment.
//! Never use this token with an unrestricted CreateProcessW launch.
#![cfg(windows)]

use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, LUID},
    Security::{
        CreateRestrictedToken, GetTokenInformation, IsTokenRestricted, SetTokenInformation,
        LookupPrivilegeValueW,
        GetSidSubAuthority, GetSidSubAuthorityCount, IsValidSid,
        TokenPrivileges, TokenIntegrityLevel, TokenRestrictedSids, TokenUser, DISABLE_MAX_PRIVILEGE, TOKEN_DUPLICATE, TOKEN_QUERY,
        TOKEN_MANDATORY_LABEL, SID_AND_ATTRIBUTES, TOKEN_USER,
        LUID_AND_ATTRIBUTES, TOKEN_PRIVILEGES, SE_PRIVILEGE_ENABLED,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

/// Verify the OS reports no enabled privileges except SeChangeNotifyPrivilege.
/// Windows may retain SeChangeNotifyPrivilege when DISABLE_MAX_PRIVILEGE is used.\n/// This is only one property of an execution identity, not proof of sandboxing.
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
    // Resolve the allowed exception through Windows rather than assuming
    // the numeric LUID is constant across environments.
    let privilege_name: Vec<u16> = "SeChangeNotifyPrivilege".encode_utf16()
        .chain(std::iter::once(0)).collect();
    let mut change_notify_luid = LUID { LowPart: 0, HighPart: 0 };
    if unsafe {
        LookupPrivilegeValueW(std::ptr::null(), privilege_name.as_ptr(), &mut change_notify_luid)
    } == 0 {
        return Err(format!("LookupPrivilegeValueW failed: {}", std::io::Error::last_os_error()));
    }
    for i in 0..count {
        let entry = unsafe {
            std::ptr::read_unaligned(base.add(offset + i * entry_size).cast::<LUID_AND_ATTRIBUTES>())
        };
        if entry.Attributes & SE_PRIVILEGE_ENABLED != 0
            && (entry.Luid.LowPart != change_notify_luid.LowPart
                || entry.Luid.HighPart != change_notify_luid.HighPart)
        {
            return Err(format!(
                "Restricted token has an unexpected enabled privilege (LUID {}:{}).",
                entry.Luid.HighPart, entry.Luid.LowPart
            ));
        }
    }
    Ok(())
}

/// Inspect the token's mandatory integrity SID and reject malformed values.
/// Merely reading an integrity level does not lower it. A future launcher must
/// establish a suitably low level and verify it again on the launched process.
fn inspect_integrity_level(token: HANDLE) -> Result<u32, String> {
    let mut required = 0u32;
    unsafe { GetTokenInformation(token, TokenIntegrityLevel, std::ptr::null_mut(), 0, &mut required); }
    if (required as usize) < std::mem::size_of::<TOKEN_MANDATORY_LABEL>() {
        return Err("Cannot determine token integrity information size.".into());
    }
    let mut storage = vec![0usize; (required as usize).div_ceil(std::mem::size_of::<usize>())];
    let mut returned = 0u32;
    if unsafe {
        GetTokenInformation(token, TokenIntegrityLevel, storage.as_mut_ptr().cast(),
            (storage.len() * std::mem::size_of::<usize>()) as u32, &mut returned)
    } == 0 {
        return Err(format!("Token integrity query failed: {}", std::io::Error::last_os_error()));
    }
    if (returned as usize) < std::mem::size_of::<TOKEN_MANDATORY_LABEL>() {
        return Err("Token integrity information truncated.".into());
    }
    let label = unsafe { &*storage.as_ptr().cast::<TOKEN_MANDATORY_LABEL>() };
    let sid = label.Label.Sid;
    if sid.is_null() || unsafe { IsValidSid(sid) } == 0 {
        return Err("Token integrity SID is invalid.".into());
    }
    let count_ptr = unsafe { GetSidSubAuthorityCount(sid) };
    if count_ptr.is_null() { return Err("Token integrity SID lacks a subauthority count.".into()); }
    let count = unsafe { *count_ptr };
    if count == 0 { return Err("Token integrity SID has no RID.".into()); }
    let rid_ptr = unsafe { GetSidSubAuthority(sid, u32::from(count - 1)) };
    if rid_ptr.is_null() { return Err("Token integrity RID is unavailable.".into()); }
    Ok(unsafe { *rid_ptr })
}

/// Set a low mandatory integrity label (S-1-16-4096) on the candidate.
/// This restricts write-up access but does NOT confine reads or network access.
fn enforce_low_integrity(token: HANDLE) -> Result<(), String> {
    // SID revision 1, one subauthority, mandatory label authority 16,
    // little-endian low-integrity RID 4096.
    let mut low_sid: [u8; 12] = [1, 1, 0, 0, 0, 0, 0, 16, 0, 16, 0, 0];
    let mut label = TOKEN_MANDATORY_LABEL {
        Label: SID_AND_ATTRIBUTES {
            Sid: low_sid.as_mut_ptr().cast(),
            Attributes: 0x0000_0020, // SE_GROUP_INTEGRITY: mandatory integrity SID
        },
    };
    let size = (std::mem::size_of::<TOKEN_MANDATORY_LABEL>() + low_sid.len()) as u32;
    if unsafe {
        SetTokenInformation(token, TokenIntegrityLevel, (&mut label as *mut TOKEN_MANDATORY_LABEL).cast(), size)
    } == 0 {
        return Err(format!("SetTokenInformation(low integrity) failed: {}", std::io::Error::last_os_error()));
    }
    let actual = inspect_integrity_level(token)?;
    if actual != 0x1000 {
        return Err(format!("Token integrity verification failed: expected low (4096), got {actual}."));
    }
    Ok(())
}

/// Read the number of restricting SIDs reported by Windows.
/// A zero count means no restricting-SID allowlist was installed; disabling
/// privileges and lowering integrity do not substitute for this boundary.
fn inspect_restricting_sid_count(token: HANDLE) -> Result<u32, String> {
    let mut required = 0u32;
    unsafe { GetTokenInformation(token, TokenRestrictedSids, std::ptr::null_mut(), 0, &mut required); }
    if required < 4 {
        return Err("Cannot determine restricting-SID information size.".into());
    }
    let mut storage = vec![0usize; (required as usize).div_ceil(std::mem::size_of::<usize>())];
    let mut returned = 0u32;
    if unsafe {
        GetTokenInformation(token, TokenRestrictedSids, storage.as_mut_ptr().cast(),
            (storage.len() * std::mem::size_of::<usize>()) as u32, &mut returned)
    } == 0 {
        return Err(format!("Token restricting-SID query failed: {}", std::io::Error::last_os_error()));
    }
    if returned < 4 {
        return Err("Token restricting-SID information truncated.".into());
    }
    let count = unsafe { std::ptr::read_unaligned(storage.as_ptr().cast::<u32>()) };
    let offset = std::mem::offset_of!(windows_sys::Win32::Security::TOKEN_GROUPS, Groups);
    let needed = (count as usize)
        .checked_mul(std::mem::size_of::<SID_AND_ATTRIBUTES>())
        .and_then(|n| offset.checked_add(n))
        .ok_or("Restricting-SID count overflow.")?;
    if needed > returned as usize {
        return Err("Restricting-SID entries exceed returned data.".into());
    }
    Ok(count)
}

/// Obtain the caller's user SID as a baseline restricting SID.
/// IMPORTANT: the caller's own SID does not isolate the browser from files
/// accessible to that user. A dedicated sandbox SID and ACL boundary are needed.
fn source_user_sid(source: HANDLE) -> Result<(Vec<usize>, *mut core::ffi::c_void), String> {
    let mut required = 0u32;
    unsafe { GetTokenInformation(source, TokenUser, std::ptr::null_mut(), 0, &mut required); }
    if (required as usize) < std::mem::size_of::<TOKEN_USER>() {
        return Err("Cannot determine token user SID size.".into());
    }
    let mut storage = vec![0usize; (required as usize).div_ceil(std::mem::size_of::<usize>())];
    let mut returned = 0u32;
    if unsafe {
        GetTokenInformation(source, TokenUser, storage.as_mut_ptr().cast(),
            (storage.len() * std::mem::size_of::<usize>()) as u32, &mut returned)
    } == 0 {
        return Err(format!("TokenUser query failed: {}", std::io::Error::last_os_error()));
    }
    if (returned as usize) < std::mem::size_of::<TOKEN_USER>() {
        return Err("TokenUser information truncated.".into());
    }
    let sid = unsafe { (*storage.as_ptr().cast::<TOKEN_USER>()).User.Sid };
    if sid.is_null() || unsafe { IsValidSid(sid) } == 0 {
        return Err("TokenUser SID invalid.".into());
    }
    Ok((storage, sid))
}

/// Owns a Windows restricted token. Not a claim of complete isolation.
pub(crate) struct RestrictedBrowserToken {
    handle: HANDLE,
}

impl RestrictedBrowserToken {
    /// Create a restricted primary-token candidate from the current process.
    /// This disables available privileges and sets low integrity; it does not
    /// confine filesystem reads or enforce any network policy. Do not use for production launch
    /// until token type, integrity, SID restrictions and ACLs are verified.
    pub(crate) fn create_candidate() -> Result<Self, String> {
        let mut source: HANDLE = std::ptr::null_mut();
        let opened = unsafe {
            OpenProcessToken(GetCurrentProcess(), TOKEN_DUPLICATE | TOKEN_QUERY, &mut source)
        };
        if opened == 0 {
            return Err(format!("OpenProcessToken failed: {}", std::io::Error::last_os_error()));
        }
        let (_user_buffer, user_sid) = match source_user_sid(source) {
            Ok(result) => result,
            Err(error) => {
                unsafe { CloseHandle(source); }
                return Err(error);
            }
        };
        let restricting_sid = SID_AND_ATTRIBUTES { Sid: user_sid, Attributes: 0 };
        let mut restricted: HANDLE = std::ptr::null_mut();
        let created = unsafe {
            CreateRestrictedToken(
                source, DISABLE_MAX_PRIVILEGE, 0, std::ptr::null(),
                0, std::ptr::null(), 1, &restricting_sid, &mut restricted,
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
        if let Err(error) = enforce_low_integrity(restricted) {
            unsafe { CloseHandle(restricted); }
            return Err(error);
        }
        // A user-SID restriction is a verified baseline, NOT a sandbox-specific
        // allowlist or filesystem boundary.
        match inspect_restricting_sid_count(restricted) {
            Ok(1) => {}
            Ok(count) => {
                unsafe { CloseHandle(restricted); }
                return Err(format!("Expected one restricting SID, found {count}."));
            }
            Err(error) => {
                unsafe { CloseHandle(restricted); }
                return Err(error);
            }
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
        // Do not silently pass if token construction or OS verification fails.
        // This test is intentionally strict: it must exercise the actual
        // Windows token APIs on RazedRunner, not merely compile them.
        let token = RestrictedBrowserToken::create_candidate()
            .expect("Windows must create and verify the restricted-token candidate");
        assert!(!token.handle.is_null());
        assert_ne!(unsafe { IsTokenRestricted(token.handle) }, 0);
        verify_privileges_disabled(token.handle).unwrap();
        assert_eq!(inspect_integrity_level(token.handle).unwrap(), 0x1000);
        assert_eq!(inspect_restricting_sid_count(token.handle).unwrap(), 1,
            "candidate must have exactly one baseline restricting SID");
    }
}
