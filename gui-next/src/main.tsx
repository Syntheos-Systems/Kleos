import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import './design/tokens.css';

// Mount the Kleos GUI as a React single-page application.
createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
