import React, { useState } from 'react';

export const DEFAULT_PROJECTS = [
  {
    id: 'horizon',
    title: 'Horizon',
    description: 'Tabelas canônicas do DataLake (15 abas de curadoria)',
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
  onSelect,
  pendingWarning = false,
}) {
  const [warnedId, setWarnedId] = useState(null);

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
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-5xl w-full px-4">
      {projects.map((project) => (
        <button
          key={project.id}
          type="button"
          onClick={() => choose(project)}
          className="text-left bg-white border border-slate-200 rounded-xl shadow-sm hover:shadow-md hover:border-sky-400 transition-all p-6 focus:outline-none focus:ring-2 focus:ring-sky-500 flex flex-col justify-between"
        >
          <div>
            <h2 className="text-xl font-bold text-slate-900">{project.title}</h2>
            <p className="mt-2 text-sm text-slate-600">{project.description}</p>
          </div>
          <span className="mt-4 inline-block text-sm font-medium text-sky-700">
            Abrir projeto →
          </span>
        </button>
      ))}
    </div>
  );
}
