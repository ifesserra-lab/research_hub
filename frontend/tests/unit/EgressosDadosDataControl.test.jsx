/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react';

const apiFetchMock = vi.fn();
vi.mock('../../src/services/api.js', () => ({
  apiFetch: (...args) => apiFetchMock(...args),
  onEgressosDadosImportProgress: vi.fn(() => Promise.resolve(() => {})),
  onSyncProgress: vi.fn(() => Promise.resolve(() => {})),
}));

import EgressosDadosDataControl from '../../src/components/EgressosDadosDataControl.jsx';

describe('EgressosDadosDataControl (US3 & US4 - Local Import/Export, Snapshots & GitHub Sync)', () => {
  beforeEach(() => {
    apiFetchMock.mockReset();
  });

  afterEach(() => {
    cleanup();
  });

  it('imports and exports local repository /home/thiagodeps/egressos-dados and lists snapshots (T018)', async () => {
    apiFetchMock.mockImplementation(async (endpoint, options) => {
      if (endpoint === '/egressos-dados/catalog') {
        return {
          loaded: true,
          total_files: 39,
          total_rows: 2216,
          default_local_dir: '/home/thiagodeps/egressos-dados',
          default_local_dir_exists: true,
          files: [
            {
              file_key: 'alunos',
              title: 'Cadastro Mestre de Alunos (PII)',
              relative_path: 'alunos.json',
              total_rows: 103,
            },
          ],
        };
      }
      if (endpoint === '/egressos-dados/snapshots') {
        return [
          {
            file_name: 'egressos-dados-pre-import-1700000000.db',
            path: '/tmp/snapshots/egressos-dados-pre-import-1700000000.db',
            size_bytes: 204800,
            created_at_secs: 1700000000,
          },
        ];
      }
      if (endpoint === '/egressos-dados/sync/config' && !options?.method) {
        return {
          repo: 'ifesserra-lab/egressos-dados',
          branch: 'main',
          path: '.',
          has_token: true,
          token_hint: '9f2a',
        };
      }
      if (endpoint === '/egressos-dados/import' && options?.method === 'POST') {
        return {
          path: '/home/thiagodeps/egressos-dados',
          total_files: 39,
          total_rows: 2216,
          replaced: true,
          snapshot: 'egressos-dados-pre-import-1700000099.db',
        };
      }
      if (endpoint === '/egressos-dados/export' && options?.method === 'POST') {
        return {
          path: '/home/thiagodeps/egressos-dados',
          total_files: 39,
          total_rows: 2216,
        };
      }
      throw new Error(`Unexpected endpoint: ${endpoint}`);
    });

    render(<EgressosDadosDataControl />);

    await waitFor(() => {
      expect(
        screen.getByText(/egressos-dados-pre-import-1700000000\.db/i)
      ).toBeDefined();
    });

    const pathInput = screen.getByLabelText(/Caminho do repositório local/i);
    expect(pathInput.value).toBe('/home/thiagodeps/egressos-dados');

    // Executa importação local dos 39 JSONs
    fireEvent.click(screen.getByRole('button', { name: /Importar caminho informado/i }));

    await waitFor(() => {
      expect(screen.getByText(/Importação concluída: 39 arquivos JSON importados/i)).toBeDefined();
    });

    expect(apiFetchMock).toHaveBeenCalledWith('/egressos-dados/import', {
      method: 'POST',
      body: JSON.stringify({ path: '/home/thiagodeps/egressos-dados' }),
    });

    // Executa exportação local dos 39 JSONs
    fireEvent.click(screen.getByRole('button', { name: /Exportar para o caminho informado/i }));

    await waitFor(() => {
      expect(screen.getByText(/Exportação concluída: 39 arquivos JSON gravados/i)).toBeDefined();
    });

    expect(apiFetchMock).toHaveBeenCalledWith('/egressos-dados/export', {
      method: 'POST',
      body: JSON.stringify({ path: '/home/thiagodeps/egressos-dados' }),
    });
  });

  it('pre-configures ifesserra-lab/egressos-dados and triggers GitHub download and atomic upload reusing session token (T023)', async () => {
    apiFetchMock.mockImplementation(async (endpoint, options) => {
      if (endpoint === '/egressos-dados/catalog') {
        return { loaded: true, total_files: 39, total_rows: 2216, files: [] };
      }
      if (endpoint === '/egressos-dados/snapshots') {
        return [];
      }
      if (endpoint === '/egressos-dados/sync/config' && !options?.method) {
        return {
          repo: 'ifesserra-lab/egressos-dados',
          branch: 'main',
          path: '.',
          has_token: true,
          token_hint: '9f2a',
        };
      }
      if (endpoint === '/egressos-dados/sync/download' && options?.method === 'POST') {
        return {
          path: 'https://github.com/ifesserra-lab/egressos-dados/tree/main',
          total_files: 39,
          total_rows: 2216,
          replaced: true,
          snapshot: 'egressos-dados-pre-import-1700000123.db',
        };
      }
      if (endpoint === '/egressos-dados/sync/upload' && options?.method === 'POST') {
        return {
          commit_sha: 'a1b2c3d4e5f6',
          html_url: 'https://github.com/ifesserra-lab/egressos-dados/commit/a1b2c3d4e5f6',
          replaced: true,
        };
      }
      throw new Error(`Unexpected endpoint: ${endpoint}`);
    });

    render(<EgressosDadosDataControl />);

    await waitFor(() => {
      expect(screen.getByText(/Token da sessão ativo \(…9f2a\)/i)).toBeDefined();
    });

    expect(screen.getByLabelText(/Repositório \(owner\/repo\)/i).value).toBe(
      'ifesserra-lab/egressos-dados'
    );
    expect(screen.getByLabelText(/^Branch$/i).value).toBe('main');
    expect(screen.getByLabelText(/Diretório raiz no repositório/i).value).toBe('.');

    // Dispara download dos 39 JSONs do GitHub
    fireEvent.click(screen.getByRole('button', { name: /Baixar do GitHub \(39 JSONs\)/i }));

    await waitFor(() => {
      expect(
        screen.getByText(/Download do GitHub concluído: 39 arquivos JSON importados/i)
      ).toBeDefined();
    });

    // Dispara upload atômico dos 39 JSONs para o GitHub
    fireEvent.click(screen.getByRole('button', { name: /Publicar 39 JSONs no GitHub/i }));

    await waitFor(() => {
      expect(
        screen.getByText(/Publicação concluída no GitHub \(ifesserra-lab\/egressos-dados@main\)/i)
      ).toBeDefined();
    });
  });
});
