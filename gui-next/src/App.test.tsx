import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';
import type { ReactNode } from 'react';

const liveStats = vi.hoisted(() => ({
  axon: { by_channel: [], channels: 5, sources: 3, total_events: 88 },
  broca: { agents: 7, by_action: [], by_agent: [], by_service: [], services: 6, total_actions: 120 },
  chiasm: { by_status: { active: 3, completed: 39 }, total: 42 },
  loom: { active_runs: 2, runs: 9, runs_by_status: [], steps: 30, workflows: 4 },
  soma: { by_status: [], by_type: [], online_agents: 5, total_agents: 8, types: 3 },
  thymus: { agent_count: 4, by_rubric: [], evaluations: 11, metrics: 2, rubrics: 6 }
}));

vi.mock('$lib/realtime', () => ({
  RealtimeProvider: ({ children }: { children: ReactNode }) => <>{children}</>,
  useLive: (_key: unknown, _fetcher: unknown, channel: keyof typeof liveStats) => ({
    data: liveStats[channel],
    isError: false,
    isLoading: false
  }),
  useStreamStatus: () => 'live'
}));

describe('App shell', () => {
  beforeEach(() => {
    window.history.pushState({}, '', '/');
  });

  it('renders mission control with service navigation and live stats', async () => {
    render(<App />);

    expect(screen.getByRole('heading', { name: 'Mission Control' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Chiasm' })).toHaveAttribute('href', '/chiasm');
    expect(screen.getByRole('link', { name: 'Memory' })).toHaveAttribute('href', '/memory');
    expect(screen.getAllByText('live').length).toBeGreaterThan(0);
    expect(screen.getByText('42')).toBeInTheDocument();
    expect(screen.getByText('120')).toBeInTheDocument();
    expect(screen.getByText('88')).toBeInTheDocument();
  });

  it('keeps service routes reachable before service-specific pages land', () => {
    window.history.pushState({}, '', '/axon');

    render(<App />);

    expect(screen.getByRole('heading', { name: 'Axon' })).toBeInTheDocument();
    expect(screen.getByText('Service view pending Phase 2.')).toBeInTheDocument();
  });
});
