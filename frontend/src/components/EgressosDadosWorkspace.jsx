import React, { useState, useEffect } from 'react';
import { apiFetch } from '../services/api.js';
import EgressosDadosTablesPage from './EgressosDadosTablesPage.jsx';
import EgressosDadosDataControl from './EgressosDadosDataControl.jsx';

export default function EgressosDadosWorkspace({ mode = '', children = null }) {
  const [loading, setLoading] = useState(true);
  const [authStatus, setAuthStatus] = useState({
    authorized: false,
    login: null,
    organization: 'ifesserra-lab',
    token_hint: null,
  });
  const [workspaceInfo, setWorkspaceInfo] = useState(null);
  const [tokenInput, setTokenInput] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState(null);

  async function loadWorkspaceDetails() {
    try {
      const ws = await apiFetch('/egressos-dados/workspace');
      setWorkspaceInfo(ws);
    } catch {
      setWorkspaceInfo(null);
    }
  }

  useEffect(() => {
    let mounted = true;
    (async () => {
      try {
        const status = await apiFetch('/egressos-dados/auth/status');
        if (!mounted) return;
        setAuthStatus(status);
        if (status?.authorized) {
          await loadWorkspaceDetails();
        }
      } catch (err) {
        if (!mounted) return;
        setError({
          message:
            err?.message || 'Não foi possível verificar o status da aba egressos-dados.',
          kind: err?.kind || null,
        });
      } finally {
        if (mounted) setLoading(false);
      }
    })();
    return () => {
      mounted = false;
    };
  }, []);

  async function handleLogin(e) {
    e.preventDefault();
    const trimmed = tokenInput.trim();
    if (!trimmed) {
      setError({
        message: 'Informe o Token de Acesso Pessoal do GitHub.',
        kind: 'validation',
      });
      return;
    }

    setSubmitting(true);
    setError(null);

    try {
      const status = await apiFetch('/egressos-dados/auth/login', {
        method: 'POST',
        body: JSON.stringify({ token: trimmed }),
      });
      setTokenInput('');
      setAuthStatus(status);
      if (status?.authorized) {
        await loadWorkspaceDetails();
      }
    } catch (err) {
      setAuthStatus({
        authorized: false,
        login: null,
        organization: 'ifesserra-lab',
        token_hint: null,
      });
      setWorkspaceInfo(null);
      setError({
        message:
          err?.message ||
          'Falha ao validar participação na organização ifesserra-lab.',
        kind: err?.kind || null,
      });
    } finally {
      setSubmitting(false);
    }
  }

  async function handleLogout() {
    setSubmitting(true);
    setError(null);
    try {
      const status = await apiFetch('/egressos-dados/auth/logout', {
        method: 'DELETE',
      });
      setAuthStatus(
        status || {
          authorized: false,
          login: null,
          organization: 'ifesserra-lab',
          token_hint: null,
        }
      );
      setWorkspaceInfo(null);
      setTokenInput('');
    } catch (err) {
      setError({
        message: err?.message || 'Falha ao encerrar a sessão especial.',
        kind: err?.kind || null,
      });
    } finally {
      setSubmitting(false);
    }
  }

  if (loading) {
    return (
      <div className="bg-white border border-slate-200 rounded-xl shadow-sm p-6 max-w-2xl">
        <p className="text-sm text-slate-500">
          Verificando autorização da organização <code>ifesserra-lab</code>…
        </p>
      </div>
    );
  }

  if (!authStatus?.authorized) {
    return (
      <div
        data-testid="egressos-dados-locked-gate"
        className="bg-white border border-slate-200 rounded-xl shadow-sm p-8 max-w-xl"
      >
        <div className="flex items-center gap-2 mb-3">
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-semibold bg-amber-100 text-amber-800 border border-amber-200">
            Login Especial Obrigatório
          </span>
          <span className="text-xs font-mono text-slate-500">
            github.com/orgs/ifesserra-lab
          </span>
        </div>

        <h2 className="text-xl font-bold text-slate-900 mb-2">
          Acesso Restrito — Organização ifesserra-lab
        </h2>
        <p className="text-sm text-slate-600 mb-4">
          Somente membros ativos da organização{' '}
          <strong className="font-semibold text-slate-800">ifesserra-lab</strong> no
          GitHub conseguem desbloquear a aba <code>egressos-dados</code> e suas tabelas.
        </p>

        <div className="p-3 mb-5 bg-slate-50 border border-slate-200 rounded-lg text-xs text-slate-600 space-y-1">
          <p className="font-semibold text-slate-700">Requisitos do Token do GitHub:</p>
          <ul className="list-disc list-inside space-y-0.5">
            <li>
              Pertencer a uma conta membro ativa de{' '}
              <code>https://github.com/orgs/ifesserra-lab/dashboard</code>.
            </li>
            <li>
              Possuir permissão <code>read:org</code> (caso sua filiação na organização seja
              privada).
            </li>
            <li>
              O token fica apenas em memória durante a sessão atual e nunca é gravado em disco.
            </li>
          </ul>
        </div>

        {error && (
          <div
            role="alert"
            className="p-3 mb-4 rounded-lg bg-rose-50 border border-rose-200 text-xs text-rose-800 flex items-start justify-between gap-2"
          >
            <span>{error.message}</span>
            {error.kind && (
              <span className="px-1.5 py-0.5 bg-rose-200 text-rose-900 rounded font-mono text-[10px] shrink-0">
                {error.kind}
              </span>
            )}
          </div>
        )}

        <form onSubmit={handleLogin} className="space-y-4">
          <div>
            <label
              htmlFor="egressos-dados-github-token"
              className="block text-sm font-medium text-slate-700 mb-1"
            >
              Token de Acesso Pessoal do GitHub
            </label>
            <input
              id="egressos-dados-github-token"
              name="token"
              type="password"
              autoComplete="off"
              placeholder="ghp_... ou github_pat_..."
              value={tokenInput}
              onChange={(e) => setTokenInput(e.target.value)}
              disabled={submitting}
              className="w-full px-3.5 py-2 border border-slate-300 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-sky-500 disabled:bg-slate-50"
            />
          </div>

          <button
            type="submit"
            disabled={submitting}
            className="w-full px-4 py-2.5 bg-slate-900 hover:bg-slate-800 disabled:bg-slate-400 text-white text-sm font-medium rounded-lg transition-colors shadow-sm"
          >
            {submitting
              ? 'Verificando participação em ifesserra-lab…'
              : 'Validar membro ifesserra-lab e entrar'}
          </button>
        </form>
      </div>
    );
  }

  return (
    <div data-testid="egressos-dados-unlocked-panel" className="space-y-6">
      <div className="bg-white border border-emerald-200 rounded-xl shadow-sm p-4 flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div className="space-y-0.5">
          <div className="flex flex-wrap items-center gap-2">
            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-semibold bg-emerald-100 text-emerald-800 border border-emerald-300">
              Membro verificado: {authStatus.organization || 'ifesserra-lab'}
            </span>
            <span className="text-xs font-mono text-slate-500">
              Token em sessão: …{authStatus.token_hint || '****'}
            </span>
          </div>
          <h2 className="text-base font-bold text-slate-900">
            Autenticado como{' '}
            <span className="font-mono text-emerald-700">@{authStatus.login}</span>
          </h2>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <a
            href="/egressos-dados"
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold border transition-colors ${
              mode === 'tables'
                ? 'bg-slate-900 text-white border-slate-900'
                : 'bg-slate-50 text-slate-700 border-slate-200 hover:bg-slate-100'
            }`}
          >
            Tabelas (39 JSONs)
          </a>
          <a
            href="/egressos-dados/controle"
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold border transition-colors ${
              mode === 'controle'
                ? 'bg-slate-900 text-white border-slate-900'
                : 'bg-slate-50 text-slate-700 border-slate-200 hover:bg-slate-100'
            }`}
          >
            Controle de Dados & Sync
          </a>
          <button
            type="button"
            onClick={handleLogout}
            disabled={submitting}
            className="px-3.5 py-1.5 bg-rose-50 hover:bg-rose-100 text-rose-700 border border-rose-200 text-xs font-semibold rounded-lg transition-colors disabled:opacity-50"
          >
            Bloquear aba / Trocar token
          </button>
        </div>
      </div>

      {children ? (
        children
      ) : mode === 'tables' ? (
        <EgressosDadosTablesPage />
      ) : mode === 'controle' ? (
        <EgressosDadosDataControl />
      ) : (
        <div className="bg-white border border-slate-200 rounded-xl shadow-sm p-6">
          <h3 className="text-base font-semibold text-slate-800 mb-2">
            Tabelas de Dados — Egressos (ifesserra-lab)
          </h3>
          <p className="text-sm text-slate-600 mb-4">
            O controle de acesso exclusivo para membros da organização{' '}
            <code>ifesserra-lab</code> está ativo.
          </p>
          <dl className="grid grid-cols-1 md:grid-cols-3 gap-4 text-center">
            <div className="bg-slate-50 rounded-lg p-3 border border-slate-100">
              <dt className="text-xs text-slate-500">Organização Autorizada</dt>
              <dd className="text-sm font-bold text-slate-800 font-mono">
                {workspaceInfo?.organization || authStatus.organization}
              </dd>
            </div>
            <div className="bg-slate-50 rounded-lg p-3 border border-slate-100">
              <dt className="text-xs text-slate-500">Membro Autenticado</dt>
              <dd className="text-sm font-bold text-emerald-700 font-mono">
                @{workspaceInfo?.login || authStatus.login}
              </dd>
            </div>
            <div className="bg-slate-50 rounded-lg p-3 border border-slate-100">
              <dt className="text-xs text-slate-500">Status das Tabelas JSON</dt>
              <dd className="text-sm font-semibold text-emerald-700">
                {workspaceInfo?.tables_ready
                  ? '39 tabelas prontas'
                  : 'Aguardando configuração do repositório'}
              </dd>
            </div>
          </dl>
        </div>
      )}
    </div>
  );
}
