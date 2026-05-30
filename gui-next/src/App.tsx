import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import { RealtimeProvider } from '$lib/realtime';
import { SERVICES } from '$lib/services';
import { AppShell } from './app/AppShell';
import { Overview } from './routes/Overview';
import { PlaceholderPage } from './routes/PlaceholderPage';

// Render the Kleos dashboard providers, router, and top-level routes.
export default function App() {
  return (
    <RealtimeProvider>
      <BrowserRouter future={{ v7_relativeSplatPath: true, v7_startTransition: true }}>
        <Routes>
          <Route element={<AppShell />}>
            <Route index element={<Overview />} />
            {SERVICES.map((service) => (
              <Route
                key={service.id}
                path={service.route.slice(1)}
                element={<PlaceholderPage title={service.label} />}
              />
            ))}
            <Route path="memory/*" element={<PlaceholderPage phase="Phase 3" title="Memory" />} />
            <Route path="graph" element={<PlaceholderPage phase="Phase 4" title="Graph" />} />
            <Route path="*" element={<Navigate replace to="/" />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </RealtimeProvider>
  );
}
