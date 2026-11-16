/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, cleanup } from '@testing-library/react';

import ProjectSelector from '../../src/components/ProjectSelector.jsx';

// SEP-033 / US1: dois projetos distinguíveis, seleção reportada ao chamador e
// aviso obrigatório antes de trocar com edição não salva (spec US1-5, FR-001..003).

const projects = [
  {
    id: 'horizon-v2',
    title: 'Horizon',
    description: 'Nova versão das tabelas canônicas do DataLake (15 abas e enriquecimento SIGPesq)',
    href: '/horizon',
  },
  {
    id: 'horizon',
    title: 'Horizon Legacy',
    description: 'Tabelas canônicas legadas do DataLake (15 abas)',
    href: '/dashboard',
  },
  {
    id: 'src',
    title: 'SRC',
    description: 'JSON consolidado do SRC/Ifes (extensão e ensino)',
    href: '/src',
  },
  {
    id: 'egressos',
    title: 'Egressos',
    description: 'Painel de carreira e renda dos egressos do IFES Campus Serra (21 tabelas JSON da pasta dados/)',
    href: '/egressos',
  },
  {
    id: 'egressos-dados',
    title: 'Egressos — Dados',
    description: 'Área restrita à organização GitHub ifesserra-lab (acesso exclusivo via GitHub Token)',
    href: '/egressos-dados',
  },
];

describe('ProjectSelector', () => {
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it('renders one distinguishable card per project including Horizon, Horizon Legacy, SRC, Egressos and Egressos — Dados', () => {
    render(<ProjectSelector projects={projects} onSelect={() => {}} />);
    expect(screen.getByRole('heading', { name: 'Horizon' })).toBeDefined();
    expect(screen.getByRole('heading', { name: 'Horizon Legacy' })).toBeDefined();
    expect(screen.getByRole('heading', { name: 'SRC' })).toBeDefined();
    expect(screen.getByRole('heading', { name: 'Egressos' })).toBeDefined();
    expect(screen.getByRole('heading', { name: 'Egressos — Dados' })).toBeDefined();
    expect(screen.getByText(/Nova versão das tabelas canônicas/i)).toBeDefined();
    expect(screen.getByText(/Tabelas canônicas legadas/i)).toBeDefined();
    expect(screen.getByText(/JSON consolidado do SRC/i)).toBeDefined();
    expect(screen.getByText(/21 tabelas JSON da pasta dados/i)).toBeDefined();
    expect(screen.getByText(/ifesserra-lab/i)).toBeDefined();
  });

  it('includes Horizon (/horizon), Horizon Legacy (/dashboard), SRC, Egressos and Egressos — Dados in DEFAULT_PROJECTS when rendered without explicit prop', () => {
    const onSelect = vi.fn();
    render(<ProjectSelector onSelect={onSelect} />);
    expect(screen.getByRole('heading', { name: 'Horizon' })).toBeDefined();
    expect(screen.getByRole('heading', { name: 'Horizon Legacy' })).toBeDefined();
    expect(screen.getByRole('heading', { name: 'SRC' })).toBeDefined();
    expect(screen.getByRole('heading', { name: 'Egressos' })).toBeDefined();
    expect(screen.getByRole('heading', { name: 'Egressos — Dados' })).toBeDefined();

    fireEvent.click(screen.getByRole('heading', { name: 'Horizon' }).closest('button'));
    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ id: 'horizon-v2', title: 'Horizon', href: '/horizon' }));

    fireEvent.click(screen.getByRole('heading', { name: 'Horizon Legacy' }).closest('button'));
    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ id: 'horizon', title: 'Horizon Legacy', href: '/dashboard' }));
  });

  it('reports the chosen project (including Egressos -> /egressos and Egressos — Dados -> /egressos-dados) to the caller', () => {
    const onSelect = vi.fn();
    render(<ProjectSelector projects={projects} onSelect={onSelect} />);
    fireEvent.click(screen.getByRole('button', { name: /Egressos — Dados/i }));
    expect(onSelect).toHaveBeenCalledTimes(1);
    expect(onSelect).toHaveBeenCalledWith(projects[4]);
  });

  it('asks before switching when there are unsaved changes and blocks on cancel', () => {
    const onSelect = vi.fn();
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);
    render(<ProjectSelector projects={projects} onSelect={onSelect} pendingWarning />);
    fireEvent.click(screen.getByRole('heading', { name: 'Horizon Legacy' }).closest('button'));
    expect(confirmSpy).toHaveBeenCalledTimes(1);
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('proceeds when the unsaved-changes warning is accepted', () => {
    const onSelect = vi.fn();
    vi.spyOn(window, 'confirm').mockReturnValue(true);
    render(<ProjectSelector projects={projects} onSelect={onSelect} pendingWarning />);
    fireEvent.click(screen.getByRole('heading', { name: 'Horizon' }).closest('button'));
    expect(onSelect).toHaveBeenCalledWith(projects[0]);
  });

  it('does not ask when there is nothing unsaved', () => {
    const confirmSpy = vi.spyOn(window, 'confirm');
    render(<ProjectSelector projects={projects} onSelect={() => {}} />);
    fireEvent.click(screen.getByRole('button', { name: /SRC/i }));
    expect(confirmSpy).not.toHaveBeenCalled();
  });

  it('displays active/inactive status badge on each project card when statuses are provided (SEP-040 FR-004)', () => {
    const statuses = {
      'horizon-v2': {
        id: 'horizon-v2',
        active: true,
        total_rows: 70545,
        active_projects: 1943,
        status_label: 'Ativo • 70.545 registros (1.943 projetos ativos)',
      },
      horizon: {
        id: 'horizon',
        active: false,
        total_rows: 0,
        active_projects: 0,
        status_label: 'Inativo • Sem dados carregados',
      },
    };
    render(<ProjectSelector projects={projects} statuses={statuses} onSelect={() => {}} />);
    expect(screen.getByText(/Ativo • 70\.545 registros/i)).toBeDefined();
    expect(screen.getByText(/Inativo • Sem dados carregados/i)).toBeDefined();
  });
});
