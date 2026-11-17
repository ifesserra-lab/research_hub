/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, afterEach } from 'vitest';
import React from 'react';
import { render, screen, cleanup } from '@testing-library/react';
import ProjectStatusBadge, { classifyProjectStatus } from '../../src/components/ProjectStatusBadge.jsx';

describe('ProjectStatusBadge (SEP-040 FR-004 / FR-005)', () => {
  afterEach(() => {
    cleanup();
  });

  it('classifies Active and In Progress as active projects', () => {
    const active = classifyProjectStatus('Active');
    expect(active.isActive).toBe(true);
    expect(active.macroLabel).toBe('Ativo');
    expect(active.badgeLabel).toBe('Ativo (Em Execução)');

    const inProg = classifyProjectStatus('In Progress');
    expect(inProg.isActive).toBe(true);
    expect(inProg.macroLabel).toBe('Ativo');
    expect(inProg.badgeLabel).toBe('Ativo (Em Andamento)');
  });

  it('classifies Concluded, Cancelled, Unknown and empty values as inactive projects', () => {
    expect(classifyProjectStatus('Concluded')).toMatchObject({
      isActive: false,
      macroLabel: 'Inativo',
      badgeLabel: 'Inativo (Concluído)',
    });
    expect(classifyProjectStatus('Cancelled')).toMatchObject({
      isActive: false,
      macroLabel: 'Inativo',
      badgeLabel: 'Inativo (Cancelado)',
    });
    expect(classifyProjectStatus('Unknown')).toMatchObject({
      isActive: false,
      macroLabel: 'Inativo',
      badgeLabel: 'Inativo (Não Informado)',
    });
    expect(classifyProjectStatus(null)).toMatchObject({
      isActive: false,
      macroLabel: 'Inativo',
      badgeLabel: 'Inativo (Não Informado)',
    });
  });

  it('renders semantic badge element with data-active attribute', () => {
    render(<ProjectStatusBadge status="Active" />);
    const badge = screen.getByTestId('project-status-badge');
    expect(badge.getAttribute('data-active')).toBe('true');
    expect(badge.textContent).toContain('Ativo (Em Execução)');
  });
});
