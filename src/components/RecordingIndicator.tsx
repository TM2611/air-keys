import { useEffect, useMemo, useState } from "react"
import { listen } from '@tauri-apps/api/event'

type RecordingAmplitudePayload = {
    level: number
}

type RecordingStatePayload = {
    state: 'listening' | 'processing' | 'cancelling'
}

const BAR_COUNT = 9

function clampLevel(level: number): number {
    if (!Number.isFinite(level)) {
        return 0
    }
    return Math.max(0, Math.min(1, level))
}

function toLevelClass(level: number): string {
    const bucket = Math.round(clampLevel(level) * 10)
    return `level-${bucket}`
}


export default function RecordingIndicator() {
    const [targetLevel, setTargetLevel] = useState(0)
    const [displayLevel, setDisplayLevel] = useState(0)
    const [phase, setPhase] = useState(0)
    const [state, setState] = useState<'listening' | 'processing' | 'cancelling'>('listening')

    useEffect(() => {
        let mounted = true
        const attachListeners = async () => {
            const unlistenAmplitude = await listen<RecordingAmplitudePayload>(
                'recording-amplitude',
                (event) => {
                    if (!mounted) {
                        return
                    }
                    if (state === 'processing' || state === 'cancelling') {
                        return
                    }
                    setTargetLevel(clampLevel(event.payload.level))
                },
            )
            const unlistenState = await listen<RecordingStatePayload>('recording-state', (event) => {
                if (!mounted) {
                    return
                }
                if (event.payload.state === 'processing') {
                    setState('processing')
                    setTargetLevel(0)
                    return
                }
                if (event.payload.state === 'cancelling') {
                    setState('cancelling')
                    setTargetLevel(0)
                    return
                }
                setState('listening')
            })
            return () => {
                unlistenAmplitude()
                unlistenState()
            }
        }

        let detachListener: (() => void) | undefined
        void attachListeners().then((unlisten) => {
            detachListener = unlisten
        }).catch(() => {
            // Running outside Tauri (e.g. plain browser preview) is non-fatal.
        })

        return () => {
            mounted = false
            detachListener?.()
        }
    }, [state])

    useEffect(() => {
        let frame = 0
        const step = () => {
            setDisplayLevel((prev) => prev + (targetLevel - prev) * 0.3)
            setPhase((prev) => prev + 0.14)
            frame = window.requestAnimationFrame(step)
        }
        frame = window.requestAnimationFrame(step)
        return () => window.cancelAnimationFrame(frame)
    }, [targetLevel])

    const bars = useMemo(() => {
        if (state === 'processing' || state === 'cancelling') {
            return Array.from({ length: BAR_COUNT }, (_, index) => {
                const offset = (index / BAR_COUNT) * Math.PI * 1.5
                const pulse = (Math.sin(phase + offset) + 1) * 0.12
                return 0.2 + pulse
            })
        }
        return Array.from({ length: BAR_COUNT }, (_, index) => {
            const offset = (index / BAR_COUNT) * Math.PI * 1.5
            const ripple = Math.sin(phase + offset) * 0.16
            const normalized = Math.max(0.08, Math.min(1, displayLevel + ripple))
            return normalized
        })
    }, [displayLevel, phase, state])

    return (
        <main
            className={`recording-shell ${state === 'processing' || state === 'cancelling' ? 'recording-shell-processing' : ''}`}
            data-tauri-drag-region
        >
            <span className="recording-label" data-tauri-drag-region>
                {state === 'processing' ? 'Processing' : state === 'cancelling' ? 'Cancelling' : 'Listening'}
            </span>
            <div className="wave-bars" data-tauri-drag-region>
                {bars.map((barLevel, index) => (
                    <span
                        key={index}
                        className={`wave-bar ${toLevelClass(barLevel)}`}
                    />
                ))}
            </div>
        </main>
    )
}