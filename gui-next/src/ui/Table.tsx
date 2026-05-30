import type { ReactNode } from 'react';

// Render a simple fixed-width data table.
export function Table({ headers, rows }: { headers: string[]; rows: ReactNode[][] }) {
  return (
    <table style={{ borderCollapse: 'collapse', fontSize: 12, width: '100%' }}>
      <thead>
        <tr>
          {headers.map((header) => (
            <th
              key={header}
              style={{
                borderBottom: '1px solid var(--border)',
                color: 'var(--text-dim)',
                fontSize: 10,
                letterSpacing: 0,
                padding: '6px 10px',
                textAlign: 'left',
                textTransform: 'uppercase'
              }}
            >
              {header}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {rows.map((row, rowIndex) => (
          <tr key={rowIndex} style={{ borderBottom: '1px solid var(--border)' }}>
            {row.map((cell, cellIndex) => (
              <td key={cellIndex} style={{ color: 'var(--text)', padding: '8px 10px' }}>
                {cell}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
