use std::sync::Arc;

use anyhow::Result;

use crate::core::orchestrator::DictationOrchestrator;

#[cfg(target_os = "windows")]
mod platform {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    use anyhow::Result;
    use once_cell::sync::Lazy;
    use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LMENU, VK_MENU, VK_RMENU};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, SetWindowsHookExW,
        TranslateMessage, UnhookWindowsHookEx, HC_ACTION, MSG, WH_KEYBOARD_LL, WM_KEYUP,
        WM_SYSKEYUP,
    };

    use crate::core::orchestrator::DictationOrchestrator;

    static TAP_SINK: Lazy<Mutex<Option<TapState>>> = Lazy::new(|| Mutex::new(None));

    struct TapState {
        last_alt_up: Option<Instant>,
        saw_non_alt: bool,
        threshold: Duration,
        orchestrator: Arc<DictationOrchestrator>,
    }

    unsafe extern "system" fn keyboard_proc(
        code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if code == HC_ACTION as i32 {
            let event = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            let message = w_param.0 as u32;
            let is_key_up = message == WM_KEYUP || message == WM_SYSKEYUP;
            let is_alt = event.vkCode == VK_MENU.0 as u32
                || event.vkCode == VK_LMENU.0 as u32
                || event.vkCode == VK_RMENU.0 as u32;

            if let Ok(mut state_guard) = TAP_SINK.lock() {
                if let Some(state) = state_guard.as_mut() {
                    if is_key_up && is_alt {
                        let now = Instant::now();
                        if !state.saw_non_alt {
                            if let Some(previous) = state.last_alt_up {
                                if now.duration_since(previous) <= state.threshold {
                                    let orchestrator = state.orchestrator.clone();
                                    tauri::async_runtime::spawn(async move {
                                        if let Err(err) = orchestrator.handle_alt_double_tap().await
                                        {
                                            log::error!("alt double tap handling failed: {err:#}");
                                        }
                                    });
                                    state.last_alt_up = None;
                                    state.saw_non_alt = false;
                                } else {
                                    state.last_alt_up = Some(now);
                                }
                            } else {
                                state.last_alt_up = Some(now);
                            }
                        } else {
                            state.last_alt_up = Some(now);
                            state.saw_non_alt = false;
                        }
                    } else if !is_alt {
                        state.saw_non_alt = true;
                    }
                }
            }
        }
        unsafe { CallNextHookEx(None, code, w_param, l_param) }
    }

    pub fn start_alt_double_tap_listener(
        orchestrator: Arc<DictationOrchestrator>,
        threshold_ms: u64,
    ) -> Result<()> {
        if let Ok(mut guard) = TAP_SINK.lock() {
            *guard = Some(TapState {
                last_alt_up: None,
                saw_non_alt: false,
                threshold: Duration::from_millis(threshold_ms),
                orchestrator,
            });
        }

        thread::spawn(move || {
            let hook = unsafe {
                SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(keyboard_proc),
                    Some(HINSTANCE::default()),
                    0,
                )
            };
            if hook.is_err() {
                log::error!("failed to set keyboard hook");
                return;
            }
            let hook = hook.expect("hook checked");

            let mut msg = MSG::default();
            while unsafe { GetMessageW(&mut msg, None, 0, 0) }.into() {
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }

            if let Err(err) = unsafe { UnhookWindowsHookEx(hook) } {
                log::warn!("failed to unhook keyboard listener: {err:?}");
            }
        });
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use std::sync::Arc;

    use anyhow::Result;

    use crate::core::orchestrator::DictationOrchestrator;

    pub fn start_alt_double_tap_listener(
        _orchestrator: Arc<DictationOrchestrator>,
        _threshold_ms: u64,
    ) -> Result<()> {
        log::warn!("low-level alt listener is inactive outside windows");
        Ok(())
    }
}

pub fn start_alt_double_tap_listener(
    orchestrator: Arc<DictationOrchestrator>,
    threshold_ms: u64,
) -> Result<()> {
    platform::start_alt_double_tap_listener(orchestrator, threshold_ms)
}
