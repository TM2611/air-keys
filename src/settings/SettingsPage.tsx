import { useEffect, useMemo, useState } from 'react'
import type { FormEvent } from 'react'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'

type SaveState = 'idle' | 'validating' | 'saving' | 'saved' | 'error'

const API_KEY_MASK = '••••••••••••'

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

function isInvalidApiKeyError(message: string): boolean {
    return /(invalid|unauthorized|forbidden|401|403)/i.test(message)
}

export function SettingsPage() {
    const [deepgramApiKey, setDeepgramApiKey] = useState('')
    const [geminiApiKey, setGeminiApiKey] = useState('')

    const hasDeepgramKey = deepgramApiKey === API_KEY_MASK
    const hasGeminiKey = geminiApiKey === API_KEY_MASK

    const [processingEnabled, setProcessingEnabled] = useState(false)
    const [appVersion, setAppVersion] = useState('...')
    const [launchOnStartupEnabled, setLaunchOnStartupEnabled] = useState(false)
    const [isCheckingDeepgramStatus, setIsCheckingDeepgramStatus] = useState(true)
    const [isCheckingGeminiStatus, setIsCheckingGeminiStatus] = useState(true)
    const [isCheckingProcessingEnabled, setIsCheckingProcessingEnabled] = useState(true)
    const [isCheckingLaunchOnStartup, setIsCheckingLaunchOnStartup] = useState(true)
    const [isSavingLaunchOnStartup, setIsSavingLaunchOnStartup] = useState(false)
    const [loggingEnabled, setLoggingEnabled] = useState(false)
    const [isCheckingLoggingEnabled, setIsCheckingLoggingEnabled] = useState(true)
    const [isSavingLoggingEnabled, setIsSavingLoggingEnabled] = useState(false)
    const [isSavingProcessingEnabled, setIsSavingProcessingEnabled] = useState(false)
    const [deepgramSaveState, setDeepgramSaveState] = useState<SaveState>('idle')
    const [geminiSaveState, setGeminiSaveState] = useState<SaveState>('idle')
    const [errorMessage, setErrorMessage] = useState('')

    const refreshDeepgramKeyStatus = async () => {
        setIsCheckingDeepgramStatus(true)
        try {
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            const status = await invoke<boolean>('has_deepgram_api_key')
            setDeepgramApiKey(status ? API_KEY_MASK : '')
        } finally {
            setIsCheckingDeepgramStatus(false)
        }
    }

    const refreshLaunchOnStartup = async () => {
        setIsCheckingLaunchOnStartup(true)
        try {
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            const enabled = await invoke<boolean>('get_launch_on_startup_enabled')
            setLaunchOnStartupEnabled(enabled)
        } finally {
            setIsCheckingLaunchOnStartup(false)
        }
    }

    const refreshGeminiKeyStatus = async () => {
        setIsCheckingGeminiStatus(true)
        try {
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            const status = await invoke<boolean>('has_gemini_api_key')
            setGeminiApiKey(status ? API_KEY_MASK : '')
        } finally {
            setIsCheckingGeminiStatus(false)
        }
    }

    const refreshProcessingEnabled = async () => {
        setIsCheckingProcessingEnabled(true)
        try {
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            const enabled = await invoke<boolean>('get_processing_enabled')
            setProcessingEnabled(enabled)
        } finally {
            setIsCheckingProcessingEnabled(false)
        }
    }

    const refreshLoggingEnabled = async () => {
        setIsCheckingLoggingEnabled(true)
        try {
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            const enabled = await invoke<boolean>('get_logging_enabled')
            setLoggingEnabled(enabled)
        } finally {
            setIsCheckingLoggingEnabled(false)
        }
    }

    useEffect(() => {
        const timer = window.setTimeout(() => {
            void Promise.all([
                refreshDeepgramKeyStatus(),
                refreshLaunchOnStartup(),
                refreshGeminiKeyStatus(),
                refreshProcessingEnabled(),
                refreshLoggingEnabled(),
                getVersion().then((version) => setAppVersion(version)),
            ]).catch((error) => {
                setDeepgramSaveState('error')
                setErrorMessage(parseInvokeError(error))
            })
        }, 0)
        return () => window.clearTimeout(timer)
    }, [])

    const deepgramSaveButtonText = useMemo(() => {
        switch (deepgramSaveState) {
            case 'validating':
                return 'Validating...'
            case 'saving':
                return 'Saving...'
            case 'saved':
                return 'Saved'
            default:
                return 'Save key'
        }
    }, [deepgramSaveState])

    const geminiSaveButtonText = useMemo(() => {
        switch (geminiSaveState) {
            case 'validating':
                return 'Validating...'
            case 'saving':
                return 'Saving...'
            case 'saved':
                return 'Saved'
            default:
                return 'Save key'
        }
    }, [geminiSaveState])

    async function onSaveDeepgram(event: FormEvent) {
        event.preventDefault()
        if (!deepgramApiKey.trim()) {
            setDeepgramSaveState('error')
            setErrorMessage('Enter a Deepgram API key before saving.')
            return
        }

        try {
            setDeepgramSaveState('validating')
            setErrorMessage('')
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            await invoke('save_deepgram_api_key', { apiKey: deepgramApiKey })
            setDeepgramApiKey(API_KEY_MASK)
            setDeepgramSaveState('saved')
        } catch (error) {
            setDeepgramSaveState('error')
            const parsedError = parseInvokeError(error)
            if (isInvalidApiKeyError(parsedError)) {
                setErrorMessage('Invalid Deepgram API key. Enter a valid API key and try again.')
                return
            }
            setErrorMessage(`Air Keys could not save the API key: ${parsedError}`)
        }
    }

    async function onClearDeepgram() {
        try {
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            await invoke('clear_deepgram_api_key')
            setDeepgramApiKey('')
            setDeepgramSaveState('idle')
            setErrorMessage('')
        } catch (error) {
            setDeepgramSaveState('error')
            setErrorMessage(`Air Keys could not clear the API key: ${parseInvokeError(error)}`)
        }
    }

    async function onSaveGemini(event: FormEvent) {
        event.preventDefault()
        if (!geminiApiKey.trim()) {
            setGeminiSaveState('error')
            setErrorMessage('Enter a Gemini API key before saving.')
            return
        }

        try {
            setGeminiSaveState('validating')
            setErrorMessage('')
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            await invoke('save_gemini_api_key', { apiKey: geminiApiKey })
            setGeminiApiKey(API_KEY_MASK)
            setGeminiSaveState('saved')
        } catch (error) {
            setGeminiSaveState('error')
            const parsedError = parseInvokeError(error)
            if (isInvalidApiKeyError(parsedError)) {
                setErrorMessage('Invalid Gemini API key. Enter a valid API key and try again.')
                return
            }
            setErrorMessage(`Air Keys could not save the API key: ${parsedError}`)
        }
    }

    async function onClearGemini() {
        try {
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            await invoke('clear_gemini_api_key')
            setGeminiApiKey('')
            setGeminiSaveState('idle')
            setErrorMessage('')
        } catch (error) {
            setGeminiSaveState('error')
            setErrorMessage(`Air Keys could not clear the API key: ${parseInvokeError(error)}`)
        }
    }

    async function onLaunchOnStartupChange(enabled: boolean) {
        try {
            setIsSavingLaunchOnStartup(true)
            setErrorMessage('')
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            await invoke('set_launch_on_startup_enabled', { enabled })
            setLaunchOnStartupEnabled(enabled)
        } catch (error) {
            setDeepgramSaveState('error')
            setErrorMessage(
                `Air Keys could not update launch on startup: ${parseInvokeError(error)}`
            )
        } finally {
            setIsSavingLaunchOnStartup(false)
        }
    }

    async function onProcessingEnabledChange(enabled: boolean) {
        try {
            setIsSavingProcessingEnabled(true)
            setErrorMessage('')
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            await invoke('set_processing_enabled', { enabled })
            setProcessingEnabled(enabled)
        } catch (error) {
            setDeepgramSaveState('error')
            setErrorMessage(`Air Keys could not update processing: ${parseInvokeError(error)}`)
        } finally {
            setIsSavingProcessingEnabled(false)
        }
    }

    async function onLoggingEnabledChange(enabled: boolean) {
        try {
            setIsSavingLoggingEnabled(true)
            setErrorMessage('')
            if (!hasTauriInvoke()) {
                throw new Error('Tauri runtime unavailable. Open this UI from the Air Keys tray app.')
            }
            await invoke('set_logging_enabled', { enabled })
            setLoggingEnabled(enabled)
        } catch (error) {
            setErrorMessage(`Air Keys could not update logging: ${parseInvokeError(error)}`)
        } finally {
            setIsSavingLoggingEnabled(false)
        }
    }

    return (
        <main className="settings-shell">
            <h1>Air Keys settings</h1>
            <p className="settings-subtitle">Press Alt twice to start/stop dictation. <br />Hold Alt to cancel in progress recording.</p>
            <br />
            <p className="settings-help">
                Air Keys runs from the system tray. Configure your transcription and post-processing
                keys here.
            </p>

            <section className="settings-section">
                <form className="settings-form" onSubmit={onSaveDeepgram}>
                    <label htmlFor="deepgramKey">Deepgram API key</label>
                    <input
                        id="deepgramKey"
                        type="password"
                        autoComplete="off"
                        spellCheck={false}
                        value={deepgramApiKey}
                        readOnly={hasDeepgramKey}
                        onChange={(event) => {
                            setDeepgramApiKey(event.target.value)
                            if (deepgramSaveState !== 'idle') {
                                setDeepgramSaveState('idle')
                            }
                        }}
                        placeholder="dg_live_..."
                    />
                    <div className="settings-actions">
                        <button
                            type="submit"
                            disabled={
                                deepgramSaveState === 'validating' ||
                                deepgramSaveState === 'saving' ||
                                hasDeepgramKey
                            }
                        >
                            {deepgramSaveButtonText}
                        </button>
                        <button type="button" onClick={onClearDeepgram}>
                            Clear key
                        </button>
                    </div>
                </form>
                <p className="settings-status">
                    Stored key:{' '}
                    <strong>
                        {isCheckingDeepgramStatus ? 'checking...' : hasDeepgramKey ? 'present' : 'not set'}
                    </strong>
                </p>
            </section>

            <section className="settings-section">
                <h2>General</h2>
                <label className="settings-checkbox" htmlFor="launchOnStartup">
                    <input
                        id="launchOnStartup"
                        type="checkbox"
                        checked={launchOnStartupEnabled}
                        disabled={isCheckingLaunchOnStartup || isSavingLaunchOnStartup}
                        onChange={(event) => {
                            void onLaunchOnStartupChange(event.target.checked)
                        }}
                    />
                    Launch on startup
                </label>
                <p className="settings-muted">
                    {isCheckingLaunchOnStartup
                        ? 'Checking startup setting...'
                        : 'When enabled, Air Keys starts when you sign in to Windows.'}
                </p>
                <label className="settings-checkbox settings-checkbox-spaced" htmlFor="loggingEnabled">
                    <input
                        id="loggingEnabled"
                        type="checkbox"
                        checked={loggingEnabled}
                        disabled={isCheckingLoggingEnabled || isSavingLoggingEnabled}
                        onChange={(event) => {
                            void onLoggingEnabledChange(event.target.checked)
                        }}
                    />
                    Store logs to file
                </label>
                <p className="settings-muted">
                    {isCheckingLoggingEnabled
                        ? 'Checking logging setting...'
                        : 'When enabled, logs are saved to disk (does not include audio or transcripts). Takes effect on next launch.'}
                </p>
            </section>

            <section className="settings-section">
                <h2>Post-processing</h2>
                <label className="settings-checkbox" htmlFor="processingEnabled">
                    <input
                        id="processingEnabled"
                        type="checkbox"
                        checked={processingEnabled}
                        disabled={isCheckingProcessingEnabled || isSavingProcessingEnabled}
                        onChange={(event) => {
                            void onProcessingEnabledChange(event.target.checked)
                        }}
                    />
                    Clean up transcripts with AI
                </label>
                <p className="settings-muted">
                    {isCheckingProcessingEnabled
                        ? 'Checking post-processing setting...'
                        : 'When enabled, Air Keys uses Gemini to remove fillers and smooth phrasing.'}
                </p>
                {processingEnabled ? (
                    <>
                        <form className="settings-form settings-inline-form" onSubmit={onSaveGemini}>
                            <label htmlFor="geminiKey">Gemini API key</label>
                            <input
                                id="geminiKey"
                                type="password"
                                autoComplete="off"
                                spellCheck={false}
                                value={geminiApiKey}
                                readOnly={hasGeminiKey}
                                onChange={(event) => {
                                    setGeminiApiKey(event.target.value)
                                    if (geminiSaveState !== 'idle') {
                                        setGeminiSaveState('idle')
                                    }
                                }}
                                placeholder="AIza..."
                            />
                            <div className="settings-actions">
                                <button
                                    type="submit"
                                    disabled={
                                        geminiSaveState === 'validating' ||
                                        geminiSaveState === 'saving' ||
                                        hasGeminiKey
                                    }
                                >
                                    {geminiSaveButtonText}
                                </button>
                                <button type="button" onClick={onClearGemini}>
                                    Clear key
                                </button>
                            </div>
                        </form>
                        <p className="settings-status">
                            Stored key:{' '}
                            <strong>
                                {isCheckingGeminiStatus ? 'checking...' : hasGeminiKey ? 'present' : 'not set'}
                            </strong>
                        </p>
                    </>
                ) : null}
            </section>

            {errorMessage ? <p className="settings-error">{errorMessage}</p> : null}
            <p className="settings-footer">Air Keys v{appVersion}</p>
        </main>
    )
}
