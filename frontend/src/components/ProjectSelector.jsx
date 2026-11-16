import React, { useState, useEffect } from 'react';
import { apiFetch } from '../services/api.js';

export const DEFAULT_PROJECTS = [
  {
    id: 'horizon-v2',
    title: 'Horizon',
    description:
      'Nova versão das tabelas canônicas do DataLake (15 abas de curadoria e enriquecimento SIGPesq)',
    href: '/horizon',
  },
  {
    id: 'horizon',
    title: 'Horizon Legacy',
    description: 'Tabelas canônicas legadas do DataLake (15 abas de curadoria)',
    href: '/dashboard',
  },
  {
    id: 'src',
    title: 'SRC',
    description: 'JSON consolidado do SRC/Ifes (ações de extensão e ensino)',
    href: '/src',
  },
  {
    id: 'egressos',
    title: 'Egressos',
    description:
      'Painel de carreira e renda dos egressos do IFES Campus Serra (21 tabelas JSON da pasta dados/)',
    href: '/egressos',
  },
  {
    id: 'egressos-dados',
    title: 'Egressos — Dados',
    description:
      'Área restrita à organização GitHub ifesserra-lab (acesso exclusivo via GitHub Token)',
    href: '/egressos-dados',
  },
];

/**
 * @param {{
 *   projects?: Array<{ id: string, title: string, description: string, href: string }>,
 *   onSelect?: (project: { id: string, title: string, description: string, href: string }) => void,
 *   pendingWarning?: boolean,
 * }} props
 */
export default function ProjectSelector({
  projects = DEFAULT_PROJECTS,
  statuses = null,
  onSelect,
  pendingWarning = false,
}) {
  const [warnedId, setWarnedId] = useState(null);
  const [liveStatuses, setLiveStatuses] = useState(statuses || {});

  useEffect(() => {
    if (statuses) {
      setLiveStatuses(statuses);
      return;
    }
    let mounted = true;
    apiFetch('/projects/status')
      .then((list) => {
        if (!mounted || !Array.isArray(list)) return;
        const map = {};
        for (const item of list) {
          if (item && item.id) map[item.id] = item;
        }
        setLiveStatuses(map);
      })
      .catch(() => {});
    return () => {
      mounted = false;
    };
  }, [statuses]);

  const go = (project) => {
    if (onSelect) {
      onSelect(project);
    } else {
      window.location.href = project.href;
    }
  };

  const choose = (project) => {
    if (pendingWarning) {
      setWarnedId(project.id);
      const proceed = window.confirm(
        'Há alterações não salvas neste projeto. Trocar de projeto agora pode perder o trabalho em andamento. Continuar?'
      );
      if (!proceed) return;
    }
    setWarnedId(null);
    go(project);
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 max-w-6xl w-full px-4">
      {projects.map((project) => {
        const st = liveStatuses ? liveStatuses[project.id] : null;
        return (
          <button
            key={project.id}
            type="button"
            onClick={() => choose(project)}
            className="text-left bg-white border border-slate-200 rounded-xl shadow-sm hover:shadow-md hover:border-sky-400 transition-all p-6 focus:outline-none focus:ring-2 focus:ring-sky-500 flex flex-col justify-between"
          >
            <div>
              <div className="flex items-center justify-between gap-2 mb-2">
                <h2 className="text-xl font-bold text-slate-900">{project.title}</h2>
                {st && (
                  <span
                    data-testid={`project-card-status-${project.id}`}
                    className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-semibold border ${
                      st.active
                        ? 'bg-emerald-100 text-emerald-800 border-emerald-300'
                        : 'bg-slate-100 text-slate-600 border-slate-200'
                    }`}
                  >
                    <span
                      className={`w-1.5 h-1.5 rounded-full ${
                        st.active ? 'bg-emerald-500' : 'bg-slate-400'
                      }`}
                    />
                    {st.active ? 'Ativo' : 'Inativo'}
                  </span>
                )}
              </div>
              <p className="mt-1 text-sm text-slate-600">{project.description}</p>
              {st && st.status_label && (
                <p className="mt-2 text-xs font-medium text-slate-500">
                  {st.status_label}
                </p>
              )}
            </div>
            <span className="mt-4 inline-block text-sm font-medium text-sky-700">
              Abrir projeto →
            </span>
          </button>
        );
      })}
    </div>
  );
}
