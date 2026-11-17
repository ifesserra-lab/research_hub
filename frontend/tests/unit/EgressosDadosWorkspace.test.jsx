/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react';

const apiFetchMock = vi.fn();
vi.mock('../../src/services/api.js', () => ({
  apiFetch: (...args) => apiFetchMock(...args),
}));

import EgressosDadosWorkspace from '../../src/components/EgressosDadosWorkspace.jsx';

describe('EgressosDadosWorkspace (US1 - GitHub Organization Auth Gate)', () => {
  beforeEach(() => {
    apiFetchMock.mockReset();
  });

  afterEach(() => {
    cleanup();
  });

  it('renders the special GitHub Token login screen when unauthorized and hides workspace tables', async () => {
    apiFetchMock.mockImplementation(async (endpoint) => {
      if (endpoint === '/egressos-dados/auth/status') {
        return {
          authorized: false,
          login: null,
          organization: 'ifesserra-lab',
          token_hint: null,
        };
      }
      throw new Error(`Unexpected endpoint: ${endpoint}`);
    });

    render(<EgressosDadosWorkspace />);

    await waitFor(() => {
      expect(
        screen.getByRole('heading', { name: /Acesso Restrito — Organização ifesserra-lab/i })
      ).toBeDefined();
    });

    const tokenInput = screen.getByLabelText(/Token de Acesso Pessoal do GitHub/i);
    expect(tokenInput.getAttribute('type')).toBe('password');
    expect(screen.queryByTestId('egressos-dados-unlocked-panel')).toBeNull();
  });

  it('unlocks the workspace when a valid ifesserra-lab member token is submitted', async () => {
    apiFetchMock.mockImplementation(async (endpoint, options) => {
      if (endpoint === '/egressos-dados/auth/status') {
        return {
          authorized: false,
          login: null,
          organization: 'ifesserra-lab',
          token_hint: null,
        };
      }
      if (endpoint === '/egressos-dados/auth/login' && options?.method === 'POST') {
        return {
          authorized: true,
          login: 'thiagodeps',
          organization: 'ifesserra-lab',
          token_hint: '9f2a',
        };
      }
      if (endpoint === '/egressos-dados/workspace') {
        return {
          organization: 'ifesserra-lab',
          login: 'thiagodeps',
          token_hint: '9f2a',
          tables_ready: false,
        };
      }
      throw new Error(`Unexpected endpoint: ${endpoint}`);
    });

    render(<EgressosDadosWorkspace />);

    await waitFor(() => {
      expect(screen.getByLabelText(/Token de Acesso Pessoal do GitHub/i)).toBeDefined();
    });

    fireEvent.change(screen.getByLabelText(/Token de Acesso Pessoal do GitHub/i), {
      target: { value: 'ghp_123456789f2a' },
    });
    fireEvent.click(screen.getByRole('button', { name: /Validar membro ifesserra-lab e entrar/i }));

    await waitFor(() => {
      expect(screen.getByTestId('egressos-dados-unlocked-panel')).toBeDefined();
    });

    expect(screen.getAllByText(/@thiagodeps/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/ifesserra-lab/i).length).toBeGreaterThan(0);
    expect(screen.getByText(/…9f2a/i)).toBeDefined();
  });

  it('opens directly in unlocked state when returning to the tab during an active authorized session', async () => {
    apiFetchMock.mockImplementation(async (endpoint) => {
      if (endpoint === '/egressos-dados/auth/status') {
        return {
          authorized: true,
          login: 'membro-lab',
          organization: 'ifesserra-lab',
          token_hint: 'abcd',
        };
      }
      if (endpoint === '/egressos-dados/workspace') {
        return {
          organization: 'ifesserra-lab',
          login: 'membro-lab',
          token_hint: 'abcd',
          tables_ready: false,
        };
      }
      throw new Error(`Unexpected endpoint: ${endpoint}`);
    });

    render(<EgressosDadosWorkspace />);

    await waitFor(() => {
      expect(screen.getByTestId('egressos-dados-unlocked-panel')).toBeDefined();
    });

    expect(screen.getAllByText(/@membro-lab/i).length).toBeGreaterThan(0);
    expect(screen.queryByLabelText(/Token de Acesso Pessoal do GitHub/i)).toBeNull();
  });

  it('rejects empty token input with validation message and keeps workspace locked (US2)', async () => {
    apiFetchMock.mockResolvedValue({
      authorized: false,
      login: null,
      organization: 'ifesserra-lab',
      token_hint: null,
    });

    render(<EgressosDadosWorkspace />);

    await waitFor(() => {
      expect(screen.getByLabelText(/Token de Acesso Pessoal do GitHub/i)).toBeDefined();
    });

    fireEvent.click(screen.getByRole('button', { name: /Validar membro ifesserra-lab e entrar/i }));

    expect(screen.getByRole('alert').textContent).toMatch(
      /Informe o Token de Acesso Pessoal do GitHub/i
    );
    expect(screen.queryByTestId('egressos-dados-unlocked-panel')).toBeNull();
  });

  it('displays clear rejection messages for non-members, pending invites, missing read:org scope or invalid tokens (US2)', async () => {
    apiFetchMock.mockImplementation(async (endpoint, options) => {
      if (endpoint === '/egressos-dados/auth/status') {
        return {
          authorized: false,
          login: null,
          organization: 'ifesserra-lab',
          token_hint: null,
        };
      }
      if (endpoint === '/egressos-dados/auth/login' && options?.method === 'POST') {
        const err = new Error(
          "Acesso negado: a conta GitHub '@externo' não pertence à organização 'ifesserra-lab'."
        );
        err.kind = 'org_unauthorized';
        throw err;
      }
      throw new Error(`Unexpected endpoint: ${endpoint}`);
    });

    render(<EgressosDadosWorkspace />);

    await waitFor(() => {
      expect(screen.getByLabelText(/Token de Acesso Pessoal do GitHub/i)).toBeDefined();
    });

    fireEvent.change(screen.getByLabelText(/Token de Acesso Pessoal do GitHub/i), {
      target: { value: 'ghp_token_de_usuario_externo' },
    });
    fireEvent.click(screen.getByRole('button', { name: /Validar membro ifesserra-lab e entrar/i }));

    await waitFor(() => {
      expect(screen.getByRole('alert')).toBeDefined();
    });

    const alertText = screen.getByRole('alert').textContent;
    expect(alertText).toMatch(/não pertence à organização 'ifesserra-lab'/i);
    expect(alertText).toMatch(/org_unauthorized/i);
    expect(alertText).not.toContain('ghp_token_de_usuario_externo');
    expect(screen.queryByTestId('egressos-dados-unlocked-panel')).toBeNull();
  });

  it('locks the tab immediately when the user clicks "Bloquear aba / Trocar token" (US3)', async () => {
    apiFetchMock.mockImplementation(async (endpoint, options) => {
      if (endpoint === '/egressos-dados/auth/status') {
        return {
          authorized: true,
          login: 'thiagodeps',
          organization: 'ifesserra-lab',
          token_hint: '9f2a',
        };
      }
      if (endpoint === '/egressos-dados/workspace') {
        return {
          organization: 'ifesserra-lab',
          login: 'thiagodeps',
          token_hint: '9f2a',
          tables_ready: false,
        };
      }
      if (endpoint === '/egressos-dados/auth/logout' && options?.method === 'DELETE') {
        return {
          authorized: false,
          login: null,
          organization: 'ifesserra-lab',
          token_hint: null,
        };
      }
      throw new Error(`Unexpected endpoint: ${endpoint}`);
    });

    render(<EgressosDadosWorkspace />);

    await waitFor(() => {
      expect(screen.getByTestId('egressos-dados-unlocked-panel')).toBeDefined();
    });

    fireEvent.click(screen.getByRole('button', { name: /Bloquear aba \/ Trocar token/i }));

    await waitFor(() => {
      expect(screen.getByTestId('egressos-dados-locked-gate')).toBeDefined();
    });

    expect(apiFetchMock).toHaveBeenCalledWith('/egressos-dados/auth/logout', {
      method: 'DELETE',
    });
    expect(screen.queryByTestId('egressos-dados-unlocked-panel')).toBeNull();
  });
});
