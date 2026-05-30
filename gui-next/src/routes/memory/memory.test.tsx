import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Memory } from './Memory';

const graphRuntime = vi.hoisted(() => {
  const linkForce: { distance: ReturnType<typeof vi.fn>; strength: ReturnType<typeof vi.fn> } = {
    distance: vi.fn(),
    strength: vi.fn()
  };
  const chargeForce: { distanceMax: ReturnType<typeof vi.fn>; strength: ReturnType<typeof vi.fn> } = {
    distanceMax: vi.fn(),
    strength: vi.fn()
  };
  const centerForce: { strength: ReturnType<typeof vi.fn> } = {
    strength: vi.fn()
  };
  linkForce.distance.mockImplementation(() => linkForce);
  linkForce.strength.mockImplementation(() => linkForce);
  chargeForce.distanceMax.mockImplementation(() => chargeForce);
  chargeForce.strength.mockImplementation(() => chargeForce);
  centerForce.strength.mockImplementation(() => centerForce);

  return {
    centerForce,
    chargeForce,
    d3Force: vi.fn((name: string) => {
      if (name === 'link') return linkForce;
      if (name === 'charge') return chargeForce;
      if (name === 'center') return centerForce;
      return undefined;
    }),
    linkForce,
    reheat: vi.fn()
  };
});

const memoryData = vi.hoisted(() => ({
  entities: [
    {
      confidence: 0.9,
      created_at: '',
      description: 'Primary project',
      entity_type: 'project',
      first_seen_at: '',
      id: 1,
      last_seen_at: '2026-05-30 12:00:00',
      name: 'Kleos',
      occurrence_count: 7
    }
  ],
  inbox: [{ content: 'pending memory candidate' }],
  projects: [
    { created_at: '', description: '', id: 1, memory_count: 12, name: 'gui rebuild', status: 'active', updated_at: '' }
  ],
  graph: {
    edge_count: 1,
    edges: [{ source: '1', target: '2', type: 'association', weight: 0.77 }],
    node_count: 2,
    nodes: [
      { category: 'progress', content: 'A', created_at: '', id: '1', importance: 4, is_static: false, label: 'A', size: 8, source: 'test' },
      { category: 'decision', content: 'B', created_at: '', id: '2', importance: 3, is_static: false, label: 'B', size: 6, source: 'test' }
    ]
  },
  results: [
    {
      category: 'progress',
      content: 'stored fact about the GUI',
      created_at: '2026-05-30 12:00:00',
      id: 1,
      importance: 4,
      score: 0.82,
      search_type: 'hybrid',
      source: 'codex',
      tags: ['gui']
    }
  ]
}));

vi.mock('@tanstack/react-query', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@tanstack/react-query')>();
  return {
    ...actual,
    useMutation: () => ({ data: memoryData.results, isPending: false, mutate: vi.fn() }),
    useQuery: ({ queryKey }: { queryKey: readonly unknown[] }) => {
      const joined = queryKey.join(':');
      if (joined === 'mem:list') return { data: memoryData.results, isLoading: false };
      if (joined === 'mem:inbox') return { data: memoryData.inbox, isLoading: false };
      if (joined === 'mem:entities') return { data: memoryData.entities, isLoading: false };
      if (joined === 'mem:projects') return { data: memoryData.projects, isLoading: false };
      if (joined === 'mem:graph') return { data: memoryData.graph, isLoading: false };
      return { data: undefined, isLoading: false };
    }
  };
});

vi.mock('react-force-graph-3d', async () => {
  const React = await import('react');
  const MockForceGraph = React.forwardRef<unknown, { height: number; width: number }>((props, ref) => {
    React.useImperativeHandle(ref, () => ({
      d3Force: graphRuntime.d3Force,
      d3ReheatSimulation: graphRuntime.reheat
    }));
    return React.createElement('div', {
      'data-height': props.height,
      'data-testid': 'graph-canvas',
      'data-width': props.width
    });
  });
  return { default: MockForceGraph };
});

describe('Memory routes', () => {
  beforeEach(() => {
    graphRuntime.centerForce.strength.mockClear();
    graphRuntime.chargeForce.distanceMax.mockClear();
    graphRuntime.chargeForce.strength.mockClear();
    graphRuntime.d3Force.mockClear();
    graphRuntime.linkForce.distance.mockClear();
    graphRuntime.linkForce.strength.mockClear();
    graphRuntime.reheat.mockClear();
  });

  it('renders the memory hub search tab', () => {
    render(
      <MemoryRouter future={{ v7_relativeSplatPath: true, v7_startTransition: true }} initialEntries={['/search']}>
        <Memory />
      </MemoryRouter>
    );

    expect(screen.getByRole('link', { name: 'Timeline' })).toHaveAttribute('href', '/timeline');
    expect(screen.getByText('stored fact about the GUI')).toBeInTheDocument();
  });

  it('renders timeline, inbox, entities, projects, and graph tabs', async () => {
    render(
      <MemoryRouter future={{ v7_relativeSplatPath: true, v7_startTransition: true }} initialEntries={['/timeline']}>
        <Memory />
      </MemoryRouter>
    );

    expect(screen.getByText('stored fact about the GUI')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('link', { name: 'Inbox' }));
    expect(screen.getByText(/pending memory candidate/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('link', { name: 'Entities' }));
    expect(screen.getByText('Kleos')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('link', { name: 'Projects' }));
    expect(screen.getByText('gui rebuild')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('link', { name: 'Graph' }));
    expect(screen.getByText(/2 nodes/)).toBeInTheDocument();
    expect(await screen.findByTestId('graph-canvas')).toHaveAttribute('data-height', '520');
    await waitFor(() => expect(graphRuntime.reheat).toHaveBeenCalled());
    expect(graphRuntime.linkForce.distance).toHaveBeenCalled();
    expect(graphRuntime.linkForce.strength).toHaveBeenCalled();
    expect(graphRuntime.chargeForce.strength).toHaveBeenCalled();
  });
});
