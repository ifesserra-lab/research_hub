import React, { useState, useEffect, useCallback } from 'react';
import { apiFetch } from '../services/api.js';

function formatCellValue(val) {
  if (val === null || val === undefined) return '—';
  if (typeof val === 'boolean') return val ? 'Sim' : 'Não';
  if (typeof val === 'number') return String(val);
  if (typeof val === 'string') return val || '—';
  if (Array.isArray(val)) {
    if (val.length === 0) return '[]';
    if (val.every((v) => typeof v === 'string' || typeof v === 'number')) {
      return val.slice(0, 4).join(', ') + (val.length > 4 ? ` (+${val.length - 4})` : '');
    }
    return `[${val.length} itens]`;
  }
  if (typeof val === 'object') {
    return JSON.stringify(val);
  }
  return String(val);
}

function setDirty(flag) {
  window.dispatchEvent(new CustomEvent('egressos:dirty', { detail: Boolean(flag) }));
  document.dispatchEvent(new CustomEvent('egressos:dirty', { detail: Boolean(flag) }));
}

export default function EgressosTablesPage() {
  const [catalog, setCatalog] = useState(null);
  const [activeFileKey, setActiveFileKey] = useState('egressos_perfil');
  const [activeSection, setActiveSection] = useState('');
  const [page, setPage] = useState({
    items: [],
    total: 0,
    limit: 50,
    offset: 0,
    columns: [],
    entity_group: 'none',
    is_map_section: false,
  });
  const [searchInput, setSearchInput] = useState('');
  const [appliedSearch, setAppliedSearch] = useState('');
  const [sortCol, setSortCol] = useState('');
  const [sortOrder, setSortOrder] = useState('asc');
  const [offset, setOffset] = useState(0);
  const [loading, setLoading] = useState(false);
  const [feedback, setFeedback] = useState(null);
  const [propagateDelete, setPropagateDelete] = useState(false);

  // Modal de criação/edição de registro
  const [modalOpen, setModalOpen] = useState(false);
  const [editingRowId, setEditingRowId] = useState(null);
  const [formValues, setFormValues] = useState({});
  const [rawJsonDraft, setRawJsonDraft] = useState('');
  const [useRawJsonMode, setUseRawJsonMode] = useState(false);
  const [propagateEdit, setPropagateEdit] = useState(false);
  const [modalError, setModalError] = useState(null);

  // Modal de metadados de nível de arquivo (envelope FR-009)
  const [envelopeOpen, setEnvelopeOpen] = useState(false);
  const [envelopeText, setEnvelopeText] = useState('{}');
  const [envelopeError, setEnvelopeError] = useState(null);

  const loadCatalog = useCallback(async () => {
    try {
      const data = await apiFetch('/egressos/catalog');
      if (data && Array.isArray(data.files)) {
        setCatalog(data);
        const params = new URLSearchParams(window.location.search || '');
        const tabParam = params.get('tab');
        const found = data.files.find((f) => f.file_key === tabParam);
        const initialFile = found || data.files[0];
        if (initialFile) {
          setActiveFileKey(initialFile.file_key);
          setActiveSection((prev) => prev || initialFile.primary_section);
        }
      }
    } catch (err) {
      setFeedback({ type: 'error', text: err?.message ?? String(err) });
    }
  }, []);

  useEffect(() => {
    loadCatalog();
  }, [loadCatalog]);

  const currentFileDef =
    catalog?.files?.find((f) => f.file_key === activeFileKey) || null;

  const effectiveSection =
    activeSection || currentFileDef?.primary_section || '';

  const loadRows = useCallback(async () => {
    if (!activeFileKey || !effectiveSection) return;
    setLoading(true);
    try {
      const qs = new URLSearchParams();
      qs.set('section', effectiveSection);
      qs.set('limit', '50');
      qs.set('offset', String(offset));
      if (appliedSearch) qs.set('search', appliedSearch);
      if (sortCol) {
        qs.set('sort', sortCol);
        qs.set('order', sortOrder);
      }
      const res = await apiFetch(`/egressos/files/${activeFileKey}/rows?${qs.toString()}`);
      if (res) {
        setPage({
          items: res.items || [],
          total: res.total ?? 0,
          limit: res.limit ?? 50,
          offset: res.offset ?? 0,
          columns: res.columns || currentFileDef?.columns || [],
          entity_group: res.entity_group || currentFileDef?.entity_group || 'none',
          is_map_section: Boolean(res.is_map_section),
        });
      }
    } catch (err) {
      setFeedback({ type: 'error', text: err?.message ?? String(err) });
    } finally {
      setLoading(false);
    }
  }, [activeFileKey, effectiveSection, offset, appliedSearch, sortCol, sortOrder, currentFileDef]);

  useEffect(() => {
    if (catalog && activeFileKey && effectiveSection) {
      loadRows();
    }
  }, [catalog, activeFileKey, effectiveSection, loadRows]);

  function handleSelectFileTab(fileDef) {
    setActiveFileKey(fileDef.file_key);
    setActiveSection(fileDef.primary_section);
    setOffset(0);
    setSearchInput('');
    setAppliedSearch('');
    setSortCol('');
    setFeedback(null);
  }

  function handleSelectSection(secKey) {
    setActiveSection(secKey);
    setOffset(0);
    setSearchInput('');
    setAppliedSearch('');
    setSortCol('');
    setFeedback(null);
  }

  function handleSearchSubmit(e) {
    e.preventDefault();
    setOffset(0);
    setAppliedSearch(searchInput.trim());
  }

  function handleSort(col) {
    if (sortCol === col) {
      setSortOrder((prev) => (prev === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortCol(col);
      setSortOrder('asc');
    }
    setOffset(0);
  }

  function openCreateModal() {
    const cols = page.columns?.length ? page.columns : currentFileDef?.columns || ['value'];
    const initial = {};
    for (const c of cols) {
      initial[c] = '';
    }
    setEditingRowId(null);
    setFormValues(initial);
    setRawJsonDraft(JSON.stringify(initial, null, 2));
    setUseRawJsonMode(false);
    setPropagateEdit(false);
    setModalError(null);
    setModalOpen(true);
    setDirty(true);
  }

  async function openEditModal(row) {
    setModalError(null);
    try {
      const full = await apiFetch(`/egressos/rows/${row.id}`);
      const clean = { ...(full || row) };
      delete clean.id;
      delete clean._record_key;
      setEditingRowId(row.id);
      setFormValues(clean);
      setRawJsonDraft(JSON.stringify(clean, null, 2));
      setUseRawJsonMode(false);
      setPropagateEdit(false);
      setModalOpen(true);
      setDirty(true);
    } catch (err) {
      setFeedback({ type: 'error', text: err?.message ?? String(err) });
    }
  }

  function closeRecordModal() {
    setModalOpen(false);
    setEditingRowId(null);
    setModalError(null);
    setDirty(false);
  }

  function handleFieldChange(field, rawValue) {
    setFormValues((prev) => {
      const currentVal = prev[field];
      let parsed = rawValue;
      if (typeof currentVal === 'number') {
        const num = Number(rawValue);
        parsed = Number.isNaN(num) ? rawValue : num;
      } else if (typeof currentVal === 'boolean') {
        parsed = rawValue === 'true' || rawValue === true;
      }
      const next = { ...prev, [field]: parsed };
      setRawJsonDraft(JSON.stringify(next, null, 2));
      return next;
    });
  }

  async function handleSaveRecord(e) {
    e.preventDefault();
    setModalError(null);
    let payloadToSave = { ...formValues };
    if (useRawJsonMode) {
      try {
        payloadToSave = JSON.parse(rawJsonDraft);
      } catch {
        setModalError('JSON inválido no modo avançado.');
        return;
      }
    }

    try {
      if (editingRowId === null) {
        await apiFetch(`/egressos/files/${activeFileKey}/rows`, {
          method: 'POST',
          body: JSON.stringify({
            section_key: effectiveSection,
            payload: payloadToSave,
          }),
        });
        setFeedback({ type: 'success', text: 'Novo registro criado com sucesso.' });
      } else {
        const res = await apiFetch(`/egressos/rows/${editingRowId}`, {
          method: 'PUT',
          body: JSON.stringify({
            payload: payloadToSave,
            propagate: propagateEdit,
          }),
        });
        const propMsg =
          res?.propagated_count > 0
            ? ` (${res.propagated_count} registros sincronizados nas tabelas irmãs)`
            : '';
        setFeedback({
          type: 'success',
          text: `Registro #${editingRowId} atualizado com sucesso${propMsg}.`,
        });
      }
      closeRecordModal();
      await loadRows();
      await loadCatalog();
    } catch (err) {
      setModalError(err?.message ?? String(err));
    }
  }

  async function handleDeleteRow(row) {
    const canPropagate =
      page.entity_group === 'alumni' || page.entity_group === 'company';
    const useProp = canPropagate && propagateDelete;
    const confirmMsg = useProp
      ? `Excluir o registro #${row.id} E remover também os registros equivalentes nas demais tabelas relacionadas?`
      : `Excluir o registro #${row.id} apenas desta tabela?`;
    if (typeof window.confirm === 'function' && !window.confirm(confirmMsg)) {
      return;
    }

    try {
      const res = await apiFetch(
        `/egressos/rows/${row.id}?propagate=${useProp ? 'true' : 'false'}`,
        { method: 'DELETE' }
      );
      const extra =
        res?.propagated_deleted > 0
          ? ` (+ ${res.propagated_deleted} registros removidos nas tabelas irmãs)`
          : '';
      setFeedback({
        type: 'success',
        text: `Registro #${row.id} excluído${extra}.`,
      });
      await loadRows();
      await loadCatalog();
    } catch (err) {
      setFeedback({ type: 'error', text: err?.message ?? String(err) });
    }
  }

  async function openEnvelopeEditor() {
    setEnvelopeError(null);
    try {
      const env = await apiFetch(`/egressos/files/${activeFileKey}/envelope`);
      setEnvelopeText(JSON.stringify(env || {}, null, 2));
      setEnvelopeOpen(true);
      setDirty(true);
    } catch (err) {
      setFeedback({ type: 'error', text: err?.message ?? String(err) });
    }
  }

  async function handleSaveEnvelope(e) {
    e.preventDefault();
    setEnvelopeError(null);
    let parsed;
    try {
      parsed = JSON.parse(envelopeText);
      if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
        setEnvelopeError('Os metadados devem ser um objeto JSON.');
        return;
      }
    } catch {
      setEnvelopeError('JSON de metadados inválido.');
      return;
    }

    try {
      await apiFetch(`/egressos/files/${activeFileKey}/envelope`, {
        method: 'PUT',
        body: JSON.stringify(parsed),
      });
      setEnvelopeOpen(false);
      setDirty(false);
      setFeedback({
        type: 'success',
        text: `Metadados do arquivo ${currentFileDef?.file_name || activeFileKey} salvos com sucesso.`,
      });
    } catch (err) {
      setEnvelopeError(err?.message ?? String(err));
    }
  }

  const files = catalog?.files || [];
  const columns =
    page.columns?.length > 0
      ? page.columns
      : currentFileDef?.columns || ['_key', 'value'];
  const supportsPropagation =
    page.entity_group === 'alumni' || page.entity_group === 'company';

  return (
    <div className="space-y-6">
      {/* Barra das 21 abas primárias (1:1 com os 21 arquivos JSON de dados/) */}
      <div className="bg-white border border-slate-200 rounded-xl shadow-sm p-4">
        <div className="flex items-center justify-between mb-2">
          <span className="text-xs font-semibold uppercase tracking-wider text-slate-500">
            21 Tabelas da Pasta dados/ (1 aba por arquivo JSON)
          </span>
          {currentFileDef && (
            <span className="text-xs font-mono text-slate-500">
              Arquivo: <strong>{currentFileDef.file_name}</strong> ({currentFileDef.total_rows}{' '}
              registros)
            </span>
          )}
        </div>
        <div
          role="tablist"
          aria-label="Arquivos JSON do Egressos"
          className="flex flex-wrap gap-1.5"
        >
          {files.map((f) => {
            const isActive = f.file_key === activeFileKey;
            return (
              <button
                key={f.file_key}
                type="button"
                role="tab"
                aria-selected={isActive}
                onClick={() => handleSelectFileTab(f)}
                className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                  isActive
                    ? 'bg-emerald-600 text-white shadow-sm'
                    : 'bg-slate-100 text-slate-700 hover:bg-slate-200'
                }`}
              >
                {f.title} ({f.total_rows})
              </button>
            );
          })}
        </div>

        {/* Sub-abas para arquivos com múltiplas seções (Clarification Q1) */}
        {currentFileDef && currentFileDef.sections?.length > 1 && (
          <div className="mt-4 pt-3 border-t border-slate-100 flex flex-wrap items-center gap-2">
            <span className="text-xs font-medium text-slate-500 mr-1">Seções do arquivo:</span>
            {currentFileDef.sections.map((sec) => {
              const secActive = sec.section_key === effectiveSection;
              return (
                <button
                  key={sec.section_key}
                  type="button"
                  onClick={() => handleSelectSection(sec.section_key)}
                  className={`px-2.5 py-1 rounded text-xs font-medium transition-colors ${
                    secActive
                      ? 'bg-slate-800 text-white'
                      : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
                  }`}
                >
                  Seção: {sec.section_key} ({sec.count})
                </button>
              );
            })}
          </div>
        )}
      </div>

      {/* Barra de ferramentas: busca, propagação na exclusão, metadados e novo registro */}
      <div className="bg-white border border-slate-200 rounded-xl shadow-sm p-4 flex flex-wrap items-center justify-between gap-4">
        <form onSubmit={handleSearchSubmit} className="flex items-center gap-2 flex-1 min-w-[240px]">
          <input
            type="text"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            placeholder="Buscar nesta seção..."
            className="border border-slate-300 rounded-lg px-3 py-1.5 text-sm flex-1 focus:outline-none focus:ring-2 focus:ring-emerald-500"
          />
          <button
            type="submit"
            className="bg-slate-800 text-white px-3 py-1.5 rounded-lg text-sm font-medium hover:bg-slate-900"
          >
            Buscar
          </button>
          {appliedSearch && (
            <button
              type="button"
              onClick={() => {
                setSearchInput('');
                setAppliedSearch('');
                setOffset(0);
              }}
              className="text-xs text-slate-500 hover:text-slate-800"
            >
              Limpar
            </button>
          )}
        </form>

        <div className="flex flex-wrap items-center gap-3">
          {supportsPropagation && (
            <label className="flex items-center gap-1.5 text-xs text-slate-700 bg-emerald-50 border border-emerald-200 px-2.5 py-1.5 rounded-lg">
              <input
                type="checkbox"
                checked={propagateDelete}
                onChange={(e) => setPropagateDelete(e.target.checked)}
              />
              Propagar exclusão entre tabelas ({page.entity_group === 'alumni' ? 'Egresso' : 'Empresa'})
            </label>
          )}

          {currentFileDef && currentFileDef.kind !== 'root_map' && (
            <button
              type="button"
              onClick={openEnvelopeEditor}
              className="border border-slate-300 text-slate-700 px-3 py-1.5 rounded-lg text-sm font-medium hover:bg-slate-50"
            >
              Metadados do arquivo
            </button>
          )}

          <button
            type="button"
            onClick={openCreateModal}
            className="bg-emerald-600 text-white px-4 py-1.5 rounded-lg text-sm font-medium hover:bg-emerald-700"
          >
            + Novo registro
          </button>
        </div>
      </div>

      {feedback && (
        <div
          className={`p-3 rounded-lg text-sm border ${
            feedback.type === 'error'
              ? 'bg-red-50 text-red-700 border-red-200'
              : 'bg-emerald-50 text-emerald-800 border-emerald-200'
          }`}
        >
          {feedback.text}
        </div>
      )}

      {/* Tabela CRUD */}
      <div className="bg-white border border-slate-200 rounded-xl shadow-sm overflow-hidden">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-slate-200 text-sm">
            <thead className="bg-slate-50">
              <tr>
                <th className="px-3 py-2.5 text-left text-xs font-semibold text-slate-500 uppercase">
                  #ID
                </th>
                {columns.map((col) => (
                  <th
                    key={col}
                    className="px-3 py-2.5 text-left text-xs font-semibold text-slate-600 uppercase"
                  >
                    <button
                      type="button"
                      aria-label={`Ordenar por ${col}`}
                      onClick={() => handleSort(col)}
                      className="flex items-center gap-1 hover:text-emerald-700"
                    >
                      <span>{col}</span>
                      {sortCol === col && (
                        <span>{sortOrder === 'asc' ? '▲' : '▼'}</span>
                      )}
                    </button>
                  </th>
                ))}
                <th className="px-3 py-2.5 text-right text-xs font-semibold text-slate-500 uppercase">
                  Ações
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-200 bg-white">
              {loading ? (
                <tr>
                  <td colSpan={columns.length + 2} className="px-4 py-8 text-center text-slate-500">
                    Carregando registros...
                  </td>
                </tr>
              ) : page.items.length === 0 ? (
                <tr>
                  <td colSpan={columns.length + 2} className="px-4 py-8 text-center text-slate-500">
                    Nenhum registro encontrado nesta seção.
                  </td>
                </tr>
              ) : (
                page.items.map((row) => (
                  <tr key={row.id} className="hover:bg-slate-50">
                    <td className="px-3 py-2 font-mono text-xs text-slate-500">{row.id}</td>
                    {columns.map((col) => (
                      <td
                        key={col}
                        className="px-3 py-2 text-slate-800 max-w-xs truncate"
                        title={formatCellValue(row[col])}
                      >
                        {formatCellValue(row[col])}
                      </td>
                    ))}
                    <td className="px-3 py-2 text-right whitespace-nowrap space-x-2">
                      <button
                        type="button"
                        onClick={() => openEditModal(row)}
                        className="text-emerald-700 hover:text-emerald-900 font-medium text-xs"
                      >
                        Editar #{row.id}
                      </button>
                      <button
                        type="button"
                        onClick={() => handleDeleteRow(row)}
                        className="text-red-600 hover:text-red-800 font-medium text-xs"
                      >
                        Excluir #{row.id}
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        {/* Paginação */}
        <div className="px-4 py-3 bg-slate-50 border-t border-slate-200 flex items-center justify-between text-xs text-slate-600">
          <span>
            Exibindo {page.items.length} de {page.total} registros na seção{' '}
            <code>{effectiveSection}</code>
          </span>
          <div className="flex items-center gap-2">
            <button
              type="button"
              disabled={offset === 0}
              onClick={() => setOffset(Math.max(0, offset - 50))}
              className="px-2.5 py-1 rounded border border-slate-300 bg-white disabled:opacity-40"
            >
              Anterior
            </button>
            <button
              type="button"
              disabled={offset + 50 >= page.total}
              onClick={() => setOffset(offset + 50)}
              className="px-2.5 py-1 rounded border border-slate-300 bg-white disabled:opacity-40"
            >
              Próxima
            </button>
          </div>
        </div>
      </div>

      {/* Modal de Criar / Editar Registro */}
      {modalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-xl shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-bold text-slate-900">
                {editingRowId === null
                  ? `Novo registro em ${currentFileDef?.title || activeFileKey} (${effectiveSection})`
                  : `Editar registro #${editingRowId}`}
              </h2>
              <button
                type="button"
                onClick={() => setUseRawJsonMode((v) => !v)}
                className="text-xs text-emerald-700 hover:underline font-medium"
              >
                {useRawJsonMode ? 'Modo formulário por campos' : 'Modo JSON completo'}
              </button>
            </div>

            <form onSubmit={handleSaveRecord} className="space-y-4">
              {!useRawJsonMode ? (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                  {Array.from(
                    new Set([...columns, ...Object.keys(formValues)])
                  ).map((field) => {
                    const val = formValues[field];
                    const isComplex =
                      val !== null && typeof val === 'object';
                    if (isComplex) {
                      return (
                        <div key={field} className="md:col-span-2">
                          <label className="block text-xs font-semibold text-slate-600 mb-1">
                            Campo {field} (JSON estruturado)
                          </label>
                          <textarea
                            aria-label={`Campo ${field}`}
                            rows={3}
                            value={JSON.stringify(val)}
                            onChange={(e) => {
                              try {
                                handleFieldChange(field, JSON.parse(e.target.value));
                              } catch {
                                // aguarda JSON válido
                              }
                            }}
                            className="w-full border border-slate-300 rounded px-2.5 py-1.5 text-xs font-mono"
                          />
                        </div>
                      );
                    }
                    return (
                      <div key={field}>
                        <label
                          htmlFor={`field-${field}`}
                          className="block text-xs font-semibold text-slate-600 mb-1"
                        >
                          Campo {field}
                        </label>
                        <input
                          id={`field-${field}`}
                          aria-label={`Campo ${field}`}
                          type="text"
                          value={val === null || val === undefined ? '' : String(val)}
                          onChange={(e) => handleFieldChange(field, e.target.value)}
                          className="w-full border border-slate-300 rounded px-2.5 py-1.5 text-sm"
                        />
                      </div>
                    );
                  })}
                </div>
              ) : (
                <div>
                  <label className="block text-xs font-semibold text-slate-600 mb-1">
                    Objeto JSON completo do registro
                  </label>
                  <textarea
                    rows={12}
                    value={rawJsonDraft}
                    onChange={(e) => setRawJsonDraft(e.target.value)}
                    className="w-full border border-slate-300 rounded p-2.5 text-xs font-mono"
                  />
                </div>
              )}

              {/* Opção de propagação entre tabelas relacionadas (FR-008a / Q2) */}
              {editingRowId !== null && supportsPropagation && (
                <div className="p-3 rounded-lg bg-emerald-50 border border-emerald-200">
                  <label className="flex items-start gap-2 text-xs text-emerald-900 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={propagateEdit}
                      onChange={(e) => setPropagateEdit(e.target.checked)}
                      className="mt-0.5"
                    />
                    <span>
                      <strong>Propagar alterações para tabelas relacionadas</strong> — atualiza
                      automaticamente os campos compartilhados deste{' '}
                      {page.entity_group === 'alumni'
                        ? 'egresso (nome, curso, cargo, empresa, local)'
                        : 'registro de empresa (nome, porte, setor, origem, URL)'}{' '}
                      nas demais abas do projeto Egressos. Desmarque para editar apenas nesta
                      tabela.
                    </span>
                  </label>
                </div>
              )}

              {modalError && (
                <div className="p-2.5 rounded bg-red-50 text-red-700 text-xs border border-red-200">
                  {modalError}
                </div>
              )}

              <div className="flex justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={closeRecordModal}
                  className="px-4 py-1.5 rounded border border-slate-300 text-sm text-slate-700 hover:bg-slate-50"
                >
                  Cancelar
                </button>
                <button
                  type="submit"
                  className="px-4 py-1.5 rounded bg-emerald-600 text-white text-sm font-medium hover:bg-emerald-700"
                >
                  Salvar registro
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Modal de Metadados do Arquivo (envelope FR-009) */}
      {envelopeOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-xl shadow-xl max-w-2xl w-full p-6">
            <h2 className="text-lg font-bold text-slate-900 mb-2">
              Metadados do arquivo {currentFileDef?.file_name}
            </h2>
            <p className="text-xs text-slate-500 mb-4">
              Edite os metadados de nível de arquivo (ex.: <code>titulo</code>,{' '}
              <code>subtitulo</code>, <code>gerado_em</code>, <code>nota_metodologica</code>). As
              seções de registros são preservadas automaticamente na exportação.
            </p>
            <form onSubmit={handleSaveEnvelope} className="space-y-4">
              <textarea
                aria-label="JSON de metadados do arquivo"
                rows={12}
                value={envelopeText}
                onChange={(e) => setEnvelopeText(e.target.value)}
                className="w-full border border-slate-300 rounded p-3 text-xs font-mono"
              />
              {envelopeError && (
                <div className="p-2.5 rounded bg-red-50 text-red-700 text-xs border border-red-200">
                  {envelopeError}
                </div>
              )}
              <div className="flex justify-end gap-2">
                <button
                  type="button"
                  onClick={() => {
                    setEnvelopeOpen(false);
                    setDirty(false);
                  }}
                  className="px-4 py-1.5 rounded border border-slate-300 text-sm text-slate-700"
                >
                  Cancelar
                </button>
                <button
                  type="submit"
                  className="px-4 py-1.5 rounded bg-emerald-600 text-white text-sm font-medium hover:bg-emerald-700"
                >
                  Salvar metadados
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
