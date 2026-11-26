#[cfg(windows)]
mod platform {
    use std::ffi::c_void;
    use std::io;
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject, TerminateJobObject,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
    };

    pub struct ProcessGuard {
        job: HANDLE,
    }

    // HANDLE es un descriptor de kernel seguro para mover entre hilos mientras no se comparta sin sincronizacion.
    unsafe impl Send for ProcessGuard {}

    pub fn configure_command(_cmd: &mut tokio::process::Command) {}

    impl ProcessGuard {
        pub fn attach(pid: u32) -> io::Result<Self> {
            unsafe {
                let job = CreateJobObjectW(null(), null());
                if job.is_null() {
                    return Err(io::Error::last_os_error());
                }

                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

                let set_ok = SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    (&raw const info).cast::<c_void>(),
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                );
                if set_ok == 0 {
                    let err = io::Error::last_os_error();
                    CloseHandle(job);
                    return Err(err);
                }

                let proc_handle = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, pid);
                if !proc_handle.is_null() {
                    let _ = AssignProcessToJobObject(job, proc_handle);
                    CloseHandle(proc_handle);
                }

                Ok(Self { job })
            }
        }

        pub fn kill_tree(&mut self) {
            if !self.job.is_null() {
                unsafe {
                    TerminateJobObject(self.job, 1);
                    CloseHandle(self.job);
                }
                self.job = null_mut();
            }
        }

        pub fn disarm(&mut self) {
            if !self.job.is_null() {
                unsafe {
                    CloseHandle(self.job);
                }
                self.job = null_mut();
            }
        }
    }

    impl Drop for ProcessGuard {
        fn drop(&mut self) {
            if !self.job.is_null() {
                unsafe {
                    CloseHandle(self.job);
                }
                self.job = null_mut();
            }
        }
    }
}

#[cfg(unix)]
mod platform {
    use std::io;

    pub struct ProcessGuard {
        pgid: libc::pid_t,
        active: bool,
    }

    pub fn configure_command(cmd: &mut tokio::process::Command) {
        cmd.process_group(0);
    }

    impl ProcessGuard {
        pub fn attach(pid: u32) -> io::Result<Self> {
            Ok(Self {
                pgid: pid as libc::pid_t,
                active: true,
            })
        }

        pub fn kill_tree(&mut self) {
            if self.active && self.pgid > 0 {
                unsafe {
                    libc::killpg(self.pgid, libc::SIGKILL);
                }
                self.active = false;
            }
        }

        pub fn disarm(&mut self) {
            self.active = false;
        }
    }

    impl Drop for ProcessGuard {
        fn drop(&mut self) {
            if self.active {
                self.kill_tree();
            }
        }
    }
}

pub use platform::{ProcessGuard, configure_command};
