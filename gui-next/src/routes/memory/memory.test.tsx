import { fireEvent, render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';
import { Memory } from './Memory';

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
      return { data: undefined, isLoading: false };
    }
  };
});

describe('Memory routes', () => {
  it('renders the memory hub search tab', () => {
    render(
      <MemoryRouter future={{ v7_relativeSplatPath: true, v7_startTransition: true }} initialEntries={['/search']}>
        <Memory />
      </MemoryRouter>
    );

    expect(screen.getByRole('link', { name: 'Timeline' })).toHaveAttribute('href', '/timeline');
    expect(screen.getByText('stored fact about the GUI')).toBeInTheDocument();
  });

  it('renders timeline, inbox, entities, projects, and graph tabs', () => {
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
    expect(screen.getByText('graph')).toBeInTheDocument();
  });
});
