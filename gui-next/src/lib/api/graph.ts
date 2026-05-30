import { request } from '$lib/http';
import type { GraphData } from '$lib/types';

// Fetch memory graph data with a node cap.
export const getGraph = (max = 1500) => request<GraphData>(`/graph?max=${max}`);
