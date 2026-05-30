import { useQuery } from '@tanstack/react-query';
import { Suspense, lazy, useEffect, useRef, useState } from 'react';
import type { ForceGraphMethods, LinkObject, NodeObject } from 'react-force-graph-3d';
import { getGraph } from '$lib/api/graph';
import { EDGE_COLOR, chargeStrength, linkDistance, linkStrength } from '$lib/graph/layout';
import type { GraphEdge, GraphNode } from '$lib/types';
import { EmptyState } from '../../ui/EmptyState';
import { Spinner } from '../../ui/Spinner';

const ForceGraph3D = lazy(() => import('react-force-graph-3d'));

// Pick a stable color from community id with category fallback.
function nodeColor(node: GraphNode) {
  const key = String(node.community_id ?? node.category);
  let hue = 0;
  for (let index = 0; index < key.length; index += 1) {
    hue = (hue * 31 + key.charCodeAt(index)) % 360;
  }
  return `hsl(${hue}, 55%, 60%)`;
}

// Render the real-similarity 3D memory graph.
export function Graph() {
  const graph = useQuery({ queryFn: () => getGraph(2000), queryKey: ['mem', 'graph'] });
  const graphRef = useRef<ForceGraphMethods | undefined>(undefined);
  const shellRef = useRef<HTMLElement | null>(null);
  const [size, setSize] = useState<{ height: number; width: number } | null>(null);

  useEffect(() => {
    const shell = shellRef.current;
    if (!shell) {
      return undefined;
    }
    // Measure the shell before mounting Three so the canvas never locks to a default viewport size.
    const update = () => {
      const rect = shell.getBoundingClientRect();
      const parentRect = shell.parentElement?.getBoundingClientRect();
      setSize({
        height: Math.max(360, Math.floor(rect.height || shell.clientHeight || 520)),
        width: Math.max(320, Math.floor(rect.width || shell.clientWidth || parentRect?.width || window.innerWidth || 640))
      });
    };
    update();
    const frame = requestAnimationFrame(update);
    if (typeof ResizeObserver === 'undefined') {
      window.addEventListener('resize', update);
      return () => {
        cancelAnimationFrame(frame);
        window.removeEventListener('resize', update);
      };
    }
    const observer = new ResizeObserver(update);
    observer.observe(shell);
    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
    };
  }, [graph.data?.nodes.length]);

  useEffect(() => {
    if (!graph.data) {
      return;
    }
    let frame: number | undefined;

    // Apply forces after the lazy Three graph has published its imperative ref.
    const applyForces = () => {
      const instance = graphRef.current;
      if (!instance) {
        frame = requestAnimationFrame(applyForces);
        return;
      }

      instance.d3Force('link')?.distance((link: LinkObject) => linkDistance((link as unknown as GraphEdge).weight)).strength(
        (link: LinkObject) => linkStrength((link as unknown as GraphEdge).weight)
      );
      instance.d3Force('charge')?.strength((node: NodeObject) => chargeStrength(node as unknown as GraphNode)).distanceMax(600);
      instance.d3Force('center')?.strength(0.03);
      instance.d3Force('clusterX', null);
      instance.d3Force('clusterY', null);
      instance.d3Force('clusterZ', null);
      instance.d3ReheatSimulation();
    };
    applyForces();
    return () => {
      if (frame !== undefined) {
        cancelAnimationFrame(frame);
      }
    };
  }, [graph.data]);

  if (graph.isLoading) {
    return <Spinner />;
  }
  if (!graph.data?.nodes.length) {
    return <EmptyState message="No memories with connections yet." />;
  }

  return (
    <section className="graph-shell" ref={shellRef}>
      <div className="graph-shell__meta">
        {graph.data.node_count} nodes / {graph.data.edge_count} edges / real similarity
      </div>
      {size ? (
        <Suspense fallback={<Spinner />}>
          <ForceGraph3D
            ref={graphRef}
            backgroundColor="#060608"
            cooldownTicks={300}
            graphData={{ links: graph.data.edges, nodes: graph.data.nodes }}
            height={size.height}
            linkColor={(link) => EDGE_COLOR[(link as GraphEdge).type] ?? '#333'}
            linkOpacity={0.5}
            linkSource="source"
            linkTarget="target"
            linkWidth={(link) => 0.3 + (link as GraphEdge).weight * 1.5}
            nodeColor={(node) => nodeColor(node as GraphNode)}
            nodeId="id"
            nodeLabel={(node) => (node as GraphNode).label}
            nodeVal={(node) => (node as GraphNode).size || 4}
            showNavInfo={false}
            warmupTicks={120}
            width={size.width}
          />
        </Suspense>
      ) : (
        <Spinner />
      )}
    </section>
  );
}
