import { useEffect, useMemo, useState } from 'react'
import type { FormEvent } from 'react'
import { invoke } from '@tauri-apps/api/core'

type SaveState = 'idle' | 'saving' | 'saved' | 'error'

export function SettingsPage() {
  const [apiKey, setApiKey] = useState('')
  const [hasKey, setHasKey] = useState(false)
  const [saveState, setSaveState] = useState<SaveState>('idle')
  const [errorMessage, setErrorMessage] = useState('')

  const refreshKeyStatus = async () => {
    const status = await invoke<boolean>('has_deepgram_api_key')
    setHasKey(status)
  }

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void refreshKeyStatus()
    }, 0)
    return () => window.clearTimeout(timer)
  }, [])

  const saveButtonText = useMemo(() => {
    switch (saveState) {
      case 'saving':
        return 'Saving...'
      case 'saved':
        return 'Saved'
      default:
        return 'Save key'
    }
  }, [saveState])

  async function onSave(event: FormEvent) {
    event.preventDefault()
    if (!apiKey.trim()) {
      setSaveState('error')
      setErrorMessage('Enter a Deepgram API key before saving.')
      return
    }

    try {
      setSaveState('saving')
      setErrorMessage('')
      await invoke('save_deepgram_api_key', { apiKey })
      setApiKey('')
      await refreshKeyStatus()
      setSaveState('saved')
    } catch {
      setSaveState('error')
      setErrorMessage('Air Keys could not save the API key.')
    }
  }

  async function onClear() {
    await invoke('clear_deepgram_api_key')
    setApiKey('')
    setSaveState('idle')
    setErrorMessage('')
    await refreshKeyStatus()
  }

  return (
    <main className="settings-shell">
      <h1>Air Keys settings</h1>
      <p className="settings-help">
        Air Keys runs from the system tray. Set your Deepgram API key here.
      </p>
      <form className="settings-form" onSubmit={onSave}>
        <label htmlFor="deepgramKey">Deepgram API key</label>
        <input
          id="deepgramKey"
          type="password"
          autoComplete="off"
          spellCheck={false}
          value={apiKey}
          onChange={(event) => {
            setApiKey(event.target.value)
            if (saveState !== 'idle') {
              setSaveState('idle')
            }
          }}
          placeholder="dg_live_..."
        />
        <div className="settings-actions">
          <button type="submit" disabled={saveState === 'saving'}>
            {saveButtonText}
          </button>
          <button type="button" onClick={onClear}>
            Clear key
          </button>
        </div>
      </form>
      <p className="settings-status">
        Stored key: <strong>{hasKey ? 'present' : 'not set'}</strong>
      </p>
      {errorMessage ? <p className="settings-error">{errorMessage}</p> : null}
    </main>
  )
}
