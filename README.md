# Air Keys

Air Keys is a Windows tray dictation app that records your voice, transcribes with Deepgram, and pastes the result at your cursor.

## Installation (v1)

1. Install dependencies:
   - `npm install`
2. Build the desktop app:
   - `npm run tauri:build`
3. Run the generated installer:
   - `src-tauri/target/release/bundle/msi/Air Keys_0.1.0_x64_en-US.msi`
   - (or the `.exe` installer in the same bundle folder)
4. Launch **Air Keys** from the Windows Start Menu.

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

## Development commands

- `npm run dev` - run frontend only.
- `npm run build` - build the frontend bundle.
- `npm run tauri:dev` - run Tauri desktop app in development mode.
- `npm run tauri:build` - build release binaries and installers.

## Notes

- Windows-first runtime behavior.
- Audio is sent to Deepgram for transcription.
- API keys are stored locally for this app.
