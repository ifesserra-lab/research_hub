import React, { useState, useEffect } from 'react';

export const FILTER_OPERATORS = [
  { value: 'contains', label: 'Contém' },
  { value: 'equals', label: 'Igual a' },
  { value: 'not_equals', label: 'Diferente de' },
  { value: 'is_empty', label: 'Está vazio' },
  { value: 'not_empty', label: 'Está preenchido' },
];

export function normalizeFilterColumns(columns = []) {
  const seen = new Set();
  const list = [];
  for (const col of columns) {
    if (!col) continue;
    const key = typeof col === 'string' ? col : (col.key || col.name || '');
    const label = typeof col === 'string' ? col : (col.label || col.key || col.name || '');
    if (!key || seen.has(key)) continue;
    seen.add(key);
    list.push({ key, label });
  }
  return list;
}

export function serializeFilterSpec(filterSpec) {
  if (!filterSpec || !Array.isArray(filterSpec.rules) || filterSpec.rules.length === 0) {
    return null;
  }
  return JSON.stringify({
    logic: filterSpec.logic === 'or' ? 'or' : 'and',
    rules: filterSpec.rules.map((r) => ({
      field: r.field,
      operator: r.operator,
      value: r.value ?? '',
    })),
  });
}

export default function FilterBuilderBar({
  columns = [],
  filterSpec = { logic: 'and', rules: [] },
  onChange,
  onCreateChart = null,
  title = 'Filtros por Coluna',
  compact = false,
}) {
  const normalizedCols = normalizeFilterColumns(columns);
  const rules = Array.isArray(filterSpec?.rules) ? filterSpec.rules : [];
  const logic = filterSpec?.logic === 'or' ? 'or' : 'and';

  const [selectedField, setSelectedField] = useState(normalizedCols[0]?.key || '');
  const [selectedOp, setSelectedOp] = useState('contains');
  const [inputValue, setInputValue] = useState('');

  useEffect(() => {
    if (normalizedCols.length > 0 && !normalizedCols.some((c) => c.key === selectedField)) {
      setSelectedField(normalizedCols[0].key);
    }
  }, [normalizedCols, selectedField]);

  const needsValue = selectedOp !== 'is_empty' && selectedOp !== 'not_empty';

  const handleAddRule = (e) => {
    if (e && e.preventDefault) e.preventDefault();
    const field = selectedField || normalizedCols[0]?.key || '';
    if (!field) return;
    if (needsValue && !inputValue.trim()) return;

    const nextRules = [
      ...rules,
      {
        field,
        operator: selectedOp,
        value: needsValue ? inputValue.trim() : '',
      },
    ];
    onChange?.({ logic, rules: nextRules });
    setInputValue('');
  };

  const handleRemoveRule = (idx) => {
    const nextRules = rules.filter((_, i) => i !== idx);
    onChange?.({ logic, rules: nextRules });
  };

  const handleToggleLogic = () => {
    const nextLogic = logic === 'and' ? 'or' : 'and';
    onChange?.({ logic: nextLogic, rules });
  };

  const handleClearAll = () => {
    onChange?.({ logic: 'and', rules: [] });
  };

  const getColumnLabel = (key) => {
    const found = normalizedCols.find((c) => c.key === key);
    return found ? found.label : key;
  };

  const getOperatorLabel = (op) => {
    const found = FILTER_OPERATORS.find((o) => o.value === op);
    return found ? found.label : op;
  };

  return (
    <div
      data-testid="filter-builder-bar"
      className={`rounded-xl border border-gray-200 bg-gray-50/90 ${compact ? 'p-3' : 'p-3.5'} mb-4 shadow-xs`}
    >
      <div className="flex flex-wrap items-center justify-between gap-2 mb-2.5">
        <div className="flex items-center gap-2">
          <span className="text-xs font-semibold uppercase tracking-wider text-gray-700 flex items-center gap-1.5">
            <span>🔍</span> {title}
          </span>
          {rules.length > 0 && (
            <span
              data-testid="active-filters-count"
              className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-semibold bg-indigo-100 text-indigo-800"
            >
              {rules.length} {rules.length === 1 ? 'filtro ativo' : 'filtros ativos'}
            </span>
          )}
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <button
            type="button"
            data-testid="filter-logic-toggle"
            onClick={handleToggleLogic}
            className="px-2.5 py-1 text-xs font-semibold rounded-lg border border-gray-300 bg-white text-gray-700 hover:bg-gray-100 transition-colors"
            title="Alternar combinação lógica entre todas as regras (E / AND) ou qualquer regra (OU / OR)"
          >
            Combinação: <span className="text-indigo-700 font-bold">{logic === 'and' ? 'E (AND)' : 'OU (OR)'}</span>
          </button>

          {rules.length > 0 && (
            <button
              type="button"
              data-testid="clear-filters-btn"
              onClick={handleClearAll}
              className="px-2.5 py-1 text-xs font-medium rounded-lg border border-red-200 bg-red-50 text-red-700 hover:bg-red-100 transition-colors"
            >
              Limpar filtros
            </button>
          )}

          {onCreateChart && (
            <button
              type="button"
              data-testid="create-chart-from-filters-btn"
              onClick={() => onCreateChart({ logic, rules })}
              className="inline-flex items-center gap-1.5 px-3 py-1 text-xs font-semibold rounded-lg bg-indigo-600 text-white hover:bg-indigo-700 transition-colors shadow-xs"
            >
              <span>📊</span> Criar Gráfico com estes Filtros
            </button>
          )}
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <select
          aria-label="Coluna do filtro"
          data-testid="filter-column-select"
          value={selectedField}
          onChange={(e) => setSelectedField(e.target.value)}
          className="px-2.5 py-1.5 text-xs bg-white border border-gray-300 rounded-lg text-gray-800 focus:outline-none focus:ring-2 focus:ring-indigo-500"
        >
          {normalizedCols.length === 0 ? (
            <option value="">Sem colunas disponíveis</option>
          ) : (
            normalizedCols.map((c) => (
              <option key={c.key} value={c.key}>
                {c.label}
              </option>
            ))
          )}
        </select>

        <select
          aria-label="Operador do filtro"
          data-testid="filter-operator-select"
          value={selectedOp}
          onChange={(e) => setSelectedOp(e.target.value)}
          className="px-2.5 py-1.5 text-xs bg-white border border-gray-300 rounded-lg text-gray-800 focus:outline-none focus:ring-2 focus:ring-indigo-500"
        >
          {FILTER_OPERATORS.map((op) => (
            <option key={op.value} value={op.value}>
              {op.label}
            </option>
          ))}
        </select>

        {needsValue && (
          <input
            type="text"
            aria-label="Valor do filtro"
            data-testid="filter-value-input"
            placeholder="Valor para filtrar..."
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                handleAddRule(e);
              }
            }}
            className="flex-1 min-w-[160px] px-2.5 py-1.5 text-xs bg-white border border-gray-300 rounded-lg text-gray-800 focus:outline-none focus:ring-2 focus:ring-indigo-500"
          />
        )}

        <button
          type="button"
          data-testid="add-filter-rule-btn"
          onClick={handleAddRule}
          disabled={normalizedCols.length === 0 || (needsValue && !inputValue.trim())}
          className="px-3 py-1.5 text-xs font-semibold rounded-lg bg-gray-900 text-white hover:bg-gray-800 disabled:opacity-40 transition-colors"
        >
          + Adicionar Filtro
        </button>
      </div>

      {rules.length > 0 && (
        <div className="flex flex-wrap items-center gap-1.5 mt-2.5 pt-2.5 border-t border-gray-200/80">
          {rules.map((r, idx) => (
            <React.Fragment key={`${r.field}-${r.operator}-${r.value}-${idx}`}>
              {idx > 0 && (
                <span className="text-[10px] font-bold uppercase px-1.5 py-0.5 rounded bg-gray-200 text-gray-700">
                  {logic === 'and' ? 'E' : 'OU'}
                </span>
              )}
              <span
                data-testid={`filter-chip-${idx}`}
                className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs bg-white border border-indigo-200 text-gray-800 shadow-2xs"
              >
                <strong className="font-semibold text-indigo-900">{getColumnLabel(r.field)}</strong>
                <span className="text-gray-500">{getOperatorLabel(r.operator)}</span>
                {r.operator !== 'is_empty' && r.operator !== 'not_empty' && (
                  <span className="font-medium text-gray-900">&ldquo;{r.value}&rdquo;</span>
                )}
                <button
                  type="button"
                  data-testid={`remove-filter-chip-${idx}`}
                  onClick={() => handleRemoveRule(idx)}
                  className="ml-1 text-gray-400 hover:text-red-600 font-bold leading-none"
                  title="Remover filtro"
                >
                  ×
                </button>
              </span>
            </React.Fragment>
          ))}
        </div>
      )}
    </div>
  );
}
