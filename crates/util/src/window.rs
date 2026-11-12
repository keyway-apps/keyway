use gpui_kit::Window;

/// Changes Windows native window visibility.
#[cfg(windows)]
pub fn set_native_window_visibility(window: &Window, visible: bool) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOW, ShowWindowAsync};

    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };

    // The borrowed window handle remains valid for this native call.
    unsafe {
        ShowWindowAsync(
            handle.hwnd.get() as *mut core::ffi::c_void,
            if visible { SW_SHOW } else { SW_HIDE },
        );
    }
}
