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

import EgressosDadosTablesPage from '../../src/components/EgressosDadosTablesPage.jsx';

const SAMPLE_CATALOG = {
  loaded: true,
  imported_at: '1700000000',
  total_files: 39,
  total_rows: 2216,
  default_local_dir: '/home/thiagodeps/egressos-dados',
  default_local_dir_exists: true,
  files: [
    {
      file_key: 'alunos',
      file_name: 'alunos.json',
      relative_path: 'alunos.json',
      title: 'Cadastro Mestre de Alunos (PII)',
      kind: 'object_sections',
      publication: 'pii',
      role: 'entrada',
      primary_section: 'alunos',
      sections: [{ section_key: 'alunos', count: 103 }],
      columns: ['id', 'label', 'nome', 'curso', 'ano_conclusao', 'cargo_atual', 'empresa_atual', 'local'],
      entity_group: 'alumni',
      total_rows: 103,
    },
    {
      file_key: 'agg_pv',
      file_name: 'agg_pv.json',
      relative_path: 'data/agg_pv.json',
      title: 'Agregado Ponto de Vista',
      kind: 'root_array',
      publication: 'interno',
      role: 'saida',
      primary_section: '_root',
      sections: [{ section_key: '_root', count: 12 }],
      columns: ['id', 'curso', 'ano_conclusao', 'empresa_atual', 'cargo_atual', 'remuneracao'],
      entity_group: 'alumni',
      total_rows: 12,
    },
    {
      file_key: 'vitrine',
      file_name: 'vitrine.json',
      relative_path: 'data/vitrine.json',
      title: 'Vitrine de Egressos',
      kind: 'object_sections',
      publication: 'publicavel',
      role: 'saida',
      primary_section: 'egressos',
      sections: [
        { section_key: 'egressos', count: 10 },
        { section_key: 'empresas', count: 5 },
      ],
      columns: ['label', 'nome', 'curso', 'cargo_atual', 'empresa_atual'],
      entity_group: 'alumni',
      total_rows: 15,
    },
  ],
};

describe('EgressosDadosTablesPage (US1 & US2 - 39 Tabs, Filters, Badges & CRUD)', () => {
  beforeEach(() => {
    apiFetchMock.mockReset();
  });

  afterEach(() => {
    cleanup();
  });

  it('renders category filters, relative_path, publication/role badges and rows with preserved string id (T008)', async () => {
    apiFetchMock.mockImplementation(async (endpoint) => {
      if (endpoint === '/egressos-dados/catalog') {
        return SAMPLE_CATALOG;
      }
      if (endpoint.startsWith('/egressos-dados/files/alunos/rows')) {
        return {
          items: [
            {
              _row_id: 10,
              id: 'egresso_demo_1',
              label: 'A',
              nome: 'Egresso Demo 1',
              curso: 'BSI',
              ano_conclusao: 2020,
              cargo_atual: 'Senior Engineer',
              empresa_atual: 'Empresa Demo',
              local: 'Vitória, ES',
            },
          ],
          total: 1,
          limit: 50,
          offset: 0,
          columns: SAMPLE_CATALOG.files[0].columns,
          entity_group: 'alumni',
          is_map_section: false,
        };
      }
      if (endpoint.startsWith('/egressos-dados/files/vitrine/rows')) {
        return {
          items: [
            {
              _row_id: 21,
              label: 'A',
              nome: 'Egresso Demo 1',
              curso: 'BSI',
              cargo_atual: 'Senior Engineer',
              empresa_atual: 'Empresa Demo',
            },
          ],
          total: 1,
          limit: 50,
          offset: 0,
          columns: SAMPLE_CATALOG.files[2].columns,
          entity_group: 'alumni',
          is_map_section: false,
        };
      }
      throw new Error(`Unexpected endpoint: ${endpoint}`);
    });

    render(<EgressosDadosTablesPage />);

    await waitFor(() => {
      expect(screen.getByText(/Egresso Demo 1/i)).toBeDefined();
    });

    // Verifica badges informativos de privacidade (PII) e papel (Entrada)
    expect(screen.getByTestId('badge-publication').textContent).toMatch(/PII/i);
    expect(screen.getByTestId('badge-role').textContent).toMatch(/Entrada/i);
    expect(screen.getByText('alunos.json')).toBeDefined();

    // Verifica que tanto o _row_id numérico (#10) quanto o id textual ('egresso_demo_1') são exibidos
    expect(screen.getByText('egresso_demo_1')).toBeDefined();
    expect(screen.getByRole('button', { name: /Editar #10/i })).toBeDefined();

    // Filtra por categoria Publicáveis e seleciona Vitrine
    fireEvent.click(screen.getByRole('button', { name: /Publicáveis/i }));
    const vitrineTab = screen.getByRole('tab', { name: /Vitrine de Egressos/i });
    fireEvent.click(vitrineTab);

    await waitFor(() => {
      expect(screen.getByTestId('badge-publication').textContent).toMatch(/Publicável/i);
      expect(screen.getByTestId('badge-role').textContent).toMatch(/Saída/i);
    });

    // Verifica sub-abas de seção para arquivo multi-seção
    expect(screen.getByRole('button', { name: /Seção: egressos/i })).toBeDefined();
    expect(screen.getByRole('button', { name: /Seção: empresas/i })).toBeDefined();
  });

  it('creates, edits (preserving string id and supporting propagate flag) and deletes rows across datasets (T014)', async () => {
    window.confirm = vi.fn(() => true);

    apiFetchMock.mockImplementation(async (endpoint, options) => {
      if (endpoint === '/egressos-dados/catalog') {
        return SAMPLE_CATALOG;
      }
      if (endpoint.startsWith('/egressos-dados/files/alunos/rows') && !options?.method) {
        return {
          items: [
            {
              _row_id: 10,
              id: 'egresso_demo_1',
              label: 'A',
              nome: 'Egresso Demo 1',
              curso: 'BSI',
              ano_conclusao: 2020,
              cargo_atual: 'Senior Engineer',
              empresa_atual: 'Empresa Demo',
              local: 'Vitória, ES',
            },
          ],
          total: 1,
          limit: 50,
          offset: 0,
          columns: SAMPLE_CATALOG.files[0].columns,
          entity_group: 'alumni',
          is_map_section: false,
        };
      }
      if (endpoint === '/egressos-dados/rows/10' && !options?.method) {
        return {
          _row_id: 10,
          id: 'egresso_demo_1',
          label: 'A',
          nome: 'Egresso Demo 1',
          curso: 'BSI',
          ano_conclusao: 2020,
          cargo_atual: 'Senior Engineer',
          empresa_atual: 'Empresa Demo',
          local: 'Vitória, ES',
        };
      }
      if (endpoint === '/egressos-dados/rows/10' && options?.method === 'PUT') {
        return {
          record: { _row_id: 10, id: 'egresso_demo_1', cargo_atual: 'Staff Engineer' },
          propagated_count: 5,
        };
      }
      if (endpoint.startsWith('/egressos-dados/rows/10') && options?.method === 'DELETE') {
        return { deleted: 1, propagated_deleted: 2 };
      }
      throw new Error(`Unexpected endpoint: ${endpoint}`);
    });

    render(<EgressosDadosTablesPage />);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Editar #10/i })).toBeDefined();
    });

    // Abre modal de edição do registro #10
    fireEvent.click(screen.getByRole('button', { name: /Editar #10/i }));

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Editar registro #10/i })).toBeDefined();
    });

    // O campo textual "id" ("egresso_demo_1") deve estar preservado no formulário
    const idField = screen.getByLabelText('Campo id');
    expect(idField.value).toBe('egresso_demo_1');

    const cargoField = screen.getByLabelText('Campo cargo_atual');
    fireEvent.change(cargoField, { target: { value: 'Staff Engineer' } });

    // Marca propagação entre tabelas relacionadas
    const propCheckbox = screen.getByRole('checkbox', {
      name: /Propagar alterações para tabelas relacionadas/i,
    });
    fireEvent.click(propCheckbox);

    fireEvent.click(screen.getByRole('button', { name: /Salvar registro/i }));

    await waitFor(() => {
      expect(apiFetchMock).toHaveBeenCalledWith(
        '/egressos-dados/rows/10',
        expect.objectContaining({
          method: 'PUT',
        })
      );
    });

    const putCall = apiFetchMock.mock.calls.find(
      ([ep, opts]) => ep === '/egressos-dados/rows/10' && opts?.method === 'PUT'
    );
    const sentBody = JSON.parse(putCall[1].body);
    expect(sentBody.propagate).toBe(true);
    expect(sentBody.payload.id).toBe('egresso_demo_1');
    expect(sentBody.payload.cargo_atual).toBe('Staff Engineer');

    // Testa exclusão do registro #10
    fireEvent.click(screen.getByRole('button', { name: /Excluir #10/i }));

    await waitFor(() => {
      expect(apiFetchMock).toHaveBeenCalledWith(
        '/egressos-dados/rows/10?propagate=false',
        { method: 'DELETE' }
      );
    });
  });
});
