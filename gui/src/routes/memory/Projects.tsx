import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { listProjects } from '$lib/api/memory';
import { Badge } from '../../ui/Badge';
import { FloatingCard, FloatingCardField } from '../../ui/cards3d';
import { Spinner } from '../../ui/Spinner';

// Render memory projects as a floating 3D card field, hiding empty projects
// (and thus stale test fixtures) until the user opts to show them.
export function Projects() {
  const projects = useQuery({ queryFn: () => listProjects(), queryKey: ['mem', 'projects'] });
  const [showEmpty, setShowEmpty] = useState(false);

  if (projects.isLoading) {
    return <Spinner />;
  }

  const all = projects.data ?? [];
  const emptyCount = all.filter((p) => (p.memory_count ?? 0) === 0).length;
  const visible = showEmpty ? all : all.filter((p) => (p.memory_count ?? 0) > 0);

  return (
    <div className="memory-view">
      {emptyCount > 0 && (
        <div className="kl-projects-toolbar">
          <button onClick={() => setShowEmpty((v) => !v)}>
            {showEmpty ? 'Hide empty' : `Show empty (${emptyCount})`}
          </button>
        </div>
      )}
      <FloatingCardField>
        {visible.map((project, i) => (
          <FloatingCard
            key={project.id}
            title={project.name}
            subtitle={project.description}
            count={project.memory_count ?? 0}
            index={i}
            isEmpty={(project.memory_count ?? 0) === 0}
            badges={<Badge label={project.status} />}
          />
        ))}
      </FloatingCardField>
    </div>
  );
}
