import { useEffect, useMemo, useState } from 'react'
import type { FormEvent } from 'react'
import { invoke } from '@tauri-apps/api/core'

type SaveState = 'idle' | 'saving' | 'saved' | 'error'

function hasTauriInvoke(): boolean {
    if (typeof window === 'undefined') {
        return false
    }
    const tauriInternals = (window as Window & { __TAURI_INTERNALS__?: { invoke?: unknown } })
        .__TAURI_INTERNALS__
    return typeof tauriInternals?.invoke === 'function'
}

function parseInvokeError(error: unknown): string {
    if (error instanceof Error && error.message) {
        return error.message
    }
    if (typeof error === 'string' && error.length > 0) {
        return error
    }
    return 'Unknown error'
}

export function SettingsPage() {
    const [apiKey, setApiKey] = useState('')
    const [hasKey, setHasKey] = useState(false)
    const [isCheckingStatus, setIsCheckingStatus] = useState(true)
    const [saveState, setSaveState] = useState<SaveState>('idle')
    const [errorMessage, setErrorMessage] = useState('')

    const refreshKeyStatus = async () => {
        setIsCheckingStatus(true)
        try {
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            const status = await invoke<boolean>('has_deepgram_api_key')
            setHasKey(status)
        } finally {
            setIsCheckingStatus(false)
        }
    }

    useEffect(() => {
        const timer = window.setTimeout(() => {
            void refreshKeyStatus().catch((error) => {
                setSaveState('error')
                setErrorMessage(parseInvokeError(error))
            })
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
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            await invoke('save_deepgram_api_key', { apiKey })
            setApiKey('')
            await refreshKeyStatus()
            setSaveState('saved')
        } catch (error) {
            setSaveState('error')
            setErrorMessage(`Air Keys could not save the API key: ${parseInvokeError(error)}`)
        }
    }

    async function onClear() {
        try {
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            await invoke('clear_deepgram_api_key')
            setApiKey('')
            setSaveState('idle')
            setErrorMessage('')
            await refreshKeyStatus()
        } catch (error) {
            setSaveState('error')
            setErrorMessage(`Air Keys could not clear the API key: ${parseInvokeError(error)}`)
        }
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
                Stored key:{' '}
                <strong>{isCheckingStatus ? 'checking...' : hasKey ? 'present' : 'not set'}</strong>
            </p>
            {errorMessage ? <p className="settings-error">{errorMessage}</p> : null}
        </main>
    )
}
