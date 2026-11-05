import React, { useState, useEffect, useCallback } from 'react';
import {
  apiFetch,
  onEgressosDadosImportProgress,
  onSyncProgress,
} from '../services/api.js';

const DEFAULT_LOCAL_REPO_PATH = '/home/thiagodeps/egressos-dados';

export default function EgressosDadosDataControl() {
  const [catalog, setCatalog] = useState(null);
  const [localPath, setLocalPath] = useState(DEFAULT_LOCAL_REPO_PATH);
  const [snapshots, setSnapshots] = useState([]);
  const [busy, setBusy] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [message, setMessage] = useState(null);
  const [progress, setProgress] = useState(null);

  // Sincronização GitHub (`ifesserra-lab/egressos-dados` — US4, FR-010)
  const [syncConfig, setSyncConfig] = useState({
    repo: 'ifesserra-lab/egressos-dados',
    branch: 'main',
    path: '.',
    has_token: true,
    token_hint: null,
  });
  const [sourceUrl, setSourceUrl] = useState(
    'https://github.com/ifesserra-lab/egressos-dados/tree/main'
  );
  const [commitMessage, setCommitMessage] = useState(
    'Export dados Egressos-Dados ResearchHub (39 arquivos JSON)'
  );
  const [syncBusy, setSyncBusy] = useState(false);
  const [syncProgress, setSyncProgress] = useState(null);
  const [syncFeedback, setSyncFeedback] = useState(null);

  const loadAll = useCallback(async () => {
    try {
      const [catData, snapData, cfgData] = await Promise.all([
        apiFetch('/egressos-dados/catalog').catch(() => null),
        apiFetch('/egressos-dados/snapshots').catch(() => []),
        apiFetch('/egressos-dados/sync/config').catch(() => null),
      ]);
      if (catData) {
        setCatalog(catData);
        if (catData.default_local_dir) {
          setLocalPath((prev) => prev || catData.default_local_dir);
        }
      }
      if (Array.isArray(snapData)) {
        setSnapshots(snapData);
      }
      if (cfgData) {
        setSyncConfig({
          repo: cfgData.repo || 'ifesserra-lab/egressos-dados',
          branch: cfgData.branch || 'main',
          path: cfgData.path || '.',
          has_token: Boolean(cfgData.has_token),
          token_hint: cfgData.token_hint || null,
        });
      }
    } catch {
      // ignora silenciosamente durante inicialização
    }
  }, []);

  useEffect(() => {
    loadAll();
    const onImported = () => loadAll();
    window.addEventListener('egressos-dados:imported', onImported);
    document.addEventListener('egressos-dados:imported', onImported);
    return () => {
      window.removeEventListener('egressos-dados:imported', onImported);
      document.removeEventListener('egressos-dados:imported', onImported);
    };
  }, [loadAll]);

  useEffect(() => {
    let cleanupImport = null;
    const p1 = onEgressosDadosImportProgress((payload) => {
      setProgress(payload);
    });
    if (p1 && typeof p1.then === 'function') {
      p1.then((off) => {
        cleanupImport = off;
      });
    }

    let cleanupSync = null;
    const p2 = onSyncProgress((payload) => {
      if (!payload?.project || payload.project === 'egressos_dados') {
        setSyncProgress(payload);
      }
    });
    if (p2 && typeof p2.then === 'function') {
      p2.then((off) => {
        cleanupSync = off;
      });
    }

    return () => {
      if (typeof cleanupImport === 'function') cleanupImport();
      if (typeof cleanupSync === 'function') cleanupSync();
    };
  }, []);

  async function runImport(explicitPath) {
    setBusy(true);
    setMessage(null);
    setProgress(null);
    try {
      const summary = await apiFetch('/egressos-dados/import', {
        method: 'POST',
        body: JSON.stringify({ path: explicitPath ?? null }),
      });
      if (summary === null) {
        setBusy(false);
        return;
      }
      setMessage({
        type: 'success',
        text: `Importação concluída: ${summary.total_files} arquivos JSON importados (${summary.total_rows.toLocaleString('pt-BR')} registros no total).`,
        path: summary.path,
        snapshot: summary.snapshot,
      });
      window.dispatchEvent(
        new CustomEvent('egressos-dados:imported', { detail: summary })
      );
      document.dispatchEvent(
        new CustomEvent('egressos-dados:imported', { detail: summary })
      );
      window.dispatchEvent(
        new CustomEvent('egressos-dados:dirty', { detail: false })
      );
      document.dispatchEvent(
        new CustomEvent('egressos-dados:dirty', { detail: false })
      );
      await loadAll();
    } catch (err) {
      setMessage({ type: 'error', text: err?.message ?? String(err) });
    } finally {
      setBusy(false);
    }
  }

  async function runExport(explicitPath) {
    setExporting(true);
    setMessage(null);
    try {
      const summary = await apiFetch('/egressos-dados/export', {
        method: 'POST',
        body: JSON.stringify({ path: explicitPath ?? null }),
      });
      if (summary === null) {
        setExporting(false);
        return;
      }
      setMessage({
        type: 'success',
        text: `Exportação concluída: ${summary.total_files} arquivos JSON gravados (${summary.total_rows.toLocaleString('pt-BR')} registros).`,
        path: summary.path,
      });
      window.dispatchEvent(
        new CustomEvent('egressos-dados:dirty', { detail: false })
      );
      document.dispatchEvent(
        new CustomEvent('egressos-dados:dirty', { detail: false })
      );
    } catch (err) {
      setMessage({ type: 'error', text: err?.message ?? String(err) });
    } finally {
      setExporting(false);
    }
  }

  async function handleSaveSyncConfig(e) {
    e.preventDefault();
    setSyncFeedback(null);
    try {
      const updated = await apiFetch('/egressos-dados/sync/config', {
        method: 'POST',
        body: JSON.stringify({
          repo: syncConfig.repo.trim(),
          branch: syncConfig.branch.trim(),
          path: syncConfig.path.trim() || '.',
        }),
      });
      if (updated) {
        setSyncConfig((prev) => ({
          ...prev,
          repo: updated.repo,
          branch: updated.branch,
          path: updated.path,
        }));
      }
      setSyncFeedback({
        type: 'success',
        text: 'Destino GitHub atualizado para egressos-dados.',
      });
    } catch (err) {
      setSyncFeedback({
        type: 'error',
        text: err?.message ?? String(err),
      });
    }
  }

  async function handleGitHubDownload() {
    setSyncBusy(true);
    setSyncFeedback(null);
    setSyncProgress(null);
    try {
      const summary = await apiFetch('/egressos-dados/sync/download', {
        method: 'POST',
        body: JSON.stringify({
          source_url: sourceUrl.trim() || null,
          repo: syncConfig.repo.trim(),
          branch: syncConfig.branch.trim(),
          path: syncConfig.path.trim() || '.',
        }),
      });
      setSyncFeedback({
        type: 'success',
        text: `Download do GitHub concluído: ${summary.total_files} arquivos JSON importados (${summary.total_rows.toLocaleString('pt-BR')} registros).`,
        path: summary.path,
        snapshot: summary.snapshot,
      });
      window.dispatchEvent(
        new CustomEvent('egressos-dados:imported', { detail: summary })
      );
      await loadAll();
    } catch (err) {
      setSyncFeedback({
        type: 'error',
        text: err?.message ?? String(err),
      });
    } finally {
      setSyncBusy(false);
    }
  }

  async function handleGitHubUpload() {
    setSyncBusy(true);
    setSyncFeedback(null);
    setSyncProgress(null);
    try {
      const res = await apiFetch('/egressos-dados/sync/upload', {
        method: 'POST',
        body: JSON.stringify({
          confirmOverwrite: true,
          commitMessage: commitMessage.trim() || null,
        }),
      });
      setSyncFeedback({
        type: 'success',
        text: `Publicação concluída no GitHub (${syncConfig.repo}@${syncConfig.branch}) — commit ${res.commit_sha?.slice(0, 7) || res.commit_sha}.`,
        url: res.html_url,
      });
      window.dispatchEvent(
        new CustomEvent('egressos-dados:dirty', { detail: false })
      );
    } catch (err) {
      setSyncFeedback({
        type: 'error',
        text: err?.message ?? String(err),
      });
    } finally {
      setSyncBusy(false);
    }
  }

  return (
    <div className="space-y-6">
      {/* Resumo dinâmico do catálogo carregado (39 datasets) */}
      {catalog && catalog.loaded && (
        <div className="bg-white p-6 rounded-xl shadow-sm border border-slate-200">
          <div className="flex flex-wrap items-center justify-between gap-4 mb-4">
            <div>
              <h2 className="text-lg font-semibold text-slate-800">
                Catálogo carregado ({catalog.total_files} arquivos JSON •{' '}
                {catalog.total_rows.toLocaleString('pt-BR')} registros)
              </h2>
              <p className="text-xs text-slate-500">
                1 arquivo na raiz (<code>alunos.json</code>) + 38 arquivos em{' '}
                <code>data/*.json</code> (12 PII, 6 internos e 21 publicáveis).
              </p>
            </div>
            <a
              href="/egressos-dados"
              className="text-sm font-medium text-emerald-700 hover:text-emerald-800"
            >
              Abrir editor das 39 tabelas →
            </a>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2 max-h-60 overflow-y-auto pr-1">
            {(catalog.files || []).map((f) => (
              <a
                key={f.file_key}
                href={`/egressos-dados?tab=${encodeURIComponent(f.file_key)}`}
                className="flex items-center justify-between px-3 py-2 rounded-lg border border-slate-200 hover:border-emerald-500 hover:bg-emerald-50/40 transition-colors text-xs"
              >
                <div className="truncate pr-2">
                  <div className="font-medium text-slate-800 truncate" title={f.title}>
                    {f.title}
                  </div>
                  <div className="text-[10px] font-mono text-slate-500">
                    {f.relative_path}
                  </div>
                </div>
                <span className="shrink-0 px-2 py-0.5 rounded-full bg-slate-100 text-slate-600 font-mono">
                  {f.total_rows}
                </span>
              </a>
            ))}
          </div>
        </div>
      )}

      {/* Importação e Exportação Local (/home/thiagodeps/egressos-dados) */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-white p-6 rounded-xl shadow-sm border border-slate-200">
          <h2 className="text-lg font-semibold text-slate-800 mb-2">
            Importar Repositório Local (39 arquivos JSON)
          </h2>
          <p className="text-slate-600 mb-4 text-xs leading-relaxed">
            Lê <code>alunos.json</code> na raiz e os 38 arquivos de <code>data/*.json</code>{' '}
            com validação atômica (aceita tanto a raiz{' '}
            <code>/home/thiagodeps/egressos-dados</code> quanto a subpasta{' '}
            <code>/home/thiagodeps/egressos-dados/data</code>). Arquivos brutos auxiliares (
            <code>.txt</code>, <code>.xlsx</code>, <code>.md</code>) permanecem intocados.
          </p>

          <div className="mb-3">
            <label
              htmlFor="egressos-dados-local-path"
              className="block text-xs font-semibold text-slate-700 mb-1"
            >
              Caminho do repositório local
            </label>
            <input
              id="egressos-dados-local-path"
              type="text"
              value={localPath}
              onChange={(e) => setLocalPath(e.target.value)}
              className="w-full border border-slate-300 rounded-lg px-3 py-1.5 text-xs font-mono"
            />
          </div>

          <div className="flex flex-wrap gap-2.5">
            <button
              type="button"
              onClick={() => runImport(localPath.trim() || DEFAULT_LOCAL_REPO_PATH)}
              disabled={busy || exporting}
              className="bg-emerald-600 text-white px-4 py-2 rounded-lg font-medium hover:bg-emerald-700 transition-colors text-xs disabled:opacity-50"
            >
              {busy ? 'Importando 39 JSONs...' : 'Importar caminho informado'}
            </button>
            <button
              type="button"
              onClick={() => runImport(null)}
              disabled={busy || exporting}
              className="border border-slate-300 text-slate-700 px-3.5 py-2 rounded-lg font-medium hover:bg-slate-50 transition-colors text-xs disabled:opacity-50"
            >
              Escolher pasta...
            </button>
          </div>

          {progress && (
            <div className="mt-4 text-xs text-slate-600 bg-slate-50 p-2.5 rounded border border-slate-200">
              Fase: <strong>{progress.phase}</strong> — {progress.files_done ?? 0}/
              {progress.files_total ?? 39} arquivos
              {progress.file_key ? ` (${progress.file_key})` : ''}
            </div>
          )}

          {message && (
            <div
              className={`mt-4 p-3 rounded-lg text-xs border ${
                message.type === 'error'
                  ? 'bg-red-50 text-red-700 border-red-200'
                  : 'bg-emerald-50 text-emerald-800 border-emerald-200'
              }`}
            >
              <div>{message.text}</div>
              {message.path && (
                <div className="mt-1 text-slate-600 break-all">
                  Pasta: <code>{message.path}</code>
                </div>
              )}
              {message.snapshot && (
                <div className="mt-1 text-slate-600 break-all">
                  Snapshot de segurança: <code>{message.snapshot}</code>
                </div>
              )}
            </div>
          )}
        </div>

        <div className="bg-white p-6 rounded-xl shadow-sm border border-slate-200">
          <h2 className="text-lg font-semibold text-slate-800 mb-2">
            Exportar 39 Arquivos JSON para o Disco
          </h2>
          <p className="text-slate-600 mb-4 text-xs leading-relaxed">
            Reconstrói <code>alunos.json</code> na raiz e os 38 arquivos em{' '}
            <code>data/*.json</code> preservando a formatação, metadados de envelope e chaves
            de cada objeto, sem modificar outros arquivos do repositório.
          </p>

          <div className="flex flex-wrap gap-2.5 mt-6">
            <button
              type="button"
              onClick={() => runExport(localPath.trim() || DEFAULT_LOCAL_REPO_PATH)}
              disabled={exporting || busy}
              className="bg-blue-600 text-white px-4 py-2 rounded-lg font-medium hover:bg-blue-700 transition-colors text-xs disabled:opacity-50"
            >
              {exporting ? 'Exportando...' : 'Exportar para o caminho informado'}
            </button>
            <button
              type="button"
              onClick={() => runExport(null)}
              disabled={exporting || busy}
              className="border border-slate-300 text-slate-700 px-3.5 py-2 rounded-lg font-medium hover:bg-slate-50 transition-colors text-xs disabled:opacity-50"
            >
              Escolher pasta de destino...
            </button>
          </div>
        </div>
      </div>

      {/* Sincronização GitHub (ifesserra-lab/egressos-dados — FR-010, Clarificação Q4) */}
      <div className="bg-white p-6 rounded-xl shadow-sm border border-slate-200 space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div>
            <h2 className="text-lg font-semibold text-slate-800">
              Sincronização GitHub — <code>ifesserra-lab/egressos-dados</code>
            </h2>
            <p className="text-xs text-slate-500">
              Reutiliza automaticamente o Token do GitHub validado no login especial da sessão{' '}
              <code>ifesserra-lab</code> (sem redigitar credenciais).
            </p>
          </div>
          <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-mono bg-emerald-50 text-emerald-800 border border-emerald-200">
            Token da sessão ativo {syncConfig.token_hint ? `(…${syncConfig.token_hint})` : ''}
          </span>
        </div>

        <form
          onSubmit={handleSaveSyncConfig}
          className="grid grid-cols-1 md:grid-cols-4 gap-3 items-end"
        >
          <div>
            <label
              htmlFor="sync-repo-input"
              className="block text-xs font-semibold text-slate-700 mb-1"
            >
              Repositório (owner/repo)
            </label>
            <input
              id="sync-repo-input"
              type="text"
              value={syncConfig.repo}
              onChange={(e) =>
                setSyncConfig((prev) => ({ ...prev, repo: e.target.value }))
              }
              className="w-full border border-slate-300 rounded-lg px-3 py-1.5 text-xs font-mono"
            />
          </div>
          <div>
            <label
              htmlFor="sync-branch-input"
              className="block text-xs font-semibold text-slate-700 mb-1"
            >
              Branch
            </label>
            <input
              id="sync-branch-input"
              type="text"
              value={syncConfig.branch}
              onChange={(e) =>
                setSyncConfig((prev) => ({ ...prev, branch: e.target.value }))
              }
              className="w-full border border-slate-300 rounded-lg px-3 py-1.5 text-xs font-mono"
            />
          </div>
          <div>
            <label
              htmlFor="sync-path-input"
              className="block text-xs font-semibold text-slate-700 mb-1"
            >
              Diretório raiz no repositório
            </label>
            <input
              id="sync-path-input"
              type="text"
              value={syncConfig.path}
              onChange={(e) =>
                setSyncConfig((prev) => ({ ...prev, path: e.target.value }))
              }
              className="w-full border border-slate-300 rounded-lg px-3 py-1.5 text-xs font-mono"
            />
          </div>
          <div>
            <button
              type="submit"
              className="w-full border border-slate-300 bg-slate-100 hover:bg-slate-200 text-slate-800 px-3 py-1.5 rounded-lg text-xs font-semibold transition-colors"
            >
              Salvar destino GitHub
            </button>
          </div>
        </form>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2 border-t border-slate-100">
          <div className="space-y-2">
            <label
              htmlFor="sync-source-url"
              className="block text-xs font-semibold text-slate-700"
            >
              Baixar os 39 arquivos JSON do GitHub
            </label>
            <input
              id="sync-source-url"
              type="text"
              value={sourceUrl}
              onChange={(e) => setSourceUrl(e.target.value)}
              placeholder="https://github.com/ifesserra-lab/egressos-dados/tree/main"
              className="w-full border border-slate-300 rounded-lg px-3 py-1.5 text-xs font-mono"
            />
            <button
              type="button"
              disabled={syncBusy}
              onClick={handleGitHubDownload}
              className="bg-slate-900 text-white px-4 py-2 rounded-lg text-xs font-semibold hover:bg-slate-800 disabled:opacity-50"
            >
              {syncBusy ? 'Sincronizando...' : 'Baixar do GitHub (39 JSONs)'}
            </button>
          </div>

          <div className="space-y-2">
            <label
              htmlFor="sync-commit-msg"
              className="block text-xs font-semibold text-slate-700"
            >
              Publicar os 39 arquivos JSON no GitHub (commit único atômico)
            </label>
            <input
              id="sync-commit-msg"
              type="text"
              value={commitMessage}
              onChange={(e) => setCommitMessage(e.target.value)}
              className="w-full border border-slate-300 rounded-lg px-3 py-1.5 text-xs"
            />
            <button
              type="button"
              disabled={syncBusy}
              onClick={handleGitHubUpload}
              className="bg-emerald-600 text-white px-4 py-2 rounded-lg text-xs font-semibold hover:bg-emerald-700 disabled:opacity-50"
            >
              {syncBusy ? 'Enviando...' : 'Publicar 39 JSONs no GitHub'}
            </button>
          </div>
        </div>

        {syncProgress && (
          <div className="text-xs text-slate-600 bg-slate-50 p-2.5 rounded border border-slate-200">
            Operação GitHub ({syncProgress.operation}): fase{' '}
            <strong>{syncProgress.phase}</strong>
          </div>
        )}

        {syncFeedback && (
          <div
            className={`p-3 rounded-lg text-xs border ${
              syncFeedback.type === 'error'
                ? 'bg-red-50 text-red-700 border-red-200'
                : 'bg-emerald-50 text-emerald-800 border-emerald-200'
            }`}
          >
            <div>{syncFeedback.text}</div>
            {syncFeedback.url && (
              <div className="mt-1 font-mono break-all">{syncFeedback.url}</div>
            )}
            {syncFeedback.snapshot && (
              <div className="mt-1 text-slate-600 break-all">
                Snapshot pré-sincronização: <code>{syncFeedback.snapshot}</code>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Lista de Snapshots de Segurança (egressos-dados-*) */}
      <div className="bg-white p-6 rounded-xl shadow-sm border border-slate-200">
        <h3 className="text-sm font-semibold text-slate-800 mb-2">
          Snapshots de Segurança Locais (<code>egressos-dados-*</code>)
        </h3>
        {snapshots.length === 0 ? (
          <p className="text-xs text-slate-500">
            Nenhum snapshot criado ainda. Um snapshot automático é gravado sempre que uma
            reimportação substitui dados já existentes.
          </p>
        ) : (
          <ul className="divide-y divide-slate-100 text-xs">
            {snapshots.map((s) => (
              <li
                key={s.file_name}
                className="py-2 flex flex-wrap items-center justify-between gap-2"
              >
                <code className="font-mono text-slate-800">{s.file_name}</code>
                <span className="text-slate-500 font-mono">
                  {(s.size_bytes / 1024).toFixed(1)} KB
                </span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
