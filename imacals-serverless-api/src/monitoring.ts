import { supabase } from './supabase.js';

export interface LogEntry {
  id: string;
  timestamp: string;
  method: string;
  pathname: string;
  statusCode: number;
  durationMs: number;
  ip?: string;
  userAgent?: string;
  errorMessage?: string;
  errorCode?: string;
  stackTrace?: string;
  requestBody?: any;
  responseSummary?: any;
}

export interface EndpointMetric {
  path: string;
  method: string;
  count: number;
  errorCount: number;
  avgDurationMs: number;
}

export interface ActivityBucket {
  timestamp: string;
  count: number;
  errorCount: number;
}

export interface MetricsSummary {
  uptimeSeconds: number;
  startedAt: string;
  totalRequests: number;
  totalSuccess: number;
  totalClientErrors: number;
  totalServerErrors: number;
  avgLatencyMs: number;
  p95LatencyMs: number;
  minLatencyMs: number;
  maxLatencyMs: number;
  requestsPerMinute: number;
  errorRatePercentage: number;
  endpoints: EndpointMetric[];
  errorsByCode: Record<string, number>;
  activityBuckets: ActivityBucket[];
  supabase: {
    status: 'healthy' | 'degraded' | 'down';
    latencyMs: number;
    error?: string;
  };
}

const MAX_LOGS = 300;
const logs: LogEntry[] = [];
const latencies: number[] = [];
const endpointStats = new Map<string, { count: number; errorCount: number; totalDuration: number }>();
const errorsByCode: Record<string, number> = {};
const startedAt = new Date().toISOString();
const startTimeMs = Date.now();

// Sensitive keys to sanitize
const SENSITIVE_KEYS = new Set([
  'password',
  'token',
  'secret',
  'authorization',
  'key',
  'api_key',
  'access_token',
  'refresh_token',
  'credit_card',
]);

export function sanitizeData(data: any): any {
  if (!data || typeof data !== 'object') return data;
  if (Array.isArray(data)) return data.map((item) => sanitizeData(item));

  const clean: Record<string, any> = {};
  for (const [k, v] of Object.entries(data)) {
    if (SENSITIVE_KEYS.has(k.toLowerCase())) {
      clean[k] = '••••••••';
    } else if (typeof v === 'object' && v !== null) {
      clean[k] = sanitizeData(v);
    } else {
      clean[k] = v;
    }
  }
  return clean;
}

export function recordLog(entry: Omit<LogEntry, 'id'>): LogEntry {
  const fullEntry: LogEntry = {
    id: `log_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
    ...entry,
    requestBody: sanitizeData(entry.requestBody),
    responseSummary: sanitizeData(entry.responseSummary),
  };

  logs.unshift(fullEntry);
  if (logs.length > MAX_LOGS) {
    logs.pop();
  }

  // Update latencies buffer (keep last 500)
  latencies.push(entry.durationMs);
  if (latencies.length > 500) latencies.shift();

  // Update endpoint stats
  const epKey = `${entry.method} ${entry.pathname}`;
  const current = endpointStats.get(epKey) || { count: 0, errorCount: 0, totalDuration: 0 };
  current.count += 1;
  current.totalDuration += entry.durationMs;
  if (entry.statusCode >= 400) {
    current.errorCount += 1;
  }
  endpointStats.set(epKey, current);

  // Update error code count
  if (entry.errorCode) {
    errorsByCode[entry.errorCode] = (errorsByCode[entry.errorCode] || 0) + 1;
  }

  return fullEntry;
}

export function getLogs(filter?: {
  status?: string;
  method?: string;
  search?: string;
  limit?: number;
}): { logs: LogEntry[]; total: number; matching: number } {
  let filtered = [...logs];

  if (filter?.status) {
    if (filter.status === 'errors') {
      filtered = filtered.filter((l) => l.statusCode >= 400);
    } else if (filter.status === '5xx') {
      filtered = filtered.filter((l) => l.statusCode >= 500);
    } else if (filter.status === '4xx') {
      filtered = filtered.filter((l) => l.statusCode >= 400 && l.statusCode < 500);
    } else if (filter.status === '2xx') {
      filtered = filtered.filter((l) => l.statusCode >= 200 && l.statusCode < 300);
    } else if (filter.status === 'slow') {
      filtered = filtered.filter((l) => l.durationMs > 300);
    }
  }

  if (filter?.method && filter.method !== 'all') {
    const m = filter.method.toUpperCase();
    filtered = filtered.filter((l) => l.method.toUpperCase() === m);
  }

  if (filter?.search) {
    const q = filter.search.toLowerCase().trim();
    filtered = filtered.filter(
      (l) =>
        l.pathname.toLowerCase().includes(q) ||
        (l.errorMessage && l.errorMessage.toLowerCase().includes(q)) ||
        (l.errorCode && l.errorCode.toLowerCase().includes(q)) ||
        String(l.statusCode).includes(q)
    );
  }

  const matching = filtered.length;
  const limit = Math.min(filter?.limit || 100, MAX_LOGS);
  const sliced = filtered.slice(0, limit);

  return { logs: sliced, total: logs.length, matching };
}

export function clearLogs(): void {
  logs.length = 0;
  latencies.length = 0;
  endpointStats.clear();
  for (const k of Object.keys(errorsByCode)) {
    delete errorsByCode[k];
  }
}

export async function checkSupabaseHealth(): Promise<{
  status: 'healthy' | 'degraded' | 'down';
  latencyMs: number;
  error?: string;
}> {
  const start = Date.now();
  try {
    const { error } = await supabase.from('products').select('id').limit(1);
    const latencyMs = Date.now() - start;
    if (error) {
      return { status: 'degraded', latencyMs, error: error.message };
    }
    return { status: 'healthy', latencyMs };
  } catch (err: any) {
    const latencyMs = Date.now() - start;
    return { status: 'down', latencyMs, error: err?.message || 'Database unreachable' };
  }
}

export async function getMetrics(): Promise<MetricsSummary> {
  const uptimeSeconds = Math.floor((Date.now() - startTimeMs) / 1000);
  const totalRequests = logs.length;
  const totalServerErrors = logs.filter((l) => l.statusCode >= 500).length;
  const totalClientErrors = logs.filter((l) => l.statusCode >= 400 && l.statusCode < 500).length;
  const totalSuccess = logs.filter((l) => l.statusCode >= 200 && l.statusCode < 400).length;

  const totalErrors = totalServerErrors + totalClientErrors;
  const errorRatePercentage = totalRequests > 0 ? Number(((totalErrors / totalRequests) * 100).toFixed(2)) : 0;

  // Latencies stats
  let avgLatencyMs = 0;
  let minLatencyMs = 0;
  let maxLatencyMs = 0;
  let p95LatencyMs = 0;

  if (latencies.length > 0) {
    const sum = latencies.reduce((acc, curr) => acc + curr, 0);
    avgLatencyMs = Math.round(sum / latencies.length);
    minLatencyMs = Math.min(...latencies);
    maxLatencyMs = Math.max(...latencies);

    const sorted = [...latencies].sort((a, b) => a - b);
    const p95Idx = Math.floor(sorted.length * 0.95);
    p95LatencyMs = sorted[p95Idx] ?? sorted[sorted.length - 1];
  }

  const minutesOnline = Math.max(uptimeSeconds / 60, 0.1);
  const requestsPerMinute = Number((totalRequests / minutesOnline).toFixed(1));

  // Endpoint stats array
  const endpoints: EndpointMetric[] = [];
  for (const [key, stat] of endpointStats.entries()) {
    const [method, path] = key.split(' ');
    endpoints.push({
      method: method || 'GET',
      path: path || key,
      count: stat.count,
      errorCount: stat.errorCount,
      avgDurationMs: Math.round(stat.totalDuration / stat.count),
    });
  }
  endpoints.sort((a, b) => b.count - a.count);

  // Time-bucketed activity (last 10 buckets across logs)
  const bucketsMap = new Map<string, { count: number; errorCount: number }>();
  for (const log of logs) {
    const minute = log.timestamp.slice(0, 16); // "YYYY-MM-DDTHH:MM"
    const b = bucketsMap.get(minute) || { count: 0, errorCount: 0 };
    b.count += 1;
    if (log.statusCode >= 400) b.errorCount += 1;
    bucketsMap.set(minute, b);
  }

  const activityBuckets: ActivityBucket[] = Array.from(bucketsMap.entries())
    .map(([timestamp, val]) => ({
      timestamp,
      count: val.count,
      errorCount: val.errorCount,
    }))
    .slice(0, 20)
    .reverse();

  const dbHealth = await checkSupabaseHealth();

  return {
    uptimeSeconds,
    startedAt,
    totalRequests,
    totalSuccess,
    totalClientErrors,
    totalServerErrors,
    avgLatencyMs,
    p95LatencyMs,
    minLatencyMs,
    maxLatencyMs,
    requestsPerMinute,
    errorRatePercentage,
    endpoints,
    errorsByCode: { ...errorsByCode },
    activityBuckets,
    supabase: dbHealth,
  };
}
