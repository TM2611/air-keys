import { useEffect, useMemo, useState } from "react"
import { listen } from '@tauri-apps/api/event'

type RecordingAmplitudePayload = {
    level: number
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

    useEffect(() => {
        let mounted = true
        const attachListener = async () => {
            const unlisten = await listen<RecordingAmplitudePayload>(
                'recording-amplitude',
                (event) => {
                    if (!mounted) {
                        return
                    }
                    setTargetLevel(clampLevel(event.payload.level))
                },
            )
            return unlisten
        }

        let detachListener: (() => void) | undefined
        void attachListener().then((unlisten) => {
            detachListener = unlisten
        }).catch(() => {
            // Running outside Tauri (e.g. plain browser preview) is non-fatal.
        })

        return () => {
            mounted = false
            detachListener?.()
        }
    }, [])

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
        return Array.from({ length: BAR_COUNT }, (_, index) => {
            const offset = (index / BAR_COUNT) * Math.PI * 1.5
            const ripple = Math.sin(phase + offset) * 0.16
            const normalized = Math.max(0.08, Math.min(1, displayLevel + ripple))
            return normalized
        })
    }, [displayLevel, phase])

    return (
        <main className="recording-shell" data-tauri-drag-region>
            <span className="recording-label" data-tauri-drag-region>
                Listening
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