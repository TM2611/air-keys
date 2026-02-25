# Air Keys

Air Keys is a Windows tray dictation app that records your voice, transcribes with Deepgram, and pastes the result at your cursor.

## Quick start

1. Download the latest installer from [GitHub Releases](https://github.com/TM2611/air-keys/releases).
2. Run the MSI or EXE installer.
3. Launch **Air Keys** from the Windows Start Menu.

No Node.js, Rust, or build steps required.

## First-time setup

1. Start Air Keys from Start Menu (it runs in the system tray).
2. Right-click the tray icon and choose **Settings**.
3. Enter your Deepgram API key (`dg_live_...`) and click **Save key**.
4. (Optional) Enable **Launch on startup**.

## How to use

1. Place your cursor where text should be inserted.
2. Double-tap **Alt** to start recording.
3. Speak.
4. Double-tap **Alt** again to stop recording.
5. Air Keys transcribes audio and pastes text at your current cursor location.

## Settings

- Deepgram API key save/clear
- Stored key status
- Launch on startup toggle
- Shortcut hint and app version display

## Building from source

For contributors who want to build locally:

1. Install dependencies: `npm install`
2. Build the desktop app: `npm run tauri:build`
3. Run the generated installer from `src-tauri/target/release/bundle/msi/` or `bundle/nsis/`
4. Or run in development mode: `npm run tauri:dev`

**Development commands:**

- `npm run dev` - run frontend only
- `npm run build` - build the frontend bundle
- `npm run tauri:dev` - run Tauri desktop app in development mode
- `npm run tauri:build` - build release binaries and installers

## Notes

- Windows-first runtime behavior.
- Audio is sent to Deepgram for transcription.
- API keys are stored locally for this app.
