import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import './recording.css'
import RecordingIndicator from './components/RecordingIndicator'

createRoot(document.getElementById('recording-root')!).render(
  <StrictMode>
    <RecordingIndicator />
  </StrictMode>,
)
