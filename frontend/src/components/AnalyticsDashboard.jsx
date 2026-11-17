import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { apiFetch } from '../services/api.js';
import FilterBuilderBar, { serializeFilterSpec, normalizeFilterColumns } from './FilterBuilderBar.jsx';
import CustomChartModal from './CustomChartModal.jsx';

const KPI_TONE_CLASSES = {
  emerald: 'border-emerald-200 bg-emerald-50/60 text-emerald-900',
  indigo: 'border-indigo-200 bg-indigo-50/60 text-indigo-900',
  amber: 'border-amber-200 bg-amber-50/60 text-amber-900',
  sky: 'border-sky-200 bg-sky-50/60 text-sky-900',
  rose: 'border-rose-200 bg-rose-50/60 text-rose-900',
  slate: 'border-slate-200 bg-slate-50 text-slate-800',
};

const PALETTE = [
  '#10b981', // emerald-500
  '#4f46e5', // indigo-600
  '#0ea5e9', // sky-500
  '#f59e0b', // amber-500
  '#f43f5e', // rose-500
  '#8b5cf6', // violet-500
  '#64748b', // slate-500
  '#14b8a6', // teal-500
];

function inferScopeAndEntity(endpoint, payload) {
  if (payload?.scope) {
    const [, qs = ''] = endpoint.split('?');
    const q = new URLSearchParams(qs);
    return {
      scope: payload.scope,
      entity: q.get('entity') || q.get('file') || '',
    };
  }
  const [path, qs = ''] = endpoint.split('?');
  const q = new URLSearchParams(qs);
  if (path.startsWith('/horizon-v2')) {
    return { scope: 'horizon-v2', entity: q.get('entity') || '' };
  }
  if (path.startsWith('/src')) {
    return { scope: 'src', entity: 'acoes' };
  }
  if (path.startsWith('/egressos-dados')) {
    return { scope: 'egressos-dados', entity: q.get('file') || '' };
  }
  if (path.startsWith('/egressos')) {
    return { scope: 'egressos', entity: q.get('file') || '' };
  }
  return { scope: 'horizon', entity: q.get('entity') || '' };
}

function DonutChart({ items }) {
  const total = items.reduce((acc, it) => acc + (it.count || 0), 0) || 1;
  const radius = 56;
  const circumference = 2 * Math.PI * radius;
  let offsetAcc = 0;

  return (
    <div className="flex flex-col md:flex-row items-center gap-6">
      <div className="relative w-36 h-36 shrink-0 flex items-center justify-center">
        <svg viewBox="0 0 140 140" className="w-36 h-36 -rotate-90">
          <circle
            cx="70"
            cy="70"
            r={radius}
            fill="transparent"
            stroke="#f1f5f9"
            strokeWidth="18"
          />
          {items.map((it, idx) => {
            const frac = (it.count || 0) / total;
            const dash = frac * circumference;
            const dashOffset = -offsetAcc;
            offsetAcc += dash;
            const color = PALETTE[idx % PALETTE.length];
            return (
              <circle
                key={it.label}
                cx="70"
                cy="70"
                r={radius}
                fill="transparent"
                stroke={color}
                strokeWidth="18"
                strokeDasharray={`${dash} ${circumference}`}
                strokeDashoffset={dashOffset}
              />
            );
          })}
        </svg>
        <div className="absolute inset-0 flex flex-col items-center justify-center text-center">
          <span className="text-xs text-slate-500">Total</span>
          <span className="text-sm font-bold text-slate-800">
            {total.toLocaleString('pt-BR')}
          </span>
        </div>
      </div>

      <div className="flex-1 w-full space-y-2">
        {items.map((it, idx) => {
          const color = PALETTE[idx % PALETTE.length];
          return (
            <div
              key={it.label}
              className="flex items-center justify-between text-xs border-b border-slate-100 pb-1.5 last:border-none"
            >
              <div className="flex items-center gap-2 min-w-0">
                <span
                  className="w-3 h-3 rounded-sm shrink-0"
                  style={{ backgroundColor: color }}
                />
                <span className="font-medium text-slate-800 truncate">{it.label}</span>
              </div>
              <div className="flex items-center gap-2 shrink-0 ml-2">
                <span className="text-slate-600 font-mono">
                  {it.formatted_value || it.count?.toLocaleString('pt-BR')}
                </span>
                <span className="px-1.5 py-0.5 rounded bg-slate-100 text-slate-700 font-semibold">
                  {Number(it.percentage ?? 0).toFixed(1)}%
                </span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function BarChart({ items, isFinancial = false }) {
  return (
    <div className="space-y-2.5">
      {items.map((it, idx) => {
        const widthPct = Math.max(2, Math.min(100, Number(it.percentage ?? 0)));
        const barColor = isFinancial
          ? 'bg-amber-500'
          : idx === 0
          ? 'bg-indigo-600'
          : 'bg-sky-500';
        return (
          <div key={it.label} className="space-y-1">
            <div className="flex items-center justify-between text-xs gap-2">
              <span className="font-medium text-slate-800 truncate" title={it.label}>
                {idx + 1}. {it.label}
              </span>
              <span className="font-mono text-slate-600 shrink-0">
                {it.formatted_value || it.count?.toLocaleString('pt-BR')}
              </span>
            </div>
            <div className="w-full h-2 bg-slate-100 rounded-full overflow-hidden">
              <div
                className={`h-full rounded-full transition-all ${barColor}`}
                style={{ width: `${widthPct}%` }}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
}

function ColumnsChart({ items }) {
  return (
    <div className="pt-2">
      <div className="grid grid-cols-3 sm:grid-cols-6 md:grid-cols-12 gap-2 items-end h-44 border-b border-slate-200 pb-2">
        {items.map((it) => {
          const heightPct = Math.max(6, Math.min(100, Number(it.percentage ?? 0)));
          return (
            <div
              key={it.label}
              className="flex flex-col items-center justify-end h-full group"
              title={`${it.label}: ${it.formatted_value || it.count}`}
            >
              <span className="text-[10px] font-mono text-slate-600 mb-1">
                {it.count?.toLocaleString('pt-BR')}
              </span>
              <div
                className="w-full max-w-[28px] bg-indigo-500 group-hover:bg-indigo-600 rounded-t transition-all"
                style={{ height: `${heightPct}%` }}
              />
              <span className="text-[10px] font-medium text-slate-700 mt-1 truncate max-w-full">
                {it.label}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function ChartCard({ section, onDeleteCustomChart = null }) {
  if (!section || !Array.isArray(section.items)) {
    return null;
  }
  if (section.items.length === 0 && !section.is_custom) {
    return null;
  }

  return (
    <div
      data-testid={`chart-section-${section.id}`}
      className="bg-white border border-slate-200 rounded-xl shadow-sm p-5 flex flex-col justify-between"
    >
      <div className="mb-4 flex items-start justify-between gap-2">
        <div>
          <div className="flex items-center gap-2">
            <h3 className="text-sm font-bold text-slate-900">{section.title}</h3>
            {section.is_custom && (
              <span className="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-indigo-100 text-indigo-800">
                Personalizado
              </span>
            )}
          </div>
          {section.subtitle && (
            <p className="text-xs text-slate-500 mt-0.5">{section.subtitle}</p>
          )}
        </div>

        {section.is_custom && section.custom_id != null && onDeleteCustomChart && (
          <button
            type="button"
            data-testid={`delete-custom-chart-${section.custom_id}`}
            onClick={() => onDeleteCustomChart(section.custom_id)}
            className="px-2 py-1 text-xs font-medium text-rose-700 bg-rose-50 hover:bg-rose-100 border border-rose-200 rounded-md transition-colors shrink-0"
            title="Excluir gráfico personalizado"
          >
            Excluir
          </button>
        )}
      </div>

      {section.items.length === 0 ? (
        <div className="text-xs text-slate-400 italic py-6 text-center">
          Nenhum registro corresponde aos filtros deste gráfico.
        </div>
      ) : section.kind === 'donut' ? (
        <DonutChart items={section.items} />
      ) : section.kind === 'columns' ? (
        <ColumnsChart items={section.items} />
      ) : section.kind === 'financial_table' ? (
        <BarChart items={section.items} isFinancial />
      ) : (
        <BarChart items={section.items} />
      )}
    </div>
  );
}

export default function AnalyticsDashboard({
  endpoint = '/horizon-v2/analytics',
  title = null,
  compact = false,
  externalFilterSpec = null,
  fallbackColumns = [],
}) {
  const [payload, setPayload] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [filterSpec, setFilterSpec] = useState(
    externalFilterSpec || { logic: 'and', rules: [] }
  );
  const [showChartModal, setShowChartModal] = useState(false);

  useEffect(() => {
    if (externalFilterSpec && Array.isArray(externalFilterSpec.rules)) {
      setFilterSpec(externalFilterSpec);
    }
  }, [externalFilterSpec]);

  const serializedFilters = useMemo(() => serializeFilterSpec(filterSpec), [filterSpec]);

  const effectiveEndpoint = useMemo(() => {
    if (!serializedFilters) return endpoint;
    const sep = endpoint.includes('?') ? '&' : '?';
    return `${endpoint}${sep}filters=${encodeURIComponent(serializedFilters)}`;
  }, [endpoint, serializedFilters]);

  const loadAnalytics = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await apiFetch(effectiveEndpoint);
      setPayload(data);
    } catch (err) {
      setError(err?.message || String(err));
    } finally {
      setLoading(false);
    }
  }, [effectiveEndpoint]);

  useEffect(() => {
    loadAnalytics();
  }, [loadAnalytics]);

  useEffect(() => {
    const refresh = () => loadAnalytics();
    document.addEventListener('src:imported', refresh);
    document.addEventListener('egressos:imported', refresh);
    document.addEventListener('egressos-dados:imported', refresh);
    window.addEventListener('horizon:imported', refresh);
    return () => {
      document.removeEventListener('src:imported', refresh);
      document.removeEventListener('egressos:imported', refresh);
      document.removeEventListener('egressos-dados:imported', refresh);
      window.removeEventListener('horizon:imported', refresh);
    };
  }, [loadAnalytics]);

  const handleDeleteCustomChart = async (customId) => {
    try {
      await apiFetch(`/custom-charts/${customId}`, { method: 'DELETE' });
      await loadAnalytics();
    } catch (err) {
      setError(err?.message || String(err));
    }
  };

  const { scope: inferredScope, entity: inferredEntity } = useMemo(
    () => inferScopeAndEntity(endpoint, payload),
    [endpoint, payload]
  );

  const dashboardColumns = useMemo(() => {
    if (Array.isArray(payload?.available_entities) && payload.available_entities.length > 0) {
      const allCols = [];
      for (const ent of payload.available_entities) {
        if (inferredEntity && ent.entity !== inferredEntity) continue;
        for (const col of ent.columns || []) {
          allCols.push(col);
        }
      }
      if (allCols.length === 0) {
        for (const ent of payload.available_entities) {
          for (const col of ent.columns || []) {
            allCols.push(col);
          }
        }
      }
      return normalizeFilterColumns(allCols);
    }
    return normalizeFilterColumns(fallbackColumns);
  }, [payload, inferredEntity, fallbackColumns]);

  if (loading && !payload) {
    return (
      <div
        data-testid="analytics-loading"
        className="my-6 bg-white border border-slate-200 rounded-xl shadow-sm p-6 text-sm text-slate-500"
      >
        Calculando indicadores e gráficos da base local…
      </div>
    );
  }

  if (error) {
    return (
      <div
        role="alert"
        className="my-6 bg-rose-50 border border-rose-200 rounded-xl p-4 text-xs text-rose-800 flex items-center justify-between"
      >
        <span>Erro ao carregar o Dashboard Analítico: {error}</span>
        <button
          type="button"
          onClick={loadAnalytics}
          className="px-3 py-1 bg-white border border-rose-300 rounded font-semibold hover:bg-rose-100"
        >
          Tentar novamente
        </button>
      </div>
    );
  }

  if (!payload || !payload.loaded) {
    return (
      <div
        data-testid="analytics-empty-state"
        className="my-6 bg-white border border-slate-200 rounded-xl shadow-sm p-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4"
      >
        <div>
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-amber-50 text-amber-800 border border-amber-200 mb-2">
            <span className="w-1.5 h-1.5 rounded-full bg-amber-400" />
            Projeto Inativo • Sem Dados Carregados
          </span>
          <h2 className="text-base font-bold text-slate-900">
            {title || payload?.title || 'Dashboard Analítico'}
          </h2>
          <p className="text-xs text-slate-600 mt-1">
            Nenhum dado carregado ainda nesta aba. Importe o pacote ou sincronize pelo GitHub abaixo para habilitar todos os gráficos e métricas.
          </p>
        </div>
        <button
          type="button"
          onClick={loadAnalytics}
          className="px-3.5 py-2 text-xs font-semibold text-slate-700 bg-slate-100 hover:bg-slate-200 rounded-lg transition-colors shrink-0"
        >
          Atualizar Gráficos
        </button>
      </div>
    );
  }

  return (
    <section
      data-testid="analytics-dashboard"
      className={compact ? 'space-y-4 my-4' : 'space-y-6 my-8'}
    >
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 bg-white border border-slate-200 rounded-xl shadow-sm p-5">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-emerald-100 text-emerald-800 border border-emerald-300">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
              Base Ativa • {(payload.total_rows ?? 0).toLocaleString('pt-BR')} registros
            </span>
          </div>
          <h2 className="text-lg font-bold text-slate-900">
            {title || payload.title || 'Dashboard Analítico'}
          </h2>
        </div>

        <div className="flex flex-wrap items-center gap-2 self-start sm:self-auto">
          <button
            type="button"
            data-testid="open-custom-chart-modal-btn"
            onClick={() => setShowChartModal(true)}
            className="px-3.5 py-2 text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg shadow-xs transition-colors"
          >
            + Novo Gráfico Personalizado
          </button>
          <button
            type="button"
            onClick={loadAnalytics}
            className="px-3.5 py-2 text-xs font-semibold text-indigo-700 bg-indigo-50 hover:bg-indigo-100 border border-indigo-200 rounded-lg transition-colors"
          >
            ↻ Atualizar Indicadores
          </button>
        </div>
      </div>

      {dashboardColumns.length > 0 && (
        <FilterBuilderBar
          title="Filtrar Indicadores e Gráficos do Dashboard"
          columns={dashboardColumns}
          filterSpec={filterSpec}
          onChange={setFilterSpec}
          onCreateChart={() => setShowChartModal(true)}
          compact={compact}
        />
      )}

      {Array.isArray(payload.kpis) && payload.kpis.length > 0 && (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {payload.kpis.map((kpi) => {
            const toneClass = KPI_TONE_CLASSES[kpi.tone] || KPI_TONE_CLASSES.slate;
            return (
              <div
                key={kpi.label}
                className={`border rounded-xl p-4 shadow-sm ${toneClass}`}
              >
                <div className="text-xs font-semibold uppercase tracking-wide opacity-80">
                  {kpi.label}
                </div>
                <div className="text-2xl font-extrabold mt-1">{kpi.value}</div>
                {kpi.sublabel && (
                  <div className="text-xs opacity-80 mt-1">{kpi.sublabel}</div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {Array.isArray(payload.charts) && payload.charts.length > 0 && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
          {payload.charts.map((sec) => (
            <ChartCard
              key={sec.id}
              section={sec}
              onDeleteCustomChart={handleDeleteCustomChart}
            />
          ))}
        </div>
      )}

      <CustomChartModal
        open={showChartModal}
        scope={inferredScope}
        defaultEntity={inferredEntity}
        availableEntities={payload.available_entities || []}
        fallbackColumns={dashboardColumns}
        initialFilterSpec={filterSpec}
        onClose={() => setShowChartModal(false)}
        onCreated={() => loadAnalytics()}
      />
    </section>
  );
}
