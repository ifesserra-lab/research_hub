import React, { useState, useEffect } from 'react';
import { apiFetch, onSyncProgress } from '../services/api.js';
import TokenDialog from './TokenDialog.jsx';

export function buildSuggestedBranchName(project) {
  const d = new Date();
  const pad = (n) => String(n).padStart(2, '0');
  const stamp = `${d.getUTCFullYear()}${pad(d.getUTCMonth() + 1)}${pad(d.getUTCDate())}-${pad(
    d.getUTCHours()
  )}${pad(d.getUTCMinutes())}${pad(d.getUTCSeconds())}`;
  return `update/${project || 'dados'}-${stamp}`;
}

export function extractRepoFromGitHubUrl(rawUrl) {
  if (!rawUrl || typeof rawUrl !== 'string') return '';
  try {
    const parsed = new URL(rawUrl.trim());
    if (
      parsed.hostname !== 'github.com' &&
      parsed.hostname !== 'raw.githubusercontent.com'
    ) {
      return '';
    }
    const parts = parsed.pathname.split('/').filter(Boolean);
    if (parts.length >= 2) {
      return `${parts[0]}/${parts[1].replace(/\.git$/, '')}`;
    }
  } catch {
    // ignore invalid URL
  }
  return '';
}

export function replaceBranchInGitHubUrl(rawUrl, nextBranch) {
  if (!rawUrl || typeof rawUrl !== 'string' || !nextBranch) return rawUrl;
  try {
    const parsed = new URL(rawUrl.trim());
    const parts = parsed.pathname.split('/').filter(Boolean);
    const isNamespaced = (seg) =>
      ['update', 'feature', 'fix', 'hotfix', 'release'].includes(seg);

    if (parsed.hostname === 'raw.githubusercontent.com' && parts.length >= 4) {
      const owner = parts[0];
      const repo = parts[1];
      const pathStart = isNamespaced(parts[2]) && parts.length >= 5 ? 4 : 3;
      const restPath = parts.slice(pathStart).join('/');
      return `https://raw.githubusercontent.com/${owner}/${repo}/${nextBranch}/${restPath}`;
    }
    if (
      parsed.hostname === 'github.com' &&
      parts.length >= 4 &&
      (parts[2] === 'tree' || parts[2] === 'blob')
    ) {
      const owner = parts[0];
      const repo = parts[1];
      const kind = parts[2];
      const pathStart = isNamespaced(parts[3]) && parts.length >= 5 ? 5 : 4;
      const restPath = parts.slice(pathStart).join('/');
      return restPath
        ? `https://github.com/${owner}/${repo}/${kind}/${nextBranch}/${restPath}`
        : `https://github.com/${owner}/${repo}/${kind}/${nextBranch}`;
    }
  } catch {
    // ignore
  }
  return rawUrl;
}

export default function SyncPanel({ project = 'horizon', allowUpload = false }) {
  const [config, setConfig] = useState(null);
  const [url, setUrl] = useState('');
  const [repo, setRepo] = useState('');
  const [branch, setBranch] = useState('main');
  const [destPath, setDestPath] = useState('');
  const [availableBranches, setAvailableBranches] = useState(['main']);
  const [newlyDetectedBranches, setNewlyDetectedBranches] = useState([]);
  const [remoteBranchShas, setRemoteBranchShas] = useState({});
  const [loadingBranches, setLoadingBranches] = useState(false);
  const [branchMode, setBranchMode] = useState('create');
  const [newBranchName, setNewBranchName] = useState(() => buildSuggestedBranchName(project));
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState(null);
  const [message, setMessage] = useState(null);
  const [isTokenDialogOpen, setIsTokenDialogOpen] = useState(false);

  async function reloadConfig() {
    try {
      const cfg = await apiFetch(`/github/config?project=${project}`);
      setConfig(cfg);
    } catch {
      // ignore
    }
  }

  useEffect(() => {
    let active = true;
    async function init() {
      try {
        const cfg = await apiFetch(`/github/config?project=${project}`);
        if (active) {
          setConfig(cfg);
          if (cfg.last_source_url) {
            setUrl(cfg.last_source_url);
          }
          if (cfg.repo) setRepo(cfg.repo);
          if (cfg.branch) {
            setBranch(cfg.branch);
            setAvailableBranches((prev) =>
              prev.includes(cfg.branch) ? prev : [cfg.branch, ...prev]
            );
          }
          if (cfg.path) setDestPath(cfg.path);
        }
      } catch (err) {
        if (active) {
          setMessage({
            type: 'error',
            text: err?.message ?? 'Falha ao carregar configurações de sincronização.',
            kind: err?.kind,
          });
        }
      }
    }

    init();

    const unlistenPromise = onSyncProgress((p) => {
      if (p.project === project) {
        setProgress(p);
      }
    });

    return () => {
      active = false;
      if (unlistenPromise && typeof unlistenPromise.then === 'function') {
        unlistenPromise.then((unlisten) => {
          if (typeof unlisten === 'function') unlisten();
        });
      }
    };
  }, [project]);

  function isValidRepo(r) {
    return typeof r === 'string' && /^[^/\s]+\/[^/\s]+$/.test(r.trim());
  }

  function handleSelectBranch(selectedBranch) {
    if (!selectedBranch) return;
    setBranch(selectedBranch);
    if (url && url.trim()) {
      setUrl((prevUrl) => replaceBranchInGitHubUrl(prevUrl, selectedBranch));
    }
  }

  async function handleLoadBranches(options = {}) {
    const isSilent = Boolean(options && options.silent === true);
    const createdBranch =
      options && typeof options.createdBranch === 'string' ? options.createdBranch : null;

    const candidateRepo = isValidRepo(repo)
      ? repo.trim()
      : extractRepoFromGitHubUrl(url);

    if (!isValidRepo(candidateRepo)) {
      if (!isSilent) {
        setMessage({
          type: 'error',
          text: 'O repositório deve estar no formato owner/name para listar as branches.',
        });
      }
      return;
    }

    if (!isValidRepo(repo) && isValidRepo(candidateRepo)) {
      setRepo(candidateRepo);
    }

    setLoadingBranches(true);
    if (!isSilent) {
      setMessage(null);
    }
    try {
      const list = await apiFetch(
        `/github/branches?project=${encodeURIComponent(project)}&repo=${encodeURIComponent(
          candidateRepo
        )}`
      );
      if (!Array.isArray(list)) {
        return;
      }
      const names = list.map((b) => b?.name).filter(Boolean);
      const shaMap = {};
      for (const item of list) {
        if (item?.name && item?.commit_sha) {
          shaMap[item.name] = item.commit_sha;
        }
      }
      setRemoteBranchShas((prev) => ({ ...prev, ...shaMap }));

      const detectedNew = names.filter((n) => !availableBranches.includes(n));
      const allNew = Array.from(
        new Set([...(createdBranch ? [createdBranch] : []), ...detectedNew])
      );
      if (allNew.length > 0) {
        setNewlyDetectedBranches((prev) => Array.from(new Set([...allNew, ...prev])));
      }

      const merged = Array.from(
        new Set([
          branch || 'main',
          'main',
          ...(createdBranch ? [createdBranch] : []),
          ...names,
        ])
      );
      setAvailableBranches(merged);

      if (!isSilent) {
        setMessage({
          type: 'success',
          text:
            detectedNew.length > 0
              ? `Branches sincronizadas com o GitHub (${names.length} encontradas). Novas branches detectadas: ${detectedNew.join(', ')}`
              : `Branches atualizadas do GitHub (${names.length} disponíveis: ${names.join(', ') || 'main'}).`,
        });
      }
    } catch (err) {
      if (!isSilent) {
        setMessage({
          type: 'error',
          text: err?.message ?? 'Falha ao carregar branches do repositório.',
          kind: err?.kind,
        });
      }
    } finally {
      setLoadingBranches(false);
    }
  }

  async function handleDownload(e) {
    if (e) e.preventDefault();
    if (!url.trim()) return;

    setBusy(true);
    setMessage(null);
    setProgress(null);

    try {
      const summary = await apiFetch('/github/download', {
        method: 'POST',
        body: JSON.stringify({ project, url: url.trim() }),
      });

      let summaryText = '';
      if (project === 'horizon') {
        const rows = summary.total_rows ?? 0;
        const tables = summary.tables?.length ?? 0;
        summaryText = `${rows.toLocaleString('pt-BR')} registros em ${tables} tabelas importados com sucesso.`;
      } else if (project === 'egressos') {
        const filesCount = summary.total_files ?? 21;
        const rowsCount = summary.total_rows ?? 0;
        summaryText = `${filesCount} arquivos JSON e ${rowsCount.toLocaleString('pt-BR')} registros importados com sucesso.`;
        window.dispatchEvent(new CustomEvent('egressos:imported', { detail: summary }));
        document.dispatchEvent(new CustomEvent('egressos:imported', { detail: summary }));
      } else {
        const acoes = summary.total_acoes ?? 0;
        const part = summary.total_participacoes ?? 0;
        summaryText = `${acoes.toLocaleString('pt-BR')} ações e ${part.toLocaleString('pt-BR')} participações importadas com sucesso.`;
        window.dispatchEvent(new CustomEvent('src:imported', { detail: summary }));
      }

      setMessage({
        type: 'success',
        text: summaryText,
        snapshot: summary.snapshot,
      });
    } catch (err) {
      setMessage({
        type: 'error',
        text: err?.message ?? 'Falha na sincronização.',
        kind: err?.kind,
      });
    } finally {
      setBusy(false);
      setProgress(null);
    }
  }

  async function handleSaveConfig() {
    if (!isValidRepo(repo)) {
      setMessage({
        type: 'error',
        text: 'O repositório deve estar no formato owner/name.',
      });
      return;
    }

    setBusy(true);
    setMessage(null);

    try {
      const updated = await apiFetch('/github/config', {
        method: 'POST',
        body: JSON.stringify({
          project,
          repo: repo.trim(),
          branch: branch.trim(),
          path: destPath.trim(),
        }),
      });
      setConfig(updated);
      setMessage({
        type: 'success',
        text: 'Destino salvo com sucesso.',
      });
    } catch (err) {
      setMessage({
        type: 'error',
        text: err?.message ?? 'Falha ao salvar destino.',
        kind: err?.kind,
      });
    } finally {
      setBusy(false);
    }
  }

  async function handleUpload() {
    if (!isValidRepo(repo)) {
      setMessage({
        type: 'error',
        text: 'O repositório deve estar no formato owner/name.',
      });
      return;
    }

    setBusy(true);
    setMessage(null);
    setProgress(null);

    try {
      const createNewBranch = branchMode === 'create';
      const baseBranch = branch.trim() || 'main';
      let confirmOverwrite = false;

      if (!createNewBranch) {
        const check = await apiFetch(`/github/check?project=${project}`);
        if (check?.file_exists) {
          const ok = window.confirm('O arquivo já existe no destino. Deseja sobrescrever?');
          if (!ok) {
            setBusy(false);
            return;
          }
          confirmOverwrite = true;
        }
      }

      const res = await apiFetch('/github/upload', {
        method: 'POST',
        body: JSON.stringify({
          project,
          confirmOverwrite,
          createNewBranch,
          baseBranch,
          newBranch: createNewBranch ? newBranchName.trim() || null : null,
          targetBranch: createNewBranch ? null : baseBranch,
        }),
      });

      const publishedBranch =
        res?.branch || (createNewBranch ? newBranchName.trim() : baseBranch);

      if (publishedBranch) {
        setAvailableBranches((prev) =>
          prev.includes(publishedBranch) ? prev : [...prev, publishedBranch]
        );
        if (res?.created_branch) {
          setNewlyDetectedBranches((prev) =>
            prev.includes(publishedBranch) ? prev : [publishedBranch, ...prev]
          );
        }
        if (res?.commit_sha) {
          setRemoteBranchShas((prev) => ({ ...prev, [publishedBranch]: res.commit_sha }));
        }
      }

      if (createNewBranch) {
        setNewBranchName(buildSuggestedBranchName(project));
      }

      setMessage({
        type: 'success',
        text: res?.created_branch
          ? `Enviado com sucesso! Nova branch criada: ${publishedBranch}`
          : `Enviado com sucesso na branch ${publishedBranch}!`,
        canRefreshBranches: true,
        link: {
          url: res.html_url,
          text: res.commit_sha ? res.commit_sha.slice(0, 7) : 'Ver commit',
        },
      });

      await handleLoadBranches({
        silent: true,
        createdBranch: res?.created_branch ? publishedBranch : null,
      });
    } catch (err) {
      setMessage({
        type: 'error',
        text: err?.message ?? 'Falha ao enviar arquivo.',
        kind: err?.kind,
      });
    } finally {
      setBusy(false);
      setProgress(null);
    }
  }

  function formatBytes(bytes) {
    if (!bytes && bytes !== 0) return '';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  return (
    <div className="bg-white border border-slate-200 rounded-xl shadow-sm p-6 max-w-3xl mt-6">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-lg font-semibold text-slate-800">
            Sincronização com GitHub
          </h2>
          <p className="text-sm text-slate-500">
            Baixe e importe dados diretamente de repositórios do GitHub.
          </p>
        </div>
        <div className="flex items-center gap-2">
          {config?.has_token && (
            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-emerald-100 text-emerald-800">
              Token ativo (…{config.token_hint})
            </span>
          )}
          <button
            type="button"
            onClick={() => setIsTokenDialogOpen(true)}
            className="px-2.5 py-1 text-xs font-medium text-slate-700 bg-slate-100 hover:bg-slate-200 border border-slate-300 rounded-lg transition-colors"
          >
            Token de acesso…
          </button>
        </div>
      </div>

      {message && (
        <div
          className={`mb-4 p-4 rounded-lg text-sm ${
            message.type === 'success'
              ? 'bg-emerald-50 text-emerald-800 border border-emerald-200'
              : 'bg-rose-50 text-rose-800 border border-rose-200'
          }`}
        >
          <div className="flex items-start justify-between">
            <div>
              <p className="font-medium">{message.text}</p>
              {message.snapshot && (
                <p className="mt-1 text-xs opacity-80">
                  Snapshot de segurança salvo em: {message.snapshot}
                </p>
              )}
              {message.link && (
                <a
                  href={message.link.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="mt-1 inline-block text-xs font-mono underline hover:opacity-80"
                >
                  {message.link.text}
                </a>
              )}
              {allowUpload && message.type === 'success' && message.canRefreshBranches && (
                <button
                  type="button"
                  onClick={() => handleLoadBranches({ silent: false })}
                  disabled={busy || loadingBranches}
                  className="mt-2 block text-xs font-semibold text-emerald-900 underline hover:text-emerald-950"
                >
                  ↻ Atualizar lista de branches do GitHub
                </button>
              )}
              {message.kind === 'github_api' &&
                (message.text.includes('autenticado') || message.text.includes('token')) && (
                  <button
                    type="button"
                    onClick={() => setIsTokenDialogOpen(true)}
                    className="mt-2 text-xs font-semibold text-rose-900 underline hover:text-rose-950 block text-left"
                  >
                    Configurar token de acesso…
                  </button>
                )}
            </div>
            {message.kind && (
              <span className="ml-2 px-2 py-0.5 bg-rose-200 text-rose-900 rounded text-xs font-mono">
                {message.kind}
              </span>
            )}
          </div>
        </div>
      )}

      {progress && (
        <div className="mb-4 p-3 bg-slate-50 border border-slate-200 rounded-lg text-xs text-slate-600">
          <div className="flex justify-between font-medium text-slate-700 mb-1">
            <span>
              {progress.phase === 'started' && 'Iniciando download…'}
              {progress.phase === 'transferring' && 'Transferindo dados…'}
              {progress.phase === 'importing' && 'Importando para a base…'}
              {progress.phase === 'committing' && 'Gravando commit…'}
              {progress.phase === 'done' && 'Concluído'}
              {progress.phase === 'failed' && 'Falhou'}
            </span>
            {progress.bytes_done > 0 && (
              <span>
                {formatBytes(progress.bytes_done)}
                {progress.bytes_total ? ` / ${formatBytes(progress.bytes_total)}` : ''}
              </span>
            )}
          </div>
          {progress.bytes_total && (
            <div className="w-full bg-slate-200 rounded-full h-1.5 overflow-hidden">
              <div
                className="bg-blue-600 h-1.5 rounded-full transition-all duration-200"
                style={{
                  width: `${Math.min(100, Math.round((progress.bytes_done / progress.bytes_total) * 100))}%`,
                }}
              />
            </div>
          )}
        </div>
      )}

      <form onSubmit={handleDownload} className="space-y-4">
        <div>
          <label htmlFor="sync-url" className="block text-xs font-medium text-slate-700 mb-1">
            {project === 'egressos'
              ? 'URL da pasta dados/ no GitHub'
              : 'URL do arquivo no GitHub'}
          </label>
          <input
            id="sync-url"
            name="sync-url"
            type="url"
            aria-label="URL do arquivo"
            placeholder={
              project === 'horizon'
                ? 'https://raw.githubusercontent.com/owner/repo/main/exports/exports_canonical.zip'
                : project === 'egressos'
                ? 'https://github.com/owner/repo/tree/main/dados'
                : 'https://raw.githubusercontent.com/owner/repo/main/dados/src_consolidado.json'
            }
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            disabled={busy}
            required
            className="w-full px-3 py-2 border border-slate-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-slate-50 disabled:text-slate-400"
          />
        </div>

        <div className="flex items-center gap-3">
          <button
            type="submit"
            disabled={busy || !url.trim()}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-300 text-white font-medium text-sm rounded-lg transition-colors shadow-sm"
          >
            {busy ? 'Processando…' : 'Baixar e importar'}
          </button>
        </div>
      </form>

      {allowUpload && (
        <div className="mt-8 pt-6 border-t border-slate-200">
          <h3 className="text-base font-semibold text-slate-800 mb-1">
            Envio para o GitHub
          </h3>
          <p className="text-xs text-slate-500 mb-4">
            {project === 'egressos'
              ? 'Configure o repositório e a pasta de destino para publicar os 21 arquivos JSON do Egressos em um único commit.'
              : project === 'src'
              ? 'Configure o repositório e o caminho do arquivo .json para publicar o JSON consolidado do SRC.'
              : 'Configure o repositório de destino para exportar a base do Horizon.'}
          </p>

          <p className="mb-4 p-3 bg-amber-50 border border-amber-200 rounded-lg text-xs text-amber-800">
            Aviso de privacidade: verifique a visibilidade do repositório antes do envio. Não envie dados pessoais ou confidenciais sem a devida autorização.
          </p>

          <form onSubmit={(e) => { e.preventDefault(); handleSaveConfig(); }} className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
              <div>
                <label htmlFor="dest-repo" className="block text-xs font-medium text-slate-700 mb-1">
                  Repositório
                </label>
                <input
                  id="dest-repo"
                  name="dest-repo"
                  type="text"
                  aria-label="Repositório"
                  placeholder="owner/repo"
                  value={repo}
                  onChange={(e) => setRepo(e.target.value)}
                  disabled={busy}
                  className="w-full px-3 py-2 border border-slate-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-slate-50 disabled:text-slate-400"
                />
              </div>
              <div>
                <div className="flex items-center justify-between mb-1">
                  <label htmlFor="dest-branch" className="block text-xs font-medium text-slate-700">
                    Branch
                  </label>
                  <button
                    type="button"
                    aria-label="Carregar branches / Atualizar branches"
                    onClick={() => handleLoadBranches({ silent: false })}
                    disabled={busy || loadingBranches}
                    className="inline-flex items-center gap-1 text-[11px] font-medium text-blue-600 hover:text-blue-800 underline disabled:text-slate-400"
                  >
                    {loadingBranches ? 'Atualizando…' : '↻ Atualizar branches'}
                  </button>
                </div>
                <input
                  id="dest-branch"
                  name="dest-branch"
                  type="text"
                  aria-label="Branch"
                  placeholder="main"
                  value={branch}
                  onChange={(e) => setBranch(e.target.value)}
                  disabled={busy}
                  className="w-full px-3 py-2 border border-slate-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-slate-50 disabled:text-slate-400"
                />
                {availableBranches.length > 0 && (
                  <select
                    aria-label="Selecionar branch existente"
                    value={availableBranches.includes(branch) ? branch : ''}
                    onChange={(e) => {
                      if (e.target.value) handleSelectBranch(e.target.value);
                    }}
                    disabled={busy}
                    className="mt-1.5 w-full px-2.5 py-1.5 border border-slate-200 rounded-lg text-xs text-slate-700 bg-slate-50"
                  >
                    {availableBranches.map((b) => (
                      <option key={b} value={b}>
                        {b}
                        {newlyDetectedBranches.includes(b) ? ' (nova)' : ''}
                      </option>
                    ))}
                  </select>
                )}
                {newlyDetectedBranches.length > 0 && (
                  <div
                    data-testid="newly-detected-branches"
                    className="mt-1.5 p-2 rounded-lg bg-emerald-50 border border-emerald-200 text-[11px] text-emerald-900"
                  >
                    <span className="font-semibold block mb-1">
                      Novas branches detectadas no GitHub:
                    </span>
                    <div className="flex flex-wrap gap-1">
                      {newlyDetectedBranches.map((nb) => (
                        <button
                          key={nb}
                          type="button"
                          onClick={() => handleSelectBranch(nb)}
                          className="px-2 py-0.5 rounded-full bg-emerald-100 hover:bg-emerald-200 text-emerald-900 font-mono border border-emerald-300 transition-colors"
                          title="Clique para selecionar esta branch"
                        >
                          {nb}
                          {remoteBranchShas[nb] ? ` (${remoteBranchShas[nb].slice(0, 7)})` : ''}
                        </button>
                      ))}
                    </div>
                  </div>
                )}
              </div>
              <div>
                <label htmlFor="dest-path" className="block text-xs font-medium text-slate-700 mb-1">
                  {project === 'egressos' ? 'Caminho da pasta' : 'Caminho do arquivo'}
                </label>
                <input
                  id="dest-path"
                  name="dest-path"
                  type="text"
                  aria-label="Caminho do arquivo"
                  placeholder={
                    project === 'egressos'
                      ? 'dados'
                      : project === 'src'
                      ? 'dados/src_consolidado.json'
                      : 'exports/exports_canonical.zip'
                  }
                  value={destPath}
                  onChange={(e) => setDestPath(e.target.value)}
                  disabled={busy}
                  className="w-full px-3 py-2 border border-slate-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-slate-50 disabled:text-slate-400"
                />
              </div>
            </div>

            <div className="p-3 bg-slate-50 border border-slate-200 rounded-lg space-y-3">
              <span className="block text-xs font-semibold text-slate-700">
                Modo de branch no envio
              </span>
              <div className="flex flex-wrap gap-4 text-xs text-slate-700">
                <label className="inline-flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name={`branch-mode-${project}`}
                    value="create"
                    checked={branchMode === 'create'}
                    onChange={() => setBranchMode('create')}
                    disabled={busy}
                    aria-label="Criar nova branch com os novos dados"
                  />
                  <span>
                    <strong>Criar nova branch com os novos dados</strong> (ramifica de{' '}
                    <code>{branch || 'main'}</code>)
                  </span>
                </label>
                <label className="inline-flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name={`branch-mode-${project}`}
                    value="existing"
                    checked={branchMode === 'existing'}
                    onChange={() => setBranchMode('existing')}
                    disabled={busy}
                    aria-label="Enviar na branch selecionada"
                  />
                  <span>
                    <strong>Enviar na branch selecionada</strong> (<code>{branch || 'main'}</code>)
                  </span>
                </label>
              </div>

              {branchMode === 'create' && (
                <div>
                  <label
                    htmlFor={`new-branch-${project}`}
                    className="block text-xs font-medium text-slate-700 mb-1"
                  >
                    Nome da nova branch
                  </label>
                  <input
                    id={`new-branch-${project}`}
                    type="text"
                    aria-label="Nome da nova branch"
                    placeholder={`update/${project}-YYYYMMDD-HHMMSS`}
                    value={newBranchName}
                    onChange={(e) => setNewBranchName(e.target.value)}
                    disabled={busy}
                    className="w-full px-3 py-1.5 border border-slate-300 rounded-lg text-xs font-mono bg-white"
                  />
                </div>
              )}
            </div>

            <div className="flex items-center gap-3">
              <button
                type="button"
                onClick={handleSaveConfig}
                disabled={busy}
                className="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 font-medium text-sm rounded-lg transition-colors border border-slate-300"
              >
                Salvar destino
              </button>
              <button
                type="button"
                onClick={handleUpload}
                disabled={busy}
                className="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:bg-slate-300 text-white font-medium text-sm rounded-lg transition-colors shadow-sm"
              >
                {busy ? 'Enviando…' : 'Enviar para o GitHub'}
              </button>
            </div>
          </form>
        </div>
      )}

      <TokenDialog
        isOpen={isTokenDialogOpen}
        onClose={() => setIsTokenDialogOpen(false)}
        hasToken={Boolean(config?.has_token)}
        tokenHint={config?.token_hint}
        onTokenChanged={reloadConfig}
      />
    </div>
  );
}
