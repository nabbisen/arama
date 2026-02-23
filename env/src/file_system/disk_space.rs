use std::path::Path;

pub struct DiskSpace {
    pub available: u64,
    pub total: u64,
}

impl DiskSpace {
    pub fn new(path: &Path) -> Result<DiskSpace, std::io::Error> {
        #[cfg(not(windows))]
        {
            use std::ffi::CString;

            let c_path = CString::new(path.to_str().unwrap()).unwrap();
            let mut stat = unsafe { std::mem::zeroed::<libc::statvfs>() };
            let ret = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
            if ret != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(DiskSpace {
                available: stat.f_bavail as u64 * stat.f_frsize as u64,
                total: stat.f_blocks as u64 * stat.f_frsize as u64,
            })
        }

        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
            let (mut free, mut total, mut tf) = (0u64, 0u64, 0u64);

            #[link(name = "kernel32")]
            unsafe extern "system" {
                fn GetDiskFreeSpaceExW(
                    path: *const u16,
                    free: *mut u64,
                    total: *mut u64,
                    total_free: *mut u64,
                ) -> i32;
            }
            let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut free, &mut total, &mut tf) };

            if ok == 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(DiskSpace {
                available: free,
                total,
            })
        }
    }

    pub fn as_gb(&self) -> Self {
        Self {
            available: self.available / 1024u64.pow(3),
            total: self.total / 1024u64.pow(3),
        }
    }
}
