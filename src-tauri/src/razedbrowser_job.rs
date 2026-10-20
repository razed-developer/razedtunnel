//! Windows Job Object primitive for future RazedBrowser process containment.
//!
//! IMPORTANT: This module cannot safely launch a browser yet. A process must
//! be created suspended, assigned to the job before any code executes, and only
//! then resumed. Spawning normally and attaching afterward has an escape race.
//! Job Objects also do not restrict filesystem access or network egress.
#[cfg(windows)]
mod windows {
    use std::{mem::size_of, ptr::null};
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
        System::JobObjects::{
            CreateJobObjectW, SetInformationJobObject,
            JobObjectExtendedLimitInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        },
    };

    pub(crate) struct BrowserJob {
        handle: HANDLE,
    }

    impl BrowserJob {
        /// Create an empty job that terminates every assigned process on close.
        /// No browser process is started or attached by this constructor.
        pub(crate) fn create() -> Result<Self, String> {
            let handle = unsafe { CreateJobObjectW(null(), null()) };
            if handle.is_null() || handle == INVALID_HANDLE_VALUE {
                return Err(format!("Cannot create browser Job Object: {}", std::io::Error::last_os_error()));
            }
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let ok = unsafe {
                SetInformationJobObject(
                    handle,
                    JobObjectExtendedLimitInformation,
                    (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            };
            if ok == 0 {
                let error = std::io::Error::last_os_error();
                unsafe { CloseHandle(handle) };
                return Err(format!("Cannot configure kill-on-close Job Object: {error}"));
            }
            Ok(Self { handle })
        }

        /// A job handle alone is not proof of process containment.
        /// Future launch code must use suspended creation and verify assignment.
        pub(crate) fn raw_handle(&self) -> HANDLE {
            self.handle
        }
    }

    impl Drop for BrowserJob {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.handle) };
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use windows_sys::Win32::System::JobObjects::{
            QueryInformationJobObject, JobObjectExtendedLimitInformation,
        };

        #[test]
        fn empty_job_has_kill_on_close_enabled() {
            let job = BrowserJob::create().unwrap();
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
            let ok = unsafe {
                QueryInformationJobObject(
                    job.raw_handle(),
                    JobObjectExtendedLimitInformation,
                    (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                    std::ptr::null_mut(),
                )
            };
            assert_ne!(ok, 0);
            assert_ne!(limits.BasicLimitInformation.LimitFlags & JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, 0);
        }
    }
}

#[cfg(windows)]
pub(crate) use windows::BrowserJob;
