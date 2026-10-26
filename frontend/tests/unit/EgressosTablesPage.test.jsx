/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react';

const mockApiFetch = vi.fn();
vi.mock('../../src/services/api.js', () => ({
  apiFetch: (...args) => mockApiFetch(...args),
}));

import EgressosTablesPage from '../../src/components/EgressosTablesPage.jsx';

const TWENTY_ONE_FILES = [
  {
    file_key: 'egressos_perfil',
    file_name: 'egressos_perfil.json',
    title: 'Perfis de Egressos',
    kind: 'object_sections',
    primary_section: 'egressos',
    sections: [
      { section_key: 'egressos', count: 50, entity_group: 'alumni' },
      { section_key: 'cursos', count: 2, entity_group: 'none' },
    ],
    columns: ['label', 'nome', 'curso', 'cargo_atual', 'empresa_atual', 'local_atual', 'area_atual'],
    entity_group: 'alumni',
    total_rows: 52,
  },
  {
    file_key: 'vitrine',
    file_name: 'vitrine.json',
    title: 'Vitrine de Egressos',
    kind: 'object_sections',
    primary_section: 'perfis',
    sections: [
      { section_key: 'perfis', count: 50, entity_group: 'alumni' },
      { section_key: 'empresas', count: 40, entity_group: 'company' },
    ],
    columns: ['rotulo', 'nome', 'curso', 'cargo', 'empresa', 'lugar'],
    entity_group: 'alumni',
    total_rows: 90,
  },
  { file_key: 'panorama', file_name: 'panorama.json', title: 'Panorama do Coorte', kind: 'object_sections', primary_section: 'coorte', sections: [{ section_key: 'coorte', count: 50, entity_group: 'alumni' }], columns: ['rotulo', 'cargo_atual'], entity_group: 'alumni', total_rows: 50 },
  { file_key: 'analise', file_name: 'analise.json', title: 'Análise do Grupo', kind: 'object_sections', primary_section: 'rows', sections: [{ section_key: 'rows', count: 50, entity_group: 'alumni' }], columns: ['label', 'curso'], entity_group: 'alumni', total_rows: 50 },
  { file_key: 'consolidado', file_name: 'consolidado.json', title: 'Consolidado Salarial', kind: 'object_sections', primary_section: 'perfis', sections: [{ section_key: 'perfis', count: 24, entity_group: 'alumni' }], columns: ['perfil', 'trilha'], entity_group: 'alumni', total_rows: 24 },
  { file_key: 'empresas_porte', file_name: 'empresas_porte.json', title: 'Empresas — Porte e Setor', kind: 'root_map', primary_section: '_root', sections: [{ section_key: '_root', count: 162, entity_group: 'company' }], columns: ['_key', 'empresa', 'porte'], entity_group: 'company', total_rows: 162 },
  { file_key: 'empresas_aliases', file_name: 'empresas_aliases.json', title: 'Empresas — Aliases', kind: 'root_map', primary_section: '_root', sections: [{ section_key: '_root', count: 162, entity_group: 'company' }], columns: ['_key', 'aliases'], entity_group: 'company', total_rows: 162 },
  { file_key: 'empresas_linkedin_data', file_name: 'empresas_linkedin_data.json', title: 'Empresas — Dados LinkedIn', kind: 'root_map', primary_section: '_root', sections: [{ section_key: '_root', count: 147, entity_group: 'company' }], columns: ['_key', 'nome_oficial'], entity_group: 'company', total_rows: 147 },
  { file_key: 'empresas_linkedin_urls', file_name: 'empresas_linkedin_urls.json', title: 'Empresas — URLs LinkedIn', kind: 'root_map', primary_section: '_root', sections: [{ section_key: '_root', count: 162, entity_group: 'company' }], columns: ['_key', 'url'], entity_group: 'company', total_rows: 162 },
  { file_key: 'fapes_fomento', file_name: 'fapes_fomento.json', title: 'Fomento FAPES / PRODEST', kind: 'object_sections', primary_section: 'projetos', sections: [{ section_key: 'projetos', count: 15, entity_group: 'none' }], columns: ['projeto_id', 'titulo'], entity_group: 'none', total_rows: 15 },
  { file_key: 'impacto', file_name: 'impacto.json', title: 'Indicadores de Impacto', kind: 'object_sections', primary_section: 'indicadores', sections: [{ section_key: 'indicadores', count: 6, entity_group: 'none' }], columns: ['rotulo', 'valor'], entity_group: 'none', total_rows: 6 },
  { file_key: 'trajetoria', file_name: 'trajetoria.json', title: 'Trajetória Salarial', kind: 'object_sections', primary_section: 'serie', sections: [{ section_key: 'serie', count: 12, entity_group: 'none' }], columns: ['ano', 'med_real'], entity_group: 'none', total_rows: 12 },
  { file_key: 'renda_por_senioridade', file_name: 'renda_por_senioridade.json', title: 'Renda por Senioridade', kind: 'object_sections', primary_section: 'cargos', sections: [{ section_key: 'cargos', count: 5, entity_group: 'none' }], columns: ['nivel', 'referencias'], entity_group: 'none', total_rows: 5 },
  { file_key: 'cargos_ao_longo_do_tempo', file_name: 'cargos_ao_longo_do_tempo.json', title: 'Cargos ao Longo do Tempo', kind: 'object_sections', primary_section: 'cargos', sections: [{ section_key: 'cargos', count: 4, entity_group: 'none' }], columns: ['cargo', 'nome'], entity_group: 'none', total_rows: 4 },
  { file_key: 'codigofonte_2026', file_name: 'codigofonte_2026.json', title: 'Pesquisa Código Fonte 2026', kind: 'object_sections', primary_section: 'por_senioridade', sections: [{ section_key: 'por_senioridade', count: 5, entity_group: 'none' }], columns: ['nivel', 'media'], entity_group: 'none', total_rows: 5 },
  { file_key: 'codigofonte_historico', file_name: 'codigofonte_historico.json', title: 'Pesquisa Código Fonte — Histórico', kind: 'nested_map', primary_section: 'media_por_senioridade', sections: [{ section_key: 'media_por_senioridade', count: 6, entity_group: 'none' }], columns: ['_key', 'Júnior'], entity_group: 'none', total_rows: 6 },
  { file_key: 'salario_minimo', file_name: 'salario_minimo.json', title: 'Salário Mínimo Nacional', kind: 'object_sections', primary_section: 'por_ano', sections: [{ section_key: 'por_ano', count: 27, entity_group: 'none' }], columns: ['ano', 'media_ponderada'], entity_group: 'none', total_rows: 27 },
  { file_key: 'ibge_series', file_name: 'ibge_series.json', title: 'Séries IBGE / Macroeconômicas', kind: 'nested_map', primary_section: 'series', sections: [{ section_key: 'series', count: 4, entity_group: 'none' }], columns: ['_key', 'fonte'], entity_group: 'none', total_rows: 4 },
  { file_key: 'so_benchmarks', file_name: 'so_benchmarks.json', title: 'Stack Overflow Benchmarks', kind: 'object_sections', primary_section: 'por_pais', sections: [{ section_key: 'por_pais', count: 16, entity_group: 'none' }], columns: ['pais', 'usd_mes'], entity_group: 'none', total_rows: 16 },
  { file_key: 'recorte_cursos', file_name: 'recorte_cursos.json', title: 'Recorte por Cursos (BSI e ECA)', kind: 'nested_map', primary_section: 'cursos', sections: [{ section_key: 'cursos', count: 2, entity_group: 'none' }], columns: ['_key', 'nome'], entity_group: 'none', total_rows: 2 },
  { file_key: 'mapa_mundi', file_name: 'mapa_mundi.json', title: 'Mapa-Múndi (Camada Base)', kind: 'object_sections', primary_section: 'paths', sections: [{ section_key: 'paths', count: 177, entity_group: 'none' }], columns: ['value'], entity_group: 'none', total_rows: 177 },
];

describe('EgressosTablesPage', () => {
  beforeEach(() => {
    mockApiFetch.mockReset();
    window.confirm = vi.fn(() => true);

    mockApiFetch.mockImplementation((endpoint, options) => {
      if (endpoint === '/egressos/catalog') {
        return Promise.resolve({
          loaded: true,
          imported_at: '1761156000',
          total_files: 21,
          total_rows: 1142,
          default_local_dir: '/home/thiagodeps/egressos/dados',
          default_local_dir_exists: true,
          files: TWENTY_ONE_FILES,
        });
      }
      if (endpoint.startsWith('/egressos/files/egressos_perfil/rows')) {
        if (options?.method === 'POST') {
          return Promise.resolve({ id: 99, label: 'ZZ', nome: 'Novo Egresso' });
        }
        return Promise.resolve({
          items: [
            {
              id: 1,
              _record_key: 'A',
              label: 'A',
              nome: 'Alice Silva',
              curso: 'bsi',
              cargo_atual: 'Senior Developer',
              empresa_atual: 'AEVO',
              local_atual: 'Vitória, ES',
              area_atual: 'Engenharia de Software',
            },
          ],
          total: 1,
          limit: 50,
          offset: 0,
          file_key: 'egressos_perfil',
          section_key: 'egressos',
          columns: ['label', 'nome', 'curso', 'cargo_atual', 'empresa_atual', 'local_atual', 'area_atual'],
          entity_group: 'alumni',
          is_map_section: false,
        });
      }
      if (endpoint.startsWith('/egressos/files/vitrine/rows')) {
        const isEmpresas = endpoint.includes('section=empresas');
        return Promise.resolve({
          items: isEmpresas
            ? [{ id: 10, _record_key: 'AEVO', nome: 'AEVO', n: 5 }]
            : [{ id: 2, _record_key: 'A', rotulo: 'A', nome: 'Alice Silva', cargo: 'Senior Developer', empresa: 'AEVO' }],
          total: 1,
          limit: 50,
          offset: 0,
          file_key: 'vitrine',
          section_key: isEmpresas ? 'empresas' : 'perfis',
          columns: isEmpresas ? ['nome', 'n'] : ['rotulo', 'nome', 'cargo', 'empresa'],
          entity_group: isEmpresas ? 'company' : 'alumni',
          is_map_section: false,
        });
      }
      if (endpoint === '/egressos/files/egressos_perfil/envelope') {
        if (options?.method === 'PUT') {
          return Promise.resolve({ titulo: 'Perfis Atualizados', total_perfis: 50 });
        }
        return Promise.resolve({ titulo: 'Perfis Originais', total_perfis: 50 });
      }
      if (endpoint.startsWith('/egressos/rows/1')) {
        if (options?.method === 'PUT') {
          return Promise.resolve({
            record: { id: 1, label: 'A', nome: 'Alice Editada', cargo_atual: 'Staff Engineer' },
            propagated_count: 3,
          });
        }
        if (options?.method === 'DELETE') {
          return Promise.resolve({ deleted: 1, propagated_deleted: 2 });
        }
        return Promise.resolve({
          id: 1,
          label: 'A',
          nome: 'Alice Silva',
          curso: 'bsi',
          cargo_atual: 'Senior Developer',
          empresa_atual: 'AEVO',
        });
      }
      return Promise.resolve({});
    });
  });

  afterEach(() => cleanup());

  it('renders all 21 primary file tabs and loads the active table rows', async () => {
    render(<EgressosTablesPage />);
    expect(await screen.findByText('Alice Silva')).toBeDefined();
    const tabs = screen.getAllByRole('tab');
    expect(tabs.length).toBe(21);
  });

  it('switches primary tab and sub-section tab in multi-section files (e.g., vitrine -> empresas)', async () => {
    render(<EgressosTablesPage />);
    await screen.findByText('Alice Silva');

    const vitrineTab = screen.getByRole('tab', { name: /vitrine de egressos/i });
    fireEvent.click(vitrineTab);

    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith(
        expect.stringContaining('/egressos/files/vitrine/rows?section=perfis')
      )
    );

    const empresasSubTab = await screen.findByRole('button', { name: /seção: empresas/i });
    fireEvent.click(empresasSubTab);

    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith(
        expect.stringContaining('/egressos/files/vitrine/rows?section=empresas')
      )
    );
  });

  it('triggers search and column sorting on the active section', async () => {
    render(<EgressosTablesPage />);
    await screen.findByText('Alice Silva');

    const searchInput = screen.getByPlaceholderText(/buscar nesta seção/i);
    fireEvent.change(searchInput, { target: { value: 'AEVO' } });
    fireEvent.submit(searchInput.closest('form'));

    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith(
        expect.stringContaining('search=AEVO')
      )
    );

    const colBtn = screen.getByRole('button', { name: /ordenar por label/i });
    fireEvent.click(colBtn);

    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith(
        expect.stringContaining('sort=label')
      )
    );
  });

  it('creates, edits (with propagate option FR-008a) and deletes a record', async () => {
    render(<EgressosTablesPage />);
    await screen.findByText('Alice Silva');

    // 1. Create record
    fireEvent.click(screen.getByRole('button', { name: /\+ novo registro/i }));
    const labelInput = await screen.findByLabelText('Campo label');
    fireEvent.change(labelInput, { target: { value: 'ZZ' } });
    fireEvent.click(screen.getByRole('button', { name: /salvar registro/i }));

    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith(
        '/egressos/files/egressos_perfil/rows',
        expect.objectContaining({ method: 'POST' })
      )
    );

    // 2. Edit record with propagate checkbox (Clarification Q2 / FR-008a)
    fireEvent.click(await screen.findByRole('button', { name: /editar #1/i }));
    const propagateCheckbox = await screen.findByRole('checkbox', {
      name: /propagar alterações para tabelas relacionadas/i,
    });
    expect(propagateCheckbox).toBeDefined();
    fireEvent.click(propagateCheckbox);
    fireEvent.click(screen.getByRole('button', { name: /salvar registro/i }));

    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith(
        '/egressos/rows/1',
        expect.objectContaining({
          method: 'PUT',
          body: expect.stringContaining('"propagate":true'),
        })
      )
    );

    // 3. Delete record
    fireEvent.click(await screen.findByRole('button', { name: /excluir #1/i }));
    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith(
        expect.stringMatching(/\/egressos\/rows\/1\?propagate=(true|false)/),
        expect.objectContaining({ method: 'DELETE' })
      )
    );
  });

  it('opens and saves the file envelope metadata editor (FR-009)', async () => {
    render(<EgressosTablesPage />);
    await screen.findByText('Alice Silva');

    fireEvent.click(screen.getByRole('button', { name: /metadados do arquivo/i }));
    const envTextarea = await screen.findByLabelText(/json de metadados do arquivo/i);
    expect(envTextarea.value).toContain('Perfis Originais');

    fireEvent.change(envTextarea, {
      target: { value: JSON.stringify({ titulo: 'Perfis Atualizados', total_perfis: 50 }, null, 2) },
    });
    fireEvent.click(screen.getByRole('button', { name: /salvar metadados/i }));

    await waitFor(() =>
      expect(mockApiFetch).toHaveBeenCalledWith(
        '/egressos/files/egressos_perfil/envelope',
        expect.objectContaining({ method: 'PUT' })
      )
    );
  });
});
