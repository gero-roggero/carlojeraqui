//! Ventanas que se pueden grabar solas (modo "Una ventana"): la lista para los ajustes y cómo volver
//! a encontrar la elegida. Se guarda su programa y su título, no la ventana: cada vez que se abre es otra.

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindow, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    GWL_EXSTYLE, GW_OWNER, WS_EX_TOOLWINDOW,
};

/// Una ventana abierta que se puede grabar.
pub struct Target {
    pub hwnd: HWND,
    /// Programa, como "chrome.exe".
    pub exe: String,
    pub title: String,
}

/// "Título — programa.exe", para mostrar.
pub fn label(exe: &str, title: &str) -> String {
    if title.is_empty() {
        exe.to_string()
    } else {
        format!("{title} — {exe}")
    }
}

/// Ventanas de aplicaciones abiertas (las que se ven en la barra de tareas), en orden de arriba hacia abajo.
pub fn list() -> Vec<Target> {
    unsafe extern "system" fn cb(h: HWND, lp: LPARAM) -> BOOL {
        let v = &mut *(lp.0 as *mut Vec<Target>);
        if let Some(t) = target(h) {
            v.push(t);
        }
        true.into()
    }
    let mut v: Vec<Target> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(cb), LPARAM(&mut v as *mut Vec<Target> as isize));
    }
    v
}

/// La ventana de ese programa: la del mismo título si está; si no, la primera de ese programa
/// (el título de un navegador o un juego cambia todo el tiempo).
pub fn find(exe: &str, title: &str) -> Option<Target> {
    let mut of_exe: Vec<Target> = list().into_iter().filter(|t| t.exe.eq_ignore_ascii_case(exe)).collect();
    let i = of_exe.iter().position(|t| t.title == title).unwrap_or(0);
    (i < of_exe.len()).then(|| of_exe.swap_remove(i))
}

unsafe fn target(h: HWND) -> Option<Target> {
    if !IsWindowVisible(h).as_bool() || GetWindowTextLengthW(h) == 0 {
        return None;
    }
    // Solo ventanas principales: ni diálogos de otra ventana ni paletas flotantes.
    if GetWindow(h, GW_OWNER).is_ok_and(|o| !o.is_invalid()) {
        return None;
    }
    if GetWindowLongW(h, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0 != 0 {
        return None;
    }
    // Las apps de la Tienda dejan ventanas invisibles ("ocultas" por Windows).
    let mut cloaked = 0u32;
    if DwmGetWindowAttribute(h, DWMWA_CLOAKED, &mut cloaked as *mut u32 as *mut _, 4).is_ok() && cloaked != 0 {
        return None;
    }
    let mut cls = [0u16; 64];
    let n = GetClassNameW(h, &mut cls).max(0) as usize;
    if matches!(String::from_utf16_lossy(&cls[..n]).as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd") {
        return None;
    }
    let mut pid = 0u32;
    GetWindowThreadProcessId(h, Some(&mut pid));
    if pid == 0 || pid == std::process::id() {
        return None;
    }
    let mut buf = [0u16; 512];
    let n = GetWindowTextW(h, &mut buf).max(0) as usize;
    let title = String::from_utf16_lossy(&buf[..n]);
    // Un programa con permisos de administrador (o un anticheat) no deja abrirse: el nombre sale de la lista.
    let exe = exe_of(pid).or_else(|| super::game::process_name(pid))?;
    Some(Target { hwnd: h, exe, title })
}

/// "chrome.exe" del proceso.
fn exe_of(pid: u32) -> Option<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
        let _ = CloseHandle(h);
        if !ok {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        path.rsplit(['\\', '/']).next().map(str::to_string)
    }
}
