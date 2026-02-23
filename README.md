# Air Keys

Air Keys is a Windows-first tray dictation application built with Tauri v2 (Rust backend) and React 19 (TypeScript frontend).

## Current scaffold

- Tray-first runtime with hidden Settings window.
- Low-level Alt double-tap listener boundary (`SetWindowsHookExW` on Windows).
- `cpal` microphone recorder writing temporary WAV output.
- `AudioProcessor` abstraction with a `DeepgramProcessor` (Nova-2, smart formatting).
- Clipboard injection pipeline using `arboard` + `enigo` with clipboard restore.
- Stronghold-backed key storage boundary via `tauri-plugin-stronghold`.

## Commands

- `npm run dev` - run frontend only.
- `npm run build` - build the frontend bundle.
- `npm run tauri:dev` - run Tauri desktop app.
- `npm run tauri:build` - build desktop binaries.

## Notes

- This scaffold targets Windows runtime behaviour.
- API keys are never hardcoded and are not logged in plaintext.
