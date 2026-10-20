//! Windows Job Object primitive for future RazedBrowser process containment.
//!
//! IMPORTANT: This module cannot safely launch a browser yet. A process must
//! be created suspended, assigned to the job before any code executes, and only
//! then resumed. Spawning normally and attaching afterward has an escape race.
//! Job Objects also do not restrict filesystem access or network egress.
#[cfg(windows)]
mod windows {
    use std::{mem::size_of, ptr::{null, null_mut}, os::windows::ffi::OsStrExt, path::Path, time::{Duration, Instant}};
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::JobObjects::{
            CreateJobObjectW, SetInformationJobObject, AssignProcessToJobObject, IsProcessInJob, TerminateJobObject,
            JobObjectExtendedLimitInformation, JobObjectBasicAccountingInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, QueryInformationJobObject,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        },
    };

    use windows_sys::Win32::System::Threading::{
        CreateProcessW, ResumeThread, TerminateProcess, WaitForSingleObject,
        PROCESS_INFORMATION, STARTUPINFOW, CREATE_SUSPENDED, CREATE_NO_WINDOW,
    };

    use crate::razedbrowser_profile::DisposableProfile;

    /// Shutdown must be verified before deleting profile data.
    /// On an unverified shutdown, preserve the directory for investigation.
    pub(crate) fn shutdown_and_cleanup(
        process: ContainedProcess,
        profile: DisposableProfile,
    ) -> Result<(), String> {
        cleanup_after_shutdown(|| process.terminate(), profile)
    }

    /// Separation permits deterministic failure-path testing without launching
    /// a process. Production always supplies ContainedProcess::terminate.
    fn cleanup_after_shutdown(
        shutdown: impl FnOnce() -> Result<(), String>,
        mut profile: DisposableProfile,
    ) -> Result<(), String> {
        if let Err(error) = shutdown() {
            profile.preserve_on_drop();
            return Err(format!("Process shutdown unverified; profile preserved at {}: {error}", profile.path().display()));
        }
        if let Err(error) = profile.cleanup() {
            // Cleanup can fail after a partial deletion. Preserve what remains
            // rather than allowing Drop to hide the failure with another attempt.
            profile.preserve_on_drop();
            return Err(format!("Profile cleanup incomplete at {}: {error}", profile.path().display()));
        }
        Ok(())
    }

    pub(crate) struct ContainedProcess {
        process: HANDLE,
        job: BrowserJob,
    }

    impl ContainedProcess {
        /// Terminate the entire job and confirm all assigned processes exited.
        pub(crate) fn terminate(self) -> Result<(), String> {
            if unsafe { TerminateJobObject(self.job.handle, 1) } == 0 {
                return Err(format!("Job termination failed: {}", std::io::Error::last_os_error()));
            }
            // A single deadline bounds both root-process and child-process
            // verification. Timeout must never be treated as safe cleanup.
            let deadline = Instant::now() + Duration::from_secs(10);
            let wait = unsafe { WaitForSingleObject(self.process, 10_000) };
            if wait == WAIT_TIMEOUT {
                return Err("Root process did not exit within shutdown deadline.".into());
            }
            if wait != WAIT_OBJECT_0 {
                return Err(format!("Root process wait failed: {}", std::io::Error::last_os_error()));
            }
            loop {
                let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { std::mem::zeroed() };
                let ok = unsafe {
                    QueryInformationJobObject(
                        self.job.handle,
                        JobObjectBasicAccountingInformation,
                        (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                        size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                        null_mut(),
                    )
                };
                if ok == 0 { return Err(format!("Job accounting query failed: {}", std::io::Error::last_os_error())); }
                if info.ActiveProcesses == 0 { return Ok(()); }
                if Instant::now() >= deadline {
                    return Err(format!("{} job processes still active after termination.", info.ActiveProcesses));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }

    impl Drop for ContainedProcess {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.process); }
        }
    }

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

        /// Launch a fixed executable with no user-supplied command line.
        /// The process is suspended until Windows confirms job membership.
        /// This is not yet an approved browser launch path: no network or
        /// filesystem boundary has been established.
        pub(crate) fn launch_suspended(self, executable: &Path) -> Result<ContainedProcess, String> {
            if !executable.is_absolute() || !executable.is_file() {
                return Err("Executable must be an existing absolute file path.".into());
            }
            self.launch_with_command_line(executable, None)
        }

        // Command lines are deliberately unavailable to production callers.
        // Used only to exercise parent/child containment with Windows utilities.
        fn launch_with_command_line(self, executable: &Path, command_line: Option<&str>) -> Result<ContainedProcess, String> {
            if !executable.is_absolute() || !executable.is_file() {
                return Err("Executable must be an existing absolute file path.".into());
            }
            let application: Vec<u16> = executable.as_os_str().encode_wide().chain(Some(0)).collect();
            let mut command: Option<Vec<u16>> = command_line.map(|value| value.encode_utf16().chain(Some(0)).collect());
            let mut startup: STARTUPINFOW = unsafe { std::mem::zeroed() };
            startup.cb = size_of::<STARTUPINFOW>() as u32;
            let mut info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
            let created = unsafe { CreateProcessW(application.as_ptr(), command.as_mut().map_or(null_mut(), |v| v.as_mut_ptr()), null(), null(), 0, CREATE_SUSPENDED | CREATE_NO_WINDOW, null(), null(), &startup, &mut info) };
            if created == 0 { return Err(format!("Suspended process creation failed: {}", std::io::Error::last_os_error())); }
            // Every failure before ResumeThread kills the still-suspended child.
            let assigned = unsafe { AssignProcessToJobObject(self.handle, info.hProcess) };
            let mut member = 0i32;
            let verified = assigned != 0 && unsafe { IsProcessInJob(info.hProcess, self.handle, &mut member) } != 0 && member != 0;
            if !verified {
                unsafe { TerminateProcess(info.hProcess, 1); WaitForSingleObject(info.hProcess, u32::MAX); CloseHandle(info.hThread); CloseHandle(info.hProcess); }
                return Err(format!("Cannot verify suspended process containment: {}", std::io::Error::last_os_error()));
            }
            let resumed = unsafe { ResumeThread(info.hThread) };
            unsafe { CloseHandle(info.hThread); }
            if resumed == u32::MAX {
                unsafe { TerminateJobObject(self.handle, 1); WaitForSingleObject(info.hProcess, u32::MAX); CloseHandle(info.hProcess); }
                return Err(format!("Cannot resume contained process: {}", std::io::Error::last_os_error()));
            }
            Ok(ContainedProcess { process: info.hProcess, job: self })
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
        fn live_process_is_assigned_and_job_wide_shutdown_verified() {
            // This launches only a local Windows utility, never Chromium or a URL.
            // Use a stable System32 executable rather than searching PATH.
            let system_root = std::env::var_os("SystemRoot").expect("Windows SystemRoot");
            let executable = std::path::PathBuf::from(system_root)
                .join("System32").join("notepad.exe");
            assert!(executable.is_file(), "Windows Notepad executable unavailable");
            let job = BrowserJob::create().unwrap();
            let process = job.launch_suspended(&executable).unwrap();
            let mut member = 0i32;
            let status = unsafe { IsProcessInJob(process.process, process.job.raw_handle(), &mut member) };
            assert_ne!(status, 0, "Job membership query failed");
            assert_ne!(member, 0, "Launched process escaped the job");
            process.terminate().unwrap(); // Also verifies zero active processes in the job.
        }

        #[test]
        fn child_processes_are_contained_and_terminated() {
            // cmd launches a long-running ping child. Both must remain in the job.
            let root = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32");
            let cmd = root.join("cmd.exe");
            let job = BrowserJob::create().unwrap();
            let process = job.launch_with_command_line(
                &cmd,
                Some(r#"cmd.exe /D /C ping.exe -t 127.0.0.1 >NUL"#),
            ).unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut observed_two = false;
            while Instant::now() < deadline {
                let mut accounting: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { std::mem::zeroed() };
                let ok = unsafe {
                    QueryInformationJobObject(
                        process.job.raw_handle(),
                        JobObjectBasicAccountingInformation,
                        (&mut accounting as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                        size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                        null_mut(),
                    )
                };
                assert_ne!(ok, 0, "Cannot query process count");
                if accounting.ActiveProcesses >= 2 { observed_two = true; break; }
                std::thread::sleep(Duration::from_millis(25));
            }
            // Always terminate even when child creation did not occur.
            process.terminate().unwrap();
            assert!(observed_two, "Parent and child were not both observed inside job");
        }

        #[test]
        fn failed_shutdown_preserves_profile_and_reports_location() {
            let profile = DisposableProfile::create().unwrap();
            let path = profile.path().to_owned();
            std::fs::write(path.join("session-data"), b"retain").unwrap();
            let error = cleanup_after_shutdown(|| Err("simulated termination failure".into()), profile).unwrap_err();
            assert!(error.contains("simulated termination failure"));
            assert!(error.contains(&path.display().to_string()));
            assert!(path.join("session-data").exists());
            std::fs::remove_dir_all(path).unwrap();
        }

        #[test]
        fn missing_profile_after_verified_shutdown_reports_cleanup_failure() {
            let profile = DisposableProfile::create().unwrap();
            let path = profile.path().to_owned();
            std::fs::remove_dir_all(&path).unwrap();
            let error = cleanup_after_shutdown(|| Ok(()), profile).unwrap_err();
            assert!(error.contains("Profile cleanup incomplete"));
        }

        #[test]
        fn verified_shutdown_removes_disposable_profile() {
            let root = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap());
            let executable = root.join("System32").join("notepad.exe");
            let profile = DisposableProfile::create().unwrap();
            let path = profile.path().to_owned();
            std::fs::write(path.join("session-data"), b"ephemeral").unwrap();
            let process = BrowserJob::create().unwrap().launch_suspended(&executable).unwrap();
            shutdown_and_cleanup(process, profile).unwrap();
            assert!(!path.exists(), "Verified shutdown must remove profile data");
        }

        #[test]
        fn relative_executable_is_rejected_before_process_creation() {
            let job = BrowserJob::create().unwrap();
            assert!(job.launch_suspended(std::path::Path::new("cmd.exe")).is_err());
        }

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
