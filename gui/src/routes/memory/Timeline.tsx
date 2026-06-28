import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { getCalendar, listMemoriesByDay } from '$lib/api/memory';
import { FloatingCard, FloatingCardField } from '../../ui/cards3d';
import { EmptyState } from '../../ui/EmptyState';
import { Spinner } from '../../ui/Spinner';

// Month-number (1-12) to short label for the month cards.
const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

// Render the year -> month -> day -> memories drill-down.
export function Timeline() {
  const [year, setYear] = useState<number | null>(null);
  const [month, setMonth] = useState<number | null>(null);
  const [day, setDay] = useState<number | null>(null);

  const years = useQuery({
    queryFn: () => getCalendar('year'),
    queryKey: ['mem', 'cal', 'year']
  });
  const months = useQuery({
    enabled: year !== null,
    queryFn: () => getCalendar('month', year!),
    queryKey: ['mem', 'cal', 'month', year]
  });
  const days = useQuery({
    enabled: year !== null && month !== null,
    queryFn: () => getCalendar('day', year!, month!),
    queryKey: ['mem', 'cal', 'day', year, month]
  });
  const memories = useQuery({
    enabled: year !== null && month !== null && day !== null,
    queryFn: () => listMemoriesByDay(year!, month!, day!),
    queryKey: ['mem', 'cal', 'list', year, month, day]
  });

  // Build a count lookup keyed by zero-padded bucket for month/day fields.
  const monthCount = (m: number) =>
    Number(months.data?.find((b) => Number(b.bucket) === m)?.count ?? 0);
  const dayCount = (d: number) =>
    Number(days.data?.find((b) => Number(b.bucket) === d)?.count ?? 0);

  const daysInMonth = year !== null && month !== null ? new Date(year, month, 0).getDate() : 0;

  return (
    <div className="memory-view">
      <nav className="kl-breadcrumb" aria-label="Timeline position">
        <button onClick={() => { setYear(null); setMonth(null); setDay(null); }}>Timeline</button>
        {year !== null && (
          <>
            <span className="sep">/</span>
            <button onClick={() => { setMonth(null); setDay(null); }}>{year}</button>
          </>
        )}
        {month !== null && (
          <>
            <span className="sep">/</span>
            <button onClick={() => setDay(null)}>{MONTHS[month - 1]}</button>
          </>
        )}
        {day !== null && (
          <>
            <span className="sep">/</span>
            <span>{day}</span>
          </>
        )}
      </nav>

      {/* Year level */}
      {year === null &&
        (years.isLoading ? (
          <Spinner />
        ) : years.data && years.data.length > 0 ? (
          <FloatingCardField>
            {years.data.map((b, i) => (
              <FloatingCard
                key={b.bucket}
                title={b.bucket}
                count={b.count}
                index={i}
                onClick={() => setYear(Number(b.bucket))}
              />
            ))}
          </FloatingCardField>
        ) : (
          <EmptyState message="No memories yet." />
        ))}

      {/* Month level */}
      {year !== null && month === null &&
        (months.isLoading ? (
          <Spinner />
        ) : (
          <FloatingCardField>
            {MONTHS.map((label, idx) => {
              const m = idx + 1;
              const c = monthCount(m);
              return (
                <FloatingCard
                  key={label}
                  title={label}
                  count={c}
                  index={idx}
                  isEmpty={c === 0}
                  onClick={() => setMonth(m)}
                />
              );
            })}
          </FloatingCardField>
        ))}

      {/* Day level */}
      {year !== null && month !== null && day === null &&
        (days.isLoading ? (
          <Spinner />
        ) : (
          <FloatingCardField>
            {Array.from({ length: daysInMonth }, (_, i) => i + 1).map((d) => {
              const c = dayCount(d);
              return (
                <FloatingCard
                  key={d}
                  title={String(d)}
                  count={c}
                  index={d - 1}
                  isEmpty={c === 0}
                  onClick={() => setDay(d)}
                />
              );
            })}
          </FloatingCardField>
        ))}

      {/* Memories level */}
      {day !== null &&
        (memories.isLoading ? (
          <Spinner />
        ) : memories.data && memories.data.length > 0 ? (
          <div className="memory-list">
            {memories.data.map((m) => (
              <article className="glass memory-card" key={m.id}>
                <div className="memory-card__meta">
                  <span>{m.category}</span>
                  <span>{m.created_at.slice(0, 16)}</span>
                </div>
                <p>{m.content}</p>
              </article>
            ))}
          </div>
        ) : (
          <EmptyState message="No memories on this day." />
        ))}
    </div>
  );
}
