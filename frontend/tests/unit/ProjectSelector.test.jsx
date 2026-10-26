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
    id: 'horizon',
    title: 'Horizon',
    description: 'Tabelas canônicas do DataLake (15 abas)',
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
];

describe('ProjectSelector', () => {
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it('renders one distinguishable card per project including Egressos', () => {
    render(<ProjectSelector projects={projects} onSelect={() => {}} />);
    expect(screen.getByRole('heading', { name: 'Horizon' })).toBeDefined();
    expect(screen.getByRole('heading', { name: 'SRC' })).toBeDefined();
    expect(screen.getByRole('heading', { name: 'Egressos' })).toBeDefined();
    expect(screen.getByText(/Tabelas canônicas do DataLake/i)).toBeDefined();
    expect(screen.getByText(/JSON consolidado do SRC/i)).toBeDefined();
    expect(screen.getByText(/21 tabelas JSON da pasta dados/i)).toBeDefined();
  });

  it('includes Egressos in DEFAULT_PROJECTS when rendered without explicit prop', () => {
    render(<ProjectSelector onSelect={() => {}} />);
    expect(screen.getByRole('heading', { name: 'Horizon' })).toBeDefined();
    expect(screen.getByRole('heading', { name: 'SRC' })).toBeDefined();
    expect(screen.getByRole('heading', { name: 'Egressos' })).toBeDefined();
  });

  it('reports the chosen project (including Egressos -> /egressos) to the caller', () => {
    const onSelect = vi.fn();
    render(<ProjectSelector projects={projects} onSelect={onSelect} />);
    fireEvent.click(screen.getByRole('button', { name: /Egressos/i }));
    expect(onSelect).toHaveBeenCalledTimes(1);
    expect(onSelect).toHaveBeenCalledWith(projects[2]);
  });

  it('asks before switching when there are unsaved changes and blocks on cancel', () => {
    const onSelect = vi.fn();
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);
    render(<ProjectSelector projects={projects} onSelect={onSelect} pendingWarning />);
    fireEvent.click(screen.getByRole('button', { name: /Horizon/i }));
    expect(confirmSpy).toHaveBeenCalledTimes(1);
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('proceeds when the unsaved-changes warning is accepted', () => {
    const onSelect = vi.fn();
    vi.spyOn(window, 'confirm').mockReturnValue(true);
    render(<ProjectSelector projects={projects} onSelect={onSelect} pendingWarning />);
    fireEvent.click(screen.getByRole('button', { name: /Horizon/i }));
    expect(onSelect).toHaveBeenCalledWith(projects[0]);
  });

  it('does not ask when there is nothing unsaved', () => {
    const confirmSpy = vi.spyOn(window, 'confirm');
    render(<ProjectSelector projects={projects} onSelect={() => {}} />);
    fireEvent.click(screen.getByRole('button', { name: /SRC/i }));
    expect(confirmSpy).not.toHaveBeenCalled();
  });
});
