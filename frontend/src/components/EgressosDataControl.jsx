import React, { useState, useEffect, useCallback } from 'react';
import { apiFetch, onEgressosImportProgress } from '../services/api.js';

// SEP-035 / US2 (FR-004..FR-006, FR-010): importação e exportação direta da
// pasta local de 21 arquivos JSON do projeto Egressos (`dados/`), preservando
// 100% da estrutura.
export default function EgressosDataControl() {
  const [catalog, setCatalog] = useState(null);
  const [busy, setBusy] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [message, setMessage] = useState(null);
  const [progress, setProgress] = useState(null);

  const loadCatalog = useCallback(async () => {
    try {
      const data = await apiFetch('/egressos/catalog');
      if (data) {
        setCatalog(data);
      }
    } catch {
      // Se ainda não autenticado ou inicializando, ignora silenciosamente.
    }
  }, []);

  useEffect(() => {
    loadCatalog();
    const onImported = () => loadCatalog();
    window.addEventListener('egressos:imported', onImported);
    document.addEventListener('egressos:imported', onImported);
    return () => {
      window.removeEventListener('egressos:imported', onImported);
      document.removeEventListener('egressos:imported', onImported);
    };
  }, [loadCatalog]);

  useEffect(() => {
    let cleanup = null;
    const maybePromise = onEgressosImportProgress((payload) => {
      setProgress(payload);
    });
    if (maybePromise && typeof maybePromise.then === 'function') {
      maybePromise.then((off) => {
        cleanup = off;
      });
    }
    return () => {
      if (typeof cleanup === 'function') cleanup();
    };
  }, []);

  async function runImport(path) {
    setBusy(true);
    setMessage(null);
    setProgress(null);
    try {
      const summary = await apiFetch('/egressos/import', {
        method: 'POST',
        body: JSON.stringify({ path: path ?? null }),
      });
      if (summary === null) {
        setBusy(false);
        return; // diálogo cancelado pelo curador
      }
      setMessage({
        type: 'success',
        text: `Importação concluída: ${summary.total_files} arquivos JSON importados (${summary.total_rows.toLocaleString('pt-BR')} registros no total).`,
        path: summary.path,
        snapshot: summary.snapshot,
      });
      window.dispatchEvent(new CustomEvent('egressos:imported', { detail: summary }));
      document.dispatchEvent(new CustomEvent('egressos:imported', { detail: summary }));
      window.dispatchEvent(new CustomEvent('egressos:dirty', { detail: false }));
      document.dispatchEvent(new CustomEvent('egressos:dirty', { detail: false }));
      await loadCatalog();
    } catch (err) {
      setMessage({ type: 'error', text: err?.message ?? String(err) });
    } finally {
      setBusy(false);
    }
  }

  async function runExport(path) {
    setExporting(true);
    setMessage(null);
    try {
      const summary = await apiFetch('/egressos/export', {
        method: 'POST',
        body: JSON.stringify({ path: path ?? null }),
      });
      if (summary === null) {
        setExporting(false);
        return; // diálogo cancelado
      }
      setMessage({
        type: 'success',
        text: `Exportação concluída: ${summary.total_files} arquivos JSON gravados (${summary.total_rows.toLocaleString('pt-BR')} registros).`,
        path: summary.path,
      });
      window.dispatchEvent(new CustomEvent('egressos:dirty', { detail: false }));
      document.dispatchEvent(new CustomEvent('egressos:dirty', { detail: false }));
    } catch (err) {
      setMessage({ type: 'error', text: err?.message ?? String(err) });
    } finally {
      setExporting(false);
    }
  }

  return (
    <div className="space-y-6 mt-6">
      {/* Resumo dinâmico da base carregada + atalhos para as 21 tabelas */}
      {catalog && catalog.loaded && (
        <div className="bg-white p-6 rounded-xl shadow-sm border border-slate-200">
          <div className="flex flex-wrap items-center justify-between gap-4 mb-4">
            <div>
              <h2 className="text-lg font-semibold text-slate-800">
                Catálogo carregado ({catalog.total_files} arquivos JSON •{' '}
                {catalog.total_rows.toLocaleString('pt-BR')} registros)
              </h2>
              <p className="text-xs text-slate-500">
                Clique em qualquer tabela abaixo para abrir sua aba diretamente no editor CRUD.
              </p>
            </div>
            <a
              href="/egressos/tabelas"
              className="text-sm font-medium text-emerald-700 hover:text-emerald-800"
            >
              Abrir editor das 21 tabelas →
            </a>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2.5 max-h-64 overflow-y-auto pr-1">
            {(catalog.files || []).map((f) => (
              <a
                key={f.file_key}
                href={`/egressos/tabelas?tab=${encodeURIComponent(f.file_key)}`}
                className="flex items-center justify-between px-3 py-2 rounded-lg border border-slate-200 hover:border-emerald-500 hover:bg-emerald-50/40 transition-colors text-xs"
              >
                <span className="font-medium text-slate-800 truncate" title={f.title}>
                  {f.title}
                </span>
                <span className="ml-2 shrink-0 px-2 py-0.5 rounded-full bg-slate-100 text-slate-600 font-mono">
                  {f.total_rows}
                </span>
              </a>
            ))}
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-white p-6 rounded-lg shadow-sm border border-slate-200">
          <h2 className="text-xl font-semibold text-slate-800 mb-3">
            Importar pasta local (21 arquivos JSON)
          </h2>
          <p className="text-slate-600 mb-4 text-sm">
            Carrega diretamente os 21 arquivos <code>.json</code> da pasta{' '}
            <code>dados/</code> do projeto Egressos com validação atômica. Se já houver
            dados carregados, um snapshot <code>egressos-pre-import-*.db</code> é salvo
            automaticamente antes da substituição.
          </p>

          <div className="flex flex-wrap gap-3">
            <button
              type="button"
              onClick={() => runImport(null)}
              disabled={busy || exporting}
              className={`bg-emerald-600 text-white px-4 py-2 rounded font-medium hover:bg-emerald-700 transition-colors text-sm ${
                busy || exporting ? 'opacity-50 cursor-not-allowed' : ''
              }`}
            >
              {busy ? 'Importando...' : 'Escolher pasta local...'}
            </button>
          </div>

          {progress && (
            <div className="mt-4 text-xs text-slate-600 bg-slate-50 p-2.5 rounded border border-slate-200">
              Fase: <strong>{progress.phase}</strong> — {progress.files_done ?? 0}/
              {progress.files_total ?? 21} arquivos
              {progress.file_key ? ` (${progress.file_key}.json)` : ''}
            </div>
          )}

          {message && (
            <div
              className={`mt-4 p-3 rounded text-sm ${
                message.type === 'error'
                  ? 'bg-red-50 text-red-700 border border-red-200'
                  : 'bg-emerald-50 text-emerald-800 border border-emerald-200'
              }`}
            >
              <div>{message.text}</div>
              {message.path && (
                <div className="mt-1 text-xs text-slate-600 break-all">
                  Pasta: <code>{message.path}</code>
                </div>
              )}
              {message.snapshot && (
                <div className="mt-1 text-xs text-slate-600 break-all">
                  Snapshot de segurança: <code>{message.snapshot}</code>
                </div>
              )}
            </div>
          )}
        </div>

        <div className="bg-white p-6 rounded-lg shadow-sm border border-slate-200">
          <h2 className="text-xl font-semibold text-slate-800 mb-3">
            Exportar base Egressos (pasta dados/)
          </h2>
          <p className="text-slate-600 mb-4 text-sm">
            Reconstrói os 21 arquivos <code>.json</code> diretamente na pasta escolhida,
            preservando todas as chaves de metadados, seções secundárias e formatação
            original do repositório Egressos.
          </p>

          <div className="flex flex-wrap gap-3">
            <button
              type="button"
              onClick={() => runExport(null)}
              disabled={exporting || busy}
              className={`bg-blue-600 text-white px-4 py-2 rounded font-medium hover:bg-blue-700 transition-colors text-sm ${
                exporting || busy ? 'opacity-50 cursor-not-allowed' : ''
              }`}
            >
              {exporting ? 'Exportando...' : 'Exportar pasta dados/...'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
