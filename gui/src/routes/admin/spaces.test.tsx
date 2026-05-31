import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as adminApi from '$lib/api/admin';
import { Spaces } from './Spaces';

vi.mock('$lib/api/admin');

// Render the Spaces page inside a fresh, retry-free query client.
function renderSpaces() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <Spaces />
    </QueryClientProvider>
  );
}

describe('Spaces admin page', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('shows grant management for admins', async () => {
    vi.mocked(adminApi.getMe).mockResolvedValue({
      is_admin: true,
      scopes: ['admin'],
      user_id: 1,
      username: 'root'
    });
    vi.mocked(adminApi.listUsers).mockResolvedValue([
      { id: 2, username: 'alice' },
      { id: 3, username: 'bob' }
    ]);

    renderSpaces();

    expect(await screen.findByRole('heading', { name: 'Spaces & Sharing' })).toBeInTheDocument();
    // The owner picker is populated from the user list.
    expect(await screen.findByRole('option', { name: 'alice' })).toBeInTheDocument();
  });

  it('blocks non-admins', async () => {
    vi.mocked(adminApi.getMe).mockResolvedValue({
      is_admin: false,
      scopes: ['read', 'write'],
      user_id: 5,
      username: 'alice'
    });

    renderSpaces();

    expect(await screen.findByText('Admins only')).toBeInTheDocument();
  });
});
