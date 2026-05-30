// Render a quiet placeholder when a panel has no records.
export function EmptyState({ message = 'Nothing here yet.' }: { message?: string }) {
  return (
    <div style={{ color: 'var(--text-faint)', fontSize: 12, padding: 'var(--sp-6)', textAlign: 'center' }}>
      {message}
    </div>
  );
}
