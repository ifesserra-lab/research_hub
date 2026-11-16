import React, { useState } from 'react';

const VALID_ENTITY_ROUTES = new Set([
  'researchers',
  'articles',
  'groups',
  'initiatives',
  'advisorships',
  'awards',
  'students',
  'campuses',
  'organizations',
  'fellowships',
  'proficiencies',
  'professional_activities',
  'knowledge_areas',
  'languages',
  'research_productions',
]);

function formatStructuredValue(val) {
  if (val === null || val === undefined) return '-';
  if (Array.isArray(val)) {
    if (val.length === 0) return '-';
    if (val.every((item) => typeof item !== 'object' || item === null)) {
      return val.join(', ');
    }
    return (
      <ul className="list-disc pl-4 mt-1 space-y-1">
        {val.map((subItem, i) => (
          <li key={i}>{formatStructuredValue(subItem)}</li>
        ))}
      </ul>
    );
  }
  if (typeof val === 'object') {
    return (
      <div className="pl-3 mt-1 border-l-2 border-slate-200 space-y-1">
        {Object.entries(val).map(([subK, subV]) => (
          <div key={subK}>
            <span className="font-medium text-slate-700 capitalize">{subK.replace(/_/g, ' ')}:</span>{' '}
            {formatStructuredValue(subV)}
          </div>
        ))}
      </div>
    );
  }
  return String(val);
}

export default function EntityForm({
  initialData = {},
  fields,
  onSubmit,
  onCancel,
  basePath = '/dashboard',
}) {
  const [data, setData] = useState(initialData);
  const isEditing = initialData.id !== undefined && initialData.id !== null;

  const handleChange = (name, value, type) => {
    let parsedValue = value;
    if (type === 'number' && value !== '') {
      parsedValue = Number(value);
    }
    setData((prev) => ({ ...prev, [name]: parsedValue }));
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    onSubmit(data);
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="p-6 mb-6 bg-white border border-slate-200 rounded-lg shadow-sm"
    >
      <h3 className="mb-4 text-lg font-semibold text-slate-900">
        {initialData.id ? 'Editar' : 'Criar'}
      </h3>
      <div className="space-y-4">
        {fields.map((f) => {
          if (f.type === 'json_readonly') {
            let items = [];
            let parseError = null;
            try {
              const parsed = data[f.name] ? JSON.parse(data[f.name]) : [];
              items = Array.isArray(parsed) ? parsed : parsed ? [parsed] : [];
            } catch (e) {
              parseError = e.message;
            }

            // Map column names (including singular/alias columns) to entity routes
            const getRoute = (colName) => {
              if (
                colName === 'research_groups' ||
                colName === 'research_group' ||
                colName === 'external_research_group'
              ) {
                return 'groups';
              }
              if (colName === 'campus') return 'campuses';
              if (
                colName === 'organizations' ||
                colName === 'organization' ||
                colName === 'demandante' ||
                colName === 'external_partner'
              ) {
                return 'organizations';
              }
              if (
                colName === 'researchers' ||
                colName === 'team' ||
                colName === 'members' ||
                colName === 'leaders'
              ) {
                return 'researchers';
              }
              return colName;
            };
            const route = getRoute(f.name);
            const isRoutableEntity = VALID_ENTITY_ROUTES.has(route);

            return (
              <div key={f.name}>
                <label className="block mb-1 text-sm font-medium text-slate-700">{f.label}</label>
                <div className="p-3 bg-slate-50 border border-slate-200 rounded max-h-48 overflow-y-auto text-sm">
                  {parseError ? (
                    <span className="text-red-600 font-medium">
                      Erro ao ler vínculos: {parseError}
                    </span>
                  ) : items.length > 0 ? (
                    <ul className="list-disc pl-5 space-y-2">
                      {items.map((item, idx) => {
                        if (item.id && isRoutableEntity) {
                          return (
                            <li key={idx}>
                              <a
                                href={`${basePath}/${route}?openId=${item.id}`}
                                className="text-indigo-600 hover:text-indigo-800 hover:underline cursor-pointer font-medium"
                              >
                                ID: {item.id} - {item.name || item.title || 'Sem nome'}
                              </a>
                            </li>
                          );
                        } else {
                          return (
                            <li
                              key={idx}
                              className="text-slate-600 text-xs border border-slate-200 rounded p-2 bg-white"
                            >
                              {Object.entries(item).map(([k, v]) => (
                                <div key={k} className="mb-1 last:mb-0">
                                  <span className="font-semibold text-slate-800 capitalize">
                                    {k.replace(/_/g, ' ')}:
                                  </span>{' '}
                                  {formatStructuredValue(v)}
                                </div>
                              ))}
                            </li>
                          );
                        }
                      })}
                    </ul>
                  ) : (
                    <span className="text-slate-400">Nenhum vínculo.</span>
                  )}
                </div>
              </div>
            );
          }

          // The id is always backend-assigned, continuing the ZIP import's own
          // numbering — it is only ever shown, never hand-entered, on create.
          const idLocked = f.name === 'id' && !isEditing;

          return (
            <div key={f.name}>
              <label className="block mb-1 text-sm font-medium text-slate-700">{f.label}</label>
              <input
                type={f.type || 'text'}
                value={idLocked ? '' : data[f.name] || ''}
                placeholder={idLocked ? 'Automático' : undefined}
                onChange={(e) => handleChange(f.name, e.target.value, f.type)}
                required={f.required && !idLocked}
                readOnly={idLocked}
                disabled={idLocked}
                className="w-full px-3 py-2 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 disabled:bg-slate-100 disabled:text-slate-400"
              />
            </div>
          );
        })}
      </div>
      <div className="flex justify-end mt-6 space-x-3">
        <button
          type="button"
          onClick={onCancel}
          className="px-4 py-2 text-sm font-medium text-slate-700 bg-white border border-slate-300 rounded-md hover:bg-slate-50 focus:outline-none focus:ring-2 focus:ring-indigo-500"
        >
          Cancelar
        </button>
        <button
          type="submit"
          className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500"
        >
          Salvar
        </button>
      </div>
    </form>
  );
}
