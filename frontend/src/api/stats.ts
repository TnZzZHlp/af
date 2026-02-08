import { requestJson } from "./client";

export interface TimeSeriesPoint {
  time: string;
  count: number;
}

export interface CategoryCount {
  category: string | null;
  count: number;
}

export interface DashboardStats {
  requests_over_time: TimeSeriesPoint[];
  requests_by_provider: CategoryCount[];
}

export interface StatsQuery {
  start?: string; // ISO string
  end?: string;   // ISO string
}

export async function getDashboardStats(query: StatsQuery = {}): Promise<DashboardStats> {
  const params = new URLSearchParams();
  if (query.start) params.append("start", query.start);
  if (query.end) params.append("end", query.end);

  return requestJson<DashboardStats>(`/stats?${params.toString()}`);
}
