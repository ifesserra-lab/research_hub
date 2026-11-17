import React, { useState, useEffect, useMemo } from 'react';
import { apiFetch } from '../services/api';
import FilterBuilderBar, { normalizeFilterColumns } from './FilterBuilderBar';

const CHART_KINDS = [
  { value: 'bar', label: 'Barras Horizontais (Ranking)' },
  { value: 'donut', label: 'Rosca (Proporção / %)' },
  { value: 'columns', label: 'Colunas Verticais' },
];

const METRIC_MODES = [
  { value: 'count', label: 'Contagem de Registros' },
  { value: 'sum', label: 'Soma de Coluna Numérica ou Quantidade de Vínculos' },
];

export default function CustomChartModal({
  open,
  scope = 'horizon-v2',
  defaultEntity = '',
  availableEntities = [],
  fallbackColumns = [],
  initialFilterSpec = { logic: 'and', rules: [] },
  onClose,
  onCreated,
}) {
  const entities = useMemo(() => {
    if (Array.isArray(availableEntities) && availableEntities.length > 0) {
      return availableEntities;
    }
    if (defaultEntity) {
      return [
        {
          entity: defaultEntity,
          label: defaultEntity,
          columns: normalizeFilterColumns(fallbackColumns).map((c) => c.key),
        },
      ];
    }
    return [];
  }, [availableEntities, defaultEntity, fallbackColumns]);

  const [entity, setEntity] = useState(defaultEntity || entities[0]?.entity || '');
  const [title, setTitle] = useState('');
  const [kind, setKind] = useState('bar');
  const [groupBy, setGroupBy] = useState('');
  const [metric, setMetric] = useState('count');
  const [metricField, setMetricField] = useState('');
  const [filterSpec, setFilterSpec] = useState(initialFilterSpec || { logic: 'and', rules: [] });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (open) {
      const targetEntity = defaultEntity || entities[0]?.entity || '';
      setEntity(targetEntity);
      setFilterSpec(
        initialFilterSpec && Array.isArray(initialFilterSpec.rules)
          ? { logic: initialFilterSpec.logic || 'and', rules: [...initialFilterSpec.rules] }
          : { logic: 'and', rules: [] }
      );
      setError(null);
    }
  }, [open, defaultEntity, entities, initialFilterSpec]);

  const activeColumns = useMemo(() => {
    const found = entities.find((e) => e.entity === entity);
    if (found && Array.isArray(found.columns) && found.columns.length > 0) {
      return normalizeFilterColumns(found.columns);
    }
    return normalizeFilterColumns(fallbackColumns);
  }, [entities, entity, fallbackColumns]);

  useEffect(() => {
    if (activeColumns.length > 0) {
      if (!activeColumns.some((c) => c.key === groupBy)) {
        setGroupBy(activeColumns[0].key);
      }
      if (!metricField || !activeColumns.some((c) => c.key === metricField)) {
        setMetricField(activeColumns[0].key);
      }
    }
  }, [activeColumns, groupBy, metricField]);

  if (!open) return null;

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError(null);

    const resolvedTitle =
      title.trim() ||
      `${metric === 'sum' ? `Soma (${metricField})` : 'Registros'} por ${groupBy}`;

    if (!entity || !groupBy) {
      setError('Selecione a tabela de origem e a coluna de agrupamento.');
      return;
    }
    if (metric === 'sum' && !metricField) {
      setError('Selecione a coluna numérica ou de vínculos para somar.');
      return;
    }

    setSaving(true);
    try {
      const saved = await apiFetch('/custom-charts', {
        method: 'POST',
        body: JSON.stringify({
          scope,
          entity,
          title: resolvedTitle,
          kind,
          group_by: groupBy,
          metric,
          metric_field: metric === 'sum' ? metricField : null,
          filter_spec: filterSpec,
        }),
      });
      if (typeof window !== 'undefined') {
        window.dispatchEvent(new CustomEvent('horizon:imported'));
      }
      onCreated?.(saved);
      onClose?.();
    } catch (err) {
      setError(err.message || 'Erro ao criar gráfico personalizado.');
    } finally {
      setSaving(false);
    }
  };

  return (
    <div
      data-testid="custom-chart-modal"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 overflow-y-auto"
    >
      <div className="bg-white rounded-2xl border border-gray-200 shadow-xl w-full max-w-2xl p-6 my-8">
        <div className="flex items-center justify-between pb-3 mb-4 border-b border-gray-100">
          <div>
            <h3 className="text-lg font-bold text-gray-900">
              📊 Criar Novo Gráfico no Dashboard
            </h3>
            <p className="text-xs text-gray-500 mt-0.5">
              O gráfico ficará salvo no painel da tabela e também no Dashboard Geral da aba atual.
            </p>
          </div>
          <button
            type="button"
            data-testid="close-custom-chart-modal-btn"
            onClick={onClose}
            className="text-gray-400 hover:text-gray-700 text-xl font-bold leading-none px-2 py-1"
          >
            ×
          </button>
        </div>

        {error && (
          <div className="mb-4 p-3 rounded-lg bg-red-50 border border-red-200 text-xs text-red-700">
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-semibold text-gray-700 mb-1">
                Título do Gráfico
              </label>
              <input
                type="text"
                data-testid="custom-chart-title-input"
                placeholder="Ex: Iniciativas Ativas por Campus"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                className="w-full px-3 py-2 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
              />
            </div>

            <div>
              <label className="block text-xs font-semibold text-gray-700 mb-1">
                Tabela / Dataset de Origem
              </label>
              <select
                data-testid="custom-chart-entity-select"
                value={entity}
                onChange={(e) => setEntity(e.target.value)}
                className="w-full px-3 py-2 text-sm bg-white border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
              >
                {entities.map((ent) => (
                  <option key={ent.entity} value={ent.entity}>
                    {ent.label || ent.entity}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-semibold text-gray-700 mb-1">
                Agrupar por Coluna (Eixo / Fatias)
              </label>
              <select
                data-testid="custom-chart-groupby-select"
                value={groupBy}
                onChange={(e) => setGroupBy(e.target.value)}
                className="w-full px-3 py-2 text-sm bg-white border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
              >
                {activeColumns.map((col) => (
                  <option key={col.key} value={col.key}>
                    {col.label}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-xs font-semibold text-gray-700 mb-1">
                Tipo Visual do Gráfico
              </label>
              <select
                data-testid="custom-chart-kind-select"
                value={kind}
                onChange={(e) => setKind(e.target.value)}
                className="w-full px-3 py-2 text-sm bg-white border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
              >
                {CHART_KINDS.map((k) => (
                  <option key={k.value} value={k.value}>
                    {k.label}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-semibold text-gray-700 mb-1">
                Métrica do Eixo de Valores
              </label>
              <select
                data-testid="custom-chart-metric-select"
                value={metric}
                onChange={(e) => setMetric(e.target.value)}
                className="w-full px-3 py-2 text-sm bg-white border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
              >
                {METRIC_MODES.map((m) => (
                  <option key={m.value} value={m.value}>
                    {m.label}
                  </option>
                ))}
              </select>
            </div>

            {metric === 'sum' && (
              <div>
                <label className="block text-xs font-semibold text-gray-700 mb-1">
                  Coluna para Somar (Numérica ou Vínculos)
                </label>
                <select
                  data-testid="custom-chart-metric-field-select"
                  value={metricField}
                  onChange={(e) => setMetricField(e.target.value)}
                  className="w-full px-3 py-2 text-sm bg-white border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
                >
                  {activeColumns.map((col) => (
                    <option key={col.key} value={col.key}>
                      {col.label}
                    </option>
                  ))}
                </select>
              </div>
            )}
          </div>

          <div>
            <FilterBuilderBar
              title="Filtros Embutidos no Gráfico"
              columns={activeColumns}
              filterSpec={filterSpec}
              onChange={setFilterSpec}
              compact
            />
          </div>

          <div className="flex items-center justify-end gap-2 pt-3 border-t border-gray-100">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-xs font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
            >
              Cancelar
            </button>
            <button
              type="submit"
              data-testid="save-custom-chart-btn"
              disabled={saving}
              className="px-4 py-2 text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 rounded-lg shadow-xs transition-colors"
            >
              {saving ? 'Salvando...' : 'Salvar Gráfico no Dashboard'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
