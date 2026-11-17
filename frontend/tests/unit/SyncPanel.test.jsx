/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, waitFor, cleanup, act } from '@testing-library/react';

const mockApiFetch = vi.fn();
let progressCallback = null;
const mockOnSyncProgress = vi.fn((cb) => {
  progressCallback = cb;
  return Promise.resolve(() => {});
});

vi.mock('../../src/services/api', () => ({
  apiFetch: (...args) => mockApiFetch(...args),
  onSyncProgress: (...args) => mockOnSyncProgress(...args),
}));

import SyncPanel from '../../src/components/SyncPanel.jsx';

describe('SyncPanel (Download - US1)', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
    mockOnSyncProgress.mockClear();
    progressCallback = null;
  });

  afterEach(() => cleanup());

  it('loads config and pre-fills URL input with last_source_url', async () => {
    mockApiFetch.mockResolvedValueOnce({
      repo: 'owner/repo',
      branch: 'main',
      path: 'exports_canonical.zip',
      last_source_url: 'https://raw.githubusercontent.com/owner/repo/main/exports_canonical.zip',
      has_token: true,
      token_hint: 'abcd',
    });

    render(<SyncPanel project="horizon" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    const input = screen.getByRole('textbox', { name: /url do arquivo/i });
    expect(input.value).toBe('https://raw.githubusercontent.com/owner/repo/main/exports_canonical.zip');
    expect(mockOnSyncProgress).toHaveBeenCalled();
  });

  it('triggers download and displays summary on success', async () => {
    mockApiFetch
      .mockResolvedValueOnce({
        repo: '',
        branch: '',
        path: '',
        last_source_url: null,
        has_token: false,
        token_hint: null,
      })
      .mockResolvedValueOnce({
        total_rows: 150,
        tables: [{ table: 'campuses', rows: 2 }],
        snapshot: '/dados/snapshots/hub-123.db',
      });

    render(<SyncPanel project="horizon" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    const input = screen.getByRole('textbox', { name: /url do arquivo/i });
    fireEvent.change(input, {
      target: { value: 'https://raw.githubusercontent.com/a/b/main/exports.zip' },
    });

    const button = screen.getByRole('button', { name: /baixar e importar/i });
    fireEvent.click(button);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/download', {
        method: 'POST',
        body: JSON.stringify({
          project: 'horizon',
          url: 'https://raw.githubusercontent.com/a/b/main/exports.zip',
        }),
      });
    });

    await waitFor(() => {
      expect(screen.getByText(/150 registros/i)).toBeDefined();
    });
    expect(screen.getByText(/hub-123\.db/i)).toBeDefined();
  });

  it('displays progress updates from onSyncProgress', async () => {
    mockApiFetch
      .mockResolvedValueOnce({
        repo: '',
        branch: '',
        path: '',
        last_source_url: null,
        has_token: false,
        token_hint: null,
      })
      .mockImplementationOnce(() => new Promise(() => {})); // stays pending

    render(<SyncPanel project="horizon" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    const input = screen.getByRole('textbox', { name: /url do arquivo/i });
    fireEvent.change(input, {
      target: { value: 'https://raw.githubusercontent.com/a/b/main/exports.zip' },
    });

    fireEvent.click(screen.getByRole('button', { name: /baixar e importar/i }));

    act(() => {
      if (progressCallback) {
        progressCallback({
          operation: 'download',
          project: 'horizon',
          phase: 'transferring',
          bytes_done: 1024,
          bytes_total: 2048,
        });
      }
    });

    expect(screen.getByText(/transferindo/i)).toBeDefined();
  });

  it('displays typed errors with kind and message', async () => {
    const error = new Error('URL exige acesso autenticado; configure o token');
    error.kind = 'github_api';

    mockApiFetch
      .mockResolvedValueOnce({
        repo: '',
        branch: '',
        path: '',
        last_source_url: null,
        has_token: false,
        token_hint: null,
      })
      .mockRejectedValueOnce(error);

    render(<SyncPanel project="horizon" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    const input = screen.getByRole('textbox', { name: /url do arquivo/i });
    fireEvent.change(input, {
      target: { value: 'https://github.com/privado/repo/blob/main/exports.zip' },
    });

    fireEvent.click(screen.getByRole('button', { name: /baixar e importar/i }));

    await waitFor(() => {
      expect(screen.getByText(/URL exige acesso autenticado/i)).toBeDefined();
    });
    expect(screen.getByText(/github_api/i)).toBeDefined();
  });

  it('does not render upload controls when allowUpload is false', async () => {
    mockApiFetch.mockResolvedValueOnce({
      repo: 'a/b',
      branch: 'main',
      path: 'x.zip',
      last_source_url: null,
      has_token: false,
      token_hint: null,
    });

    render(<SyncPanel project="src" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalled();
    });

    expect(screen.queryByRole('button', { name: /enviar para o github/i })).toBeNull();
  });
});

describe('SyncPanel (Upload - US2)', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
    mockOnSyncProgress.mockClear();
    progressCallback = null;
  });

  afterEach(() => cleanup());

  it('renders destination fields and privacy notice when allowUpload is true', async () => {
    mockApiFetch.mockResolvedValueOnce({
      repo: 'owner/repo',
      branch: 'main',
      path: 'exports/exports_canonical.zip',
      last_source_url: null,
      has_token: true,
      token_hint: 'abcd',
    });

    render(<SyncPanel project="horizon" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    expect(screen.getByRole('textbox', { name: /repositório/i })).toBeDefined();
    expect(screen.getByRole('textbox', { name: /^branch$/i })).toBeDefined();
    expect(screen.getByRole('textbox', { name: /caminho do arquivo/i })).toBeDefined();
    expect(screen.getByRole('button', { name: /enviar para o github/i })).toBeDefined();
    expect(screen.getByText(/dados pessoais|visibilidade|confidenciais/i)).toBeDefined();
  });

  it('validates owner/name repo format before saving or uploading', async () => {
    mockApiFetch.mockResolvedValueOnce({
      repo: '',
      branch: 'main',
      path: 'exports.zip',
      last_source_url: null,
      has_token: true,
      token_hint: 'abcd',
    });

    render(<SyncPanel project="horizon" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalled();
    });

    const repoInput = screen.getByRole('textbox', { name: /repositório/i });
    fireEvent.change(repoInput, { target: { value: 'invalid_repo_without_slash' } });

    fireEvent.click(screen.getByRole('button', { name: /salvar destino/i }));

    expect(screen.getByText(/owner\/name/i)).toBeDefined();
    expect(mockApiFetch).not.toHaveBeenCalledWith('/github/config', expect.anything());
  });

  it('uploads in default "Criar nova branch" mode and adds the created branch to the selector', async () => {
    mockApiFetch
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'main',
        path: 'exports/exports_canonical.zip',
        last_source_url: null,
        has_token: true,
        token_hint: 'abcd',
      })
      // POST /github/upload
      .mockResolvedValueOnce({
        commit_sha: '1234567890abcdef',
        html_url: 'https://github.com/owner/repo/commit/1234567890abcdef',
        replaced: false,
        branch: 'update/horizon-custom',
        created_branch: true,
      });

    render(<SyncPanel project="horizon" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    const createRadio = screen.getByRole('radio', {
      name: /criar nova branch com os novos dados/i,
    });
    expect(createRadio.checked).toBe(true);

    const newBranchInput = screen.getByRole('textbox', { name: /nome da nova branch/i });
    expect(newBranchInput.value).toMatch(/^update\/horizon-/);
    fireEvent.change(newBranchInput, { target: { value: 'update/horizon-custom' } });

    fireEvent.click(screen.getByRole('button', { name: /enviar para o github/i }));

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/upload', {
        method: 'POST',
        body: JSON.stringify({
          project: 'horizon',
          confirmOverwrite: false,
          createNewBranch: true,
          baseBranch: 'main',
          newBranch: 'update/horizon-custom',
          targetBranch: null,
        }),
      });
    });

    await waitFor(() => {
      expect(screen.getByText(/Nova branch criada: update\/horizon-custom/i)).toBeDefined();
    });
    const link = screen.getByRole('link', { name: /1234567/i });
    expect(link.getAttribute('href')).toBe('https://github.com/owner/repo/commit/1234567890abcdef');

    // A branch base configurada continua sendo 'main' (FR-005), e a nova branch entra no seletor
    expect(screen.getByRole('textbox', { name: /^branch$/i }).value).toBe('main');
    const branchSelect = screen.getByRole('combobox', { name: /selecionar branch existente/i });
    expect(Array.from(branchSelect.options).map((o) => o.value)).toContain(
      'update/horizon-custom'
    );
  });

  it('loads remote branches beyond main and uploads in "Enviar na branch selecionada" mode with overwrite check', async () => {
    window.confirm = vi.fn().mockReturnValue(true);

    mockApiFetch
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'main',
        path: 'exports/exports_canonical.zip',
        last_source_url: null,
        has_token: true,
        token_hint: 'abcd',
      })
      // GET /github/branches
      .mockResolvedValueOnce([
        { name: 'main', commit_sha: '111' },
        { name: 'develop', commit_sha: '222' },
        { name: 'homologacao', commit_sha: '333' },
      ])
      // GET /github/check
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'develop',
        path: 'exports/exports_canonical.zip',
        branch_exists: true,
        file_exists: true,
        file_sha: 'old_sha_123',
      })
      // POST /github/upload with confirmOverwrite: true
      .mockResolvedValueOnce({
        commit_sha: 'new_commit_sha',
        html_url: 'https://github.com/owner/repo/commit/new_commit_sha',
        replaced: true,
        branch: 'develop',
        created_branch: false,
      });

    render(<SyncPanel project="horizon" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    // Carrega branches remotas além da main (US2)
    fireEvent.click(screen.getByRole('button', { name: /carregar branches/i }));
    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith(
        '/github/branches?project=horizon&repo=owner%2Frepo'
      );
    });

    const branchSelect = screen.getByRole('combobox', { name: /selecionar branch existente/i });
    fireEvent.change(branchSelect, { target: { value: 'develop' } });
    expect(screen.getByRole('textbox', { name: /^branch$/i }).value).toBe('develop');

    // Alterna para "Enviar na branch selecionada"
    fireEvent.click(screen.getByRole('radio', { name: /enviar na branch selecionada/i }));

    fireEvent.click(screen.getByRole('button', { name: /enviar para o github/i }));

    await waitFor(() => {
      expect(window.confirm).toHaveBeenCalledWith(
        expect.stringMatching(/já existe|sobrescrever/i),
      );
    });

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/upload', {
        method: 'POST',
        body: JSON.stringify({
          project: 'horizon',
          confirmOverwrite: true,
          createNewBranch: false,
          baseBranch: 'develop',
          newBranch: null,
          targetBranch: 'develop',
        }),
      });
    });
  });

  it('does not upload if user declines overwrite confirmation in existing branch mode', async () => {
    window.confirm = vi.fn().mockReturnValue(false);

    mockApiFetch
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'main',
        path: 'exports/exports_canonical.zip',
        last_source_url: null,
        has_token: true,
        token_hint: 'abcd',
      })
      // GET /github/check
      .mockResolvedValueOnce({
        repo: 'owner/repo',
        branch: 'main',
        path: 'exports/exports_canonical.zip',
        branch_exists: true,
        file_exists: true,
        file_sha: 'old_sha_123',
      });

    render(<SyncPanel project="horizon" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=horizon');
    });

    fireEvent.click(screen.getByRole('radio', { name: /enviar na branch selecionada/i }));
    fireEvent.click(screen.getByRole('button', { name: /enviar para o github/i }));

    await waitFor(() => {
      expect(window.confirm).toHaveBeenCalled();
    });

    expect(mockApiFetch).not.toHaveBeenCalledWith('/github/upload', expect.anything());
  });

  it('enables SRC upload with .json path placeholder and displays conflict error if branch exists (US3)', async () => {
    const error = new Error("A branch 'update/src-dup' já existe no repositório");
    error.kind = 'conflict';

    mockApiFetch
      .mockResolvedValueOnce({
        repo: 'owner/src-repo',
        branch: 'main',
        path: 'dados/src_consolidado.json',
        last_source_url: null,
        has_token: true,
        token_hint: 'abcd',
      })
      .mockRejectedValueOnce(error);

    render(<SyncPanel project="src" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=src');
    });

    const pathInput = screen.getByRole('textbox', { name: /caminho do arquivo/i });
    expect(pathInput.getAttribute('placeholder')).toBe('dados/src_consolidado.json');

    fireEvent.click(screen.getByRole('button', { name: /enviar para o github/i }));

    await waitFor(() => {
      expect(screen.getByText(/já existe no repositório/i)).toBeDefined();
    });
    expect(screen.getByText(/conflict/i)).toBeDefined();
  });

  it('refreshes branches to detect newly created branches on GitHub after sync and updates download URL on selection', async () => {
    mockApiFetch
      .mockResolvedValueOnce({
        repo: 'owner/src-repo',
        branch: 'main',
        path: 'dados/src_consolidado.json',
        last_source_url: 'https://raw.githubusercontent.com/owner/src-repo/main/dados/src_consolidado.json',
        has_token: true,
        token_hint: 'abcd',
      })
      // Refresh 1: detecta duas novas branches criadas no GitHub
      .mockResolvedValueOnce([
        { name: 'main', commit_sha: '1111111aaaa' },
        { name: 'update/src-20261105-150000', commit_sha: '9999999bbbb' },
      ]);

    render(<SyncPanel project="src" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=src');
    });

    const refreshBtn = screen.getByRole('button', { name: /atualizar branches/i });
    fireEvent.click(refreshBtn);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith(
        '/github/branches?project=src&repo=owner%2Fsrc-repo'
      );
    });

    await waitFor(() => {
      expect(
        screen.getByText(/Novas branches detectadas: update\/src-20261105-150000/i)
      ).toBeDefined();
    });

    const detectedBox = screen.getByTestId('newly-detected-branches');
    expect(detectedBox.textContent).toContain('update/src-20261105-150000 (9999999)');

    // Clicar no chip da branch recém-detectada seleciona a branch e atualiza a URL de download
    const chipBtn = screen.getByRole('button', {
      name: /update\/src-20261105-150000 \(9999999\)/i,
    });
    fireEvent.click(chipBtn);

    expect(screen.getByRole('textbox', { name: /^branch$/i }).value).toBe(
      'update/src-20261105-150000'
    );
    expect(screen.getByRole('textbox', { name: /url do arquivo/i }).value).toBe(
      'https://raw.githubusercontent.com/owner/src-repo/update/src-20261105-150000/dados/src_consolidado.json'
    );
  });
});

describe('SyncPanel (Token integration - US3)', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
    mockOnSyncProgress.mockClear();
    progressCallback = null;
  });

  afterEach(() => cleanup());

  it('renders "Token de acesso…" button and opens TokenDialog on click', async () => {
    mockApiFetch.mockResolvedValueOnce({
      repo: '',
      branch: 'main',
      path: '',
      last_source_url: null,
      has_token: true,
      token_hint: '9876',
    });

    render(<SyncPanel project="horizon" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalled();
    });

    const tokenButton = screen.getByRole('button', { name: /^token de acesso/i });
    fireEvent.click(tokenButton);

    expect(screen.getByRole('dialog')).toBeDefined();
    expect(screen.getAllByText(/9876/).length).toBeGreaterThan(0);
  });

  it('shows token configure action on authentication error', async () => {
    const error = new Error('URL exige acesso autenticado; configure o token');
    error.kind = 'github_api';

    mockApiFetch
      .mockResolvedValueOnce({
        repo: '',
        branch: '',
        path: '',
        last_source_url: null,
        has_token: false,
        token_hint: null,
      })
      .mockRejectedValueOnce(error);

    render(<SyncPanel project="horizon" allowUpload={false} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalled();
    });

    const input = screen.getByRole('textbox', { name: /url do arquivo/i });
    fireEvent.change(input, {
      target: { value: 'https://github.com/privado/repo/blob/main/exports.zip' },
    });

    fireEvent.click(screen.getByRole('button', { name: /baixar e importar/i }));

    await waitFor(() => {
      expect(screen.getByText(/exige acesso autenticado/i)).toBeDefined();
    });

    const configButton = screen.getByRole('button', { name: /configurar token/i });
    fireEvent.click(configButton);
    expect(screen.getByRole('dialog')).toBeDefined();
  });
});

// SEP-035: Egressos GitHub sync (T031)
describe('SyncPanel (Egressos - US4)', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
    mockOnSyncProgress.mockClear();
    progressCallback = null;
  });

  afterEach(() => cleanup());

  it('starts with empty fields when no config is saved and downloads + uploads the 21 JSON files', async () => {
    const importedListener = vi.fn();
    window.addEventListener('egressos:imported', importedListener);
    window.confirm = vi.fn(() => true);

    mockApiFetch
      // 1. GET /github/config?project=egressos (empty when not configured)
      .mockResolvedValueOnce({
        repo: '',
        branch: '',
        path: '',
        last_source_url: null,
        has_token: false,
        token_hint: null,
      })
      // 2. POST /github/download
      .mockResolvedValueOnce({
        path: 'https://github.com/owner/egressos/tree/main/dados',
        total_files: 21,
        total_rows: 1142,
        snapshot: 'egressos-pre-import-1761156000.db',
      })
      // 3. GET /github/check?project=egressos
      .mockResolvedValueOnce({
        repo: 'owner/egressos',
        branch: 'main',
        path: 'dados',
        branch_exists: true,
        file_exists: true,
        file_sha: 'dir_sha_1',
      })
      // 4. POST /github/upload
      .mockResolvedValueOnce({
        commit_sha: 'egressos_commit_777',
        html_url: 'https://github.com/owner/egressos/commit/egressos_commit_777',
        replaced: true,
      });

    render(<SyncPanel project="egressos" allowUpload={true} />);

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/config?project=egressos');
    });

    const urlInput = screen.getByRole('textbox', { name: /url do arquivo/i });
    const repoInput = screen.getByRole('textbox', { name: /repositório/i });
    const branchInput = screen.getByRole('textbox', { name: /^branch$/i });
    const pathInput = screen.getByRole('textbox', { name: /caminho/i });

    expect(urlInput.value).toBe('');
    expect(repoInput.value).toBe('');
    expect(branchInput.value).toBe('main');
    expect(pathInput.value).toBe('');

    fireEvent.change(urlInput, {
      target: { value: 'https://github.com/owner/egressos/tree/main/dados' },
    });
    fireEvent.change(repoInput, { target: { value: 'owner/egressos' } });
    fireEvent.change(pathInput, { target: { value: 'dados' } });

    // Trigger download of the 21 JSON files
    fireEvent.click(screen.getByRole('button', { name: /baixar e importar/i }));

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/download', {
        method: 'POST',
        body: JSON.stringify({
          project: 'egressos',
          url: 'https://github.com/owner/egressos/tree/main/dados',
        }),
      });
    });

    expect(await screen.findByText(/21 arquivos json e 1\.142 registros/i)).toBeDefined();
    expect(importedListener).toHaveBeenCalledTimes(1);

    // Switch to existing branch mode to test overwrite check on Egressos
    fireEvent.click(screen.getByRole('radio', { name: /enviar na branch selecionada/i }));
    fireEvent.click(screen.getByRole('button', { name: /enviar para o github/i }));

    await waitFor(() => {
      expect(mockApiFetch).toHaveBeenCalledWith('/github/check?project=egressos');
      expect(mockApiFetch).toHaveBeenCalledWith('/github/upload', {
        method: 'POST',
        body: JSON.stringify({
          project: 'egressos',
          confirmOverwrite: true,
          createNewBranch: false,
          baseBranch: 'main',
          newBranch: null,
          targetBranch: 'main',
        }),
      });
    });

    expect(await screen.findByText(/enviado com sucesso/i)).toBeDefined();
    window.removeEventListener('egressos:imported', importedListener);
  });
});


