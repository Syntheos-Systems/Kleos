import { useEffect, useState } from 'react';
import { NavLink, Outlet } from 'react-router-dom';
import { currentToken, onUnauthorized } from '$lib/http';
import { SERVICES } from '$lib/services';
import { AuthModal } from './AuthModal';
import { ConnectionDot } from '../ui/ConnectionDot';
import './app.css';

const EXTRA_NAV = [
  { label: 'Memory', route: '/memory' },
  { label: 'Graph', route: '/graph' }
];

// Render the persistent dashboard chrome around route content.
export function AppShell() {
  const [authOpen, setAuthOpen] = useState(() => !currentToken());

  useEffect(() => onUnauthorized(() => setAuthOpen(true)), []);

  const saveApiKey = (value: string) => {
    localStorage.setItem('kleos_api_key', value);
    setAuthOpen(false);
  };

  return (
    <div className="app-shell">
      <aside className="app-shell__rail">
        <div className="app-shell__brand">
          <span>Kleos</span>
          <ConnectionDot />
        </div>
        <button className="app-shell__auth" onClick={() => setAuthOpen(true)} type="button">
          API Key
        </button>
        <nav aria-label="Primary">
          <NavLink className="app-shell__link" end to="/">
            Mission Control
          </NavLink>
          {SERVICES.map((service) => (
            <NavLink className="app-shell__link" key={service.id} to={service.route}>
              {service.label}
            </NavLink>
          ))}
          {EXTRA_NAV.map((item) => (
            <NavLink className="app-shell__link" key={item.route} to={item.route}>
              {item.label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main className="app-shell__main">
        <Outlet />
      </main>
      <AuthModal onClose={() => setAuthOpen(false)} onSave={saveApiKey} open={authOpen} />
    </div>
  );
}
