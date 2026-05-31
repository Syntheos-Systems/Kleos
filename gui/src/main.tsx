import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
// Self-hosted brand fonts (no external CDN, so the strict CSP and VPN-only
// deployments still get the real typography). Weights match the redesign.
import '@fontsource/space-grotesk/400.css';
import '@fontsource/space-grotesk/500.css';
import '@fontsource/space-grotesk/600.css';
import '@fontsource/space-grotesk/700.css';
import '@fontsource/jetbrains-mono/400.css';
import '@fontsource/jetbrains-mono/500.css';
import '@fontsource/jetbrains-mono/600.css';
import App from './App';
import './design/tokens.css';

// Mount the Kleos GUI as a React single-page application.
createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
