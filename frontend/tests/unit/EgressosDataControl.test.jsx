/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react';

const mockApiFetch = vi.fn();
const mockOnEgressosImportProgress = vi.fn();

vi.mock('../../src/services/api.js', () => ({
  apiFetch: (...args) => mockApiFetch(...args),
  onEgressosImportProgress: (...args) => mockOnEgressosImportProgress(...args),
}));

import EgressosDataControl from '../../src/components/EgressosDataControl.jsx';

describe('EgressosDataControl', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
    mockOnEgressosImportProgress.mockReset();
    mockOnEgressosImportProgress.mockResolvedValue(() => {});
    mockApiFetch.mockImplementation((endpoint) => {
      if (endpoint === '/egressos/catalog') {
        return Promise.resolve({
          loaded: false,
          imported_at: null,
          total_files: 0,
          total_rows: 0,
          default_local_dir: '/home/thiagodeps/egressos/dados',
          default_local_dir_exists: true,
          files: [],
        });
      }
      return Promise.resolve(null);
    });
  });

  afterEach(() => cleanup());

  it('renders import and export controls', async () => {
    render(<EgressosDataControl />);
    expect(await screen.findByRole('button', { name: /escolher pasta local/i })).toBeDefined();
    expect(screen.getByRole('button', { name: /exportar pasta dados/i })).toBeDefined();
  });

  it('imports a folder and displays summary + snapshot', async () => {
    const listener = vi.fn();
    window.addEventListener('egressos:imported', listener);

    mockApiFetch.mockImplementation((endpoint) => {
      if (endpoint === '/egressos/catalog') {
        return Promise.resolve({
          loaded: true,
          imported_at: '1761156000',
          total_files: 21,
          total_rows: 1142,
          default_local_dir: '/home/thiagodeps/egressos/dados',
          default_local_dir_exists: true,
          files: [
            {
              file_key: 'egressos_perfil',
              file_name: 'egressos_perfil.json',
              title: 'Perfis de Egressos',
              total_rows: 52,
            },
          ],
        });
      }
      if (endpoint === '/egressos/import') {
        return Promise.resolve({
          path: '/home/thiagodeps/egressos/dados',
          total_files: 21,
          total_rows: 1142,
          replaced: true,
          snapshot: 'egressos-pre-import-1761156000.db',
          imported_at: '1761156000',
        });
      }
      return Promise.resolve(null);
    });

    render(<EgressosDataControl />);
    const chooseBtn = await screen.findByRole('button', {
      name: /escolher pasta local/i,
    });
    fireEvent.click(chooseBtn);

    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith('/egressos/import', {
        method: 'POST',
        body: JSON.stringify({ path: null }),
      })
    );

    expect(await screen.findByText(/21 arquivos json importados/i)).toBeDefined();
    expect(screen.getByText(/egressos-pre-import-1761156000\.db/i)).toBeDefined();
    expect(listener).toHaveBeenCalledTimes(1);
    window.removeEventListener('egressos:imported', listener);
  });

  it('opens native folder dialog when choosing another folder (path: null) and handles cancel as no-op', async () => {
    render(<EgressosDataControl />);
    const chooseBtn = await screen.findByRole('button', { name: /escolher pasta local/i });
    fireEvent.click(chooseBtn);

    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith('/egressos/import', {
        method: 'POST',
        body: JSON.stringify({ path: null }),
      })
    );
    expect(screen.queryByText(/arquivos json importados/i)).toBeNull();
  });

  it('exports the 21 JSON files to a local folder and reports the destination path', async () => {
    mockApiFetch.mockImplementation((endpoint) => {
      if (endpoint === '/egressos/catalog') {
        return Promise.resolve({
          loaded: true,
          imported_at: '1761156000',
          total_files: 21,
          total_rows: 1142,
          default_local_dir: '/home/thiagodeps/egressos/dados',
          default_local_dir_exists: true,
          files: [],
        });
      }
      if (endpoint === '/egressos/export') {
        return Promise.resolve({
          path: '/tmp/egressos-export',
          total_files: 21,
          total_rows: 1142,
        });
      }
      return Promise.resolve(null);
    });

    render(<EgressosDataControl />);
    const expBtn = await screen.findByRole('button', { name: /exportar pasta dados/i });
    fireEvent.click(expBtn);

    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith('/egressos/export', {
        method: 'POST',
        body: JSON.stringify({ path: null }),
      })
    );
    expect(await screen.findByText(/\/tmp\/egressos-export/)).toBeDefined();
  });

  it('displays progress events emitted on egressos-import://progress and error messages on failure', async () => {
    let progressCb = null;
    mockOnEgressosImportProgress.mockImplementation((cb) => {
      progressCb = cb;
      return Promise.resolve(() => {});
    });
    mockApiFetch.mockImplementation((endpoint) => {
      if (endpoint === '/egressos/catalog') {
        return Promise.resolve({
          loaded: false,
          total_files: 0,
          total_rows: 0,
          default_local_dir: '/home/thiagodeps/egressos/dados',
          default_local_dir_exists: true,
          files: [],
        });
      }
      if (endpoint === '/egressos/import') {
        if (progressCb) {
          progressCb({
            phase: 'importing',
            file_key: 'vitrine',
            files_done: 2,
            files_total: 21,
          });
        }
        return Promise.reject(
          new Error('Pasta do Egressos incompleta — arquivos JSON obrigatórios ausentes: vitrine.json')
        );
      }
      return Promise.resolve(null);
    });

    render(<EgressosDataControl />);
    const chooseBtn = await screen.findByRole('button', { name: /escolher pasta local/i });
    fireEvent.click(chooseBtn);

    expect(await screen.findByText(/vitrine\.json/i)).toBeDefined();
  });
});
