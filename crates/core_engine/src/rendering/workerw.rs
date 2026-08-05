use anyhow::Result;

/// Resolves the WorkerW desktop window handle spawned behind Windows desktop icons via Progman 0x052C message.
#[cfg(windows)]
pub fn find_desktop_workerw_hwnd() -> Option<windows::Win32::Foundation::HWND> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, FindWindowExW, FindWindowW, SendMessageTimeoutW, SMTO_NORMAL,
    };

    unsafe {
        let progman = match FindWindowW(windows::core::w!("Progman"), None) {
            Ok(hwnd) if !hwnd.0.is_null() => hwnd,
            _ => return None,
        };

        let mut result = 0;
        let _ = SendMessageTimeoutW(
            progman,
            0x052C,
            WPARAM(0),
            LPARAM(0),
            SMTO_NORMAL,
            1000,
            Some(&mut result),
        );

        let mut workerw_hwnd = HWND(std::ptr::null_mut());

        unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let target_ptr = lparam.0 as *mut HWND;
            if let Ok(shell_hwnd) = FindWindowExW(
                hwnd,
                HWND(std::ptr::null_mut()),
                windows::core::w!("SHELLDLL_DefView"),
                None,
            ) {
                if !shell_hwnd.0.is_null() {
                    if let Ok(found_workerw) = FindWindowExW(
                        HWND(std::ptr::null_mut()),
                        hwnd,
                        windows::core::w!("WorkerW"),
                        None,
                    ) {
                        *target_ptr = found_workerw;
                        return BOOL(0);
                    }
                }
            }
            BOOL(1)
        }

        let _ = EnumWindows(
            Some(enum_window_proc),
            LPARAM(&mut workerw_hwnd as *mut HWND as isize),
        );

        if !workerw_hwnd.0.is_null() {
            Some(workerw_hwnd)
        } else {
            None
        }
    }
}

#[cfg(not(windows))]
pub fn find_desktop_workerw_hwnd() -> Option<()> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workerw_window_query() {
        // Safe query check: on Windows host returns Option<HWND>, on non-Windows returns None.
        let _result = find_desktop_workerw_hwnd();
    }
}
