import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import * as api from '$lib/api/memory';
import { Timeline } from './Timeline';

afterEach(() => vi.restoreAllMocks());

// Wrap the component under test in the required React Query context.
function wrap(ui: React.ReactNode) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>);
}

// Clicking a year card drills into that year's months.
it('drills from year into months', async () => {
  vi.spyOn(api, 'getCalendar').mockImplementation(async (g) =>
    g === 'year' ? [{ bucket: '2026', count: 12 }] : [{ bucket: '03', count: 4 }]
  );
  wrap(<Timeline />);
  const year = await screen.findByText('2026');
  fireEvent.click(year);
  await waitFor(() => expect(screen.getByText('Mar')).toBeInTheDocument());
});
