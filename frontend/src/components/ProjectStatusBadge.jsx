import React from 'react';

/**
 * Classifica o valor canônico de status de um projeto (`initiatives` e `advisorships`)
 * em Ativo vs. Inativo com rótulo e estilo visual (SEP-040 FR-004 / FR-005).
 */
export function classifyProjectStatus(rawStatus) {
  const trimmed = String(rawStatus ?? '').trim();
  const lower = trimmed.toLowerCase();

  if (lower === 'active' || lower === 'ativo' || lower === 'em execução' || lower === 'em execucao') {
    return {
      rawStatus: trimmed || 'Active',
      isActive: true,
      macroLabel: 'Ativo',
      badgeLabel: 'Ativo (Em Execução)',
      badgeClass: 'bg-emerald-100 text-emerald-800 border-emerald-300',
      dotClass: 'bg-emerald-500',
    };
  }

  if (lower === 'in progress' || lower === 'em andamento' || lower === 'andamento') {
    return {
      rawStatus: trimmed || 'In Progress',
      isActive: true,
      macroLabel: 'Ativo',
      badgeLabel: 'Ativo (Em Andamento)',
      badgeClass: 'bg-teal-100 text-teal-800 border-teal-300',
      dotClass: 'bg-teal-500',
    };
  }

  if (lower === 'concluded' || lower === 'concluído' || lower === 'concluido' || lower === 'encerrado') {
    return {
      rawStatus: trimmed || 'Concluded',
      isActive: false,
      macroLabel: 'Inativo',
      badgeLabel: 'Inativo (Concluído)',
      badgeClass: 'bg-slate-100 text-slate-700 border-slate-300',
      dotClass: 'bg-slate-400',
    };
  }

  if (lower === 'cancelled' || lower === 'canceled' || lower === 'cancelado' || lower === 'suspenso') {
    return {
      rawStatus: trimmed || 'Cancelled',
      isActive: false,
      macroLabel: 'Inativo',
      badgeLabel: 'Inativo (Cancelado)',
      badgeClass: 'bg-rose-100 text-rose-800 border-rose-300',
      dotClass: 'bg-rose-500',
    };
  }

  return {
    rawStatus: trimmed || 'Unknown',
    isActive: false,
    macroLabel: 'Inativo',
    badgeLabel: 'Inativo (Não Informado)',
    badgeClass: 'bg-amber-50 text-amber-800 border-amber-200',
    dotClass: 'bg-amber-400',
  };
}

export default function ProjectStatusBadge({ status, showRaw = false }) {
  const info = classifyProjectStatus(status);
  return (
    <span
      data-testid="project-status-badge"
      data-active={String(info.isActive)}
      className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold border ${info.badgeClass}`}
      title={`Status canônico: ${info.rawStatus}`}
    >
      <span className={`w-1.5 h-1.5 rounded-full ${info.dotClass}`} />
      <span>{info.badgeLabel}</span>
      {showRaw && status ? (
        <span className="font-mono text-[10px] opacity-75">[{status}]</span>
      ) : null}
    </span>
  );
}
