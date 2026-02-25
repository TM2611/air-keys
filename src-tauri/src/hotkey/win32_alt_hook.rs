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
        TranslateMessage, UnhookWindowsHookEx, HC_ACTION, MSG, WH_KEYBOARD_LL, WM_KEYDOWN,
        WM_SYSKEYDOWN,
    };

    use crate::core::orchestrator::DictationOrchestrator;

    static TAP_SINK: Lazy<Mutex<Option<TapState>>> = Lazy::new(|| Mutex::new(None));

    /// Minimum hold duration to trigger "hold Alt to cancel" (ms).
    const HOLD_CANCEL_MS: u64 = 400;

    struct TapState {
        last_alt_up: Option<Instant>,
        last_alt_down: Option<Instant>,
        alt_is_down: bool,
        hold_seq: u64,
        hold_consumed: bool,
        saw_non_alt: bool,
        threshold: Duration,
        hold_cancel: Duration,
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
            let is_alt = event.vkCode == VK_MENU.0 as u32
                || event.vkCode == VK_LMENU.0 as u32
                || event.vkCode == VK_RMENU.0 as u32;

            if let Ok(mut state_guard) = TAP_SINK.lock() {
                if let Some(state) = state_guard.as_mut() {
                    if is_alt {
                        let now = Instant::now();
                        let is_key_down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
                        if is_key_down {
                            // Ignore keyboard auto-repeat while Alt is held.
                            if !state.alt_is_down {
                                state.alt_is_down = true;
                                state.last_alt_down = Some(now);
                                state.hold_consumed = false;
                                state.hold_seq = state.hold_seq.saturating_add(1);

                                let seq = state.hold_seq;
                                let hold_cancel = state.hold_cancel;
                                let orchestrator = state.orchestrator.clone();
                                tauri::async_runtime::spawn(async move {
                                    tokio::time::sleep(hold_cancel).await;

                                    let should_cancel = if let Ok(mut guard) = TAP_SINK.lock() {
                                        if let Some(state) = guard.as_mut() {
                                            if state.alt_is_down && state.hold_seq == seq {
                                                state.hold_consumed = true;
                                                state.last_alt_up = None;
                                                state.saw_non_alt = false;
                                                true
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    };

                                    if should_cancel {
                                        if let Err(err) = orchestrator.cancel_recording().await {
                                            log::error!("hold-alt cancel failed: {err:#}");
                                        }
                                    }
                                });
                            }
                        } else {
                            // Alt key up (WM_KEYUP or WM_SYSKEYUP)
                            state.alt_is_down = false;
                            state.last_alt_down = None;

                            if state.hold_consumed {
                                state.hold_consumed = false;
                                state.last_alt_up = None;
                                state.saw_non_alt = false;
                            } else if !state.saw_non_alt {
                                if let Some(previous) = state.last_alt_up {
                                    if now.duration_since(previous) <= state.threshold {
                                        let orchestrator = state.orchestrator.clone();
                                        tauri::async_runtime::spawn(async move {
                                            if let Err(err) =
                                                orchestrator.handle_alt_double_tap().await
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
                        }
                    } else {
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
                last_alt_down: None,
                alt_is_down: false,
                hold_seq: 0,
                hold_consumed: false,
                saw_non_alt: false,
                threshold: Duration::from_millis(threshold_ms),
                hold_cancel: Duration::from_millis(HOLD_CANCEL_MS),
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
