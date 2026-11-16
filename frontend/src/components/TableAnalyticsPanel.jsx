import React, { useState, useEffect } from 'react';
import AnalyticsDashboard from './AnalyticsDashboard.jsx';

/**
 * Painel expansível "Ver Gráficos da Tabela" embutido em cada página de tabela individual
 * (Horizon, Horizon Legacy, SRC, Egressos e Egressos — Dados, SEP-040 FR-006 / SEP-041).
 */
export default function TableAnalyticsPanel({
  endpoint,
  label = 'Tabela',
  defaultOpen = false,
  externalFilterSpec = null,
  fallbackColumns = [],
  forceOpenSignal = 0,
}) {
  const [open, setOpen] = useState(defaultOpen);

  useEffect(() => {
    if (forceOpenSignal > 0) {
      setOpen(true);
    }
  }, [forceOpenSignal]);

  return (
    <div className="mb-4">
      <div className="flex items-center justify-between bg-white border border-slate-200 rounded-lg px-4 py-2.5 shadow-sm">
        <div className="flex items-center gap-2 text-sm text-slate-700">
          <span className="font-semibold text-slate-900">
            Painel Analítico — {label}
          </span>
          <span className="hidden sm:inline text-xs text-slate-500">
            (Gráficos e métricas calculados sobre os registros desta aba)
          </span>
        </div>
        <button
          type="button"
          data-testid="toggle-table-analytics-btn"
          onClick={() => setOpen((prev) => !prev)}
          className={`px-3.5 py-1.5 rounded-md text-xs font-semibold border transition-colors ${
            open
              ? 'bg-slate-900 text-white border-slate-900'
              : 'bg-indigo-50 text-indigo-700 border-indigo-200 hover:bg-indigo-100'
          }`}
        >
          {open ? 'Ocultar Gráficos da Tabela' : '📊 Ver Gráficos da Tabela'}
        </button>
      </div>

      {open && (
        <AnalyticsDashboard
          endpoint={endpoint}
          title={`Gráficos da Tabela — ${label}`}
          compact
          externalFilterSpec={externalFilterSpec}
          fallbackColumns={fallbackColumns}
        />
      )}
    </div>
  );
}
