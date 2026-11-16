/**
 * @vitest-environment jsdom
 */
import React from 'react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, cleanup } from '@testing-library/react';
import EntityForm from '../../src/components/EntityForm.jsx';
import EntityPage from '../../src/components/EntityPage.jsx';

vi.mock('../../src/services/api', () => ({
  apiFetch: vi.fn(),
}));

import { apiFetch } from '../../src/services/api';

afterEach(() => cleanup());

describe('EntityForm (Horizon v2 & Legacy compatibility)', () => {
  it('renders nested objects and arrays in json_readonly fields (such as initiatives.enrichment) in a readable format', () => {
    const enrichmentObj = {
      source: 'sigpesq_project_files',
      project_code: '6903',
      needs_review: false,
      palavras_chave: ['IA', 'Processamento de Linguagem Natural'],
      objetivos: {
        geral: 'Desenvolver modelos de linguagem aplicados',
        especificos: ['Mapear corpus institucional', 'Avaliar acurácia'],
      },
    };

    const fields = [
      { name: 'id', label: 'ID', type: 'number' },
      { name: 'name', label: 'Nome', type: 'text' },
      { name: 'enrichment', label: 'Enriquecimento SIGPesq ( Vínculo )', type: 'json_readonly' },
    ];

    render(
      <EntityForm
        initialData={{
          id: 42,
          name: 'Projeto Horizon IA',
          enrichment: JSON.stringify(enrichmentObj),
        }}
        fields={fields}
        basePath="/horizon"
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    expect(screen.getByText(/sigpesq_project_files/i)).toBeTruthy();
    expect(screen.getByText(/6903/)).toBeTruthy();
    expect(screen.getByText(/IA, Processamento de Linguagem Natural/)).toBeTruthy();
    expect(screen.getByText(/Desenvolver modelos de linguagem aplicados/)).toBeTruthy();
    expect(screen.getByText(/Mapear corpus institucional/)).toBeTruthy();
    expect(screen.queryByText(/\[object Object\]/)).toBeNull();
  });

  it('scopes relationship links to basePath (/horizon vs /dashboard), maps singular/alias columns to valid routes, and renders non-routable objects as plain text', () => {
    const fields = [
      { name: 'organization', label: 'Organização ( Vínculo )', type: 'json_readonly' },
      { name: 'research_group', label: 'Grupo de Pesquisa ( Vínculo )', type: 'json_readonly' },
      { name: 'team', label: 'Equipe ( Vínculos )', type: 'json_readonly' },
      { name: 'initiative_type', label: 'Tipo de Iniciativa ( Vínculo )', type: 'json_readonly' },
    ];

    render(
      <EntityForm
        initialData={{
          id: 10,
          organization: JSON.stringify({ id: 1, name: 'IFES' }),
          research_group: JSON.stringify({ id: 7, name: 'LEDS' }),
          team: JSON.stringify([{ id: 99, name: 'Maria Pesquisadora' }]),
          initiative_type: JSON.stringify({ id: 2, name: 'Pesquisa Aplicada' }),
        }}
        fields={fields}
        basePath="/horizon"
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    const orgLink = screen.getByRole('link', { name: /ID: 1 - IFES/i });
    expect(orgLink.getAttribute('href')).toBe('/horizon/organizations?openId=1');

    const groupLink = screen.getByRole('link', { name: /ID: 7 - LEDS/i });
    expect(groupLink.getAttribute('href')).toBe('/horizon/groups?openId=7');

    const teamLink = screen.getByRole('link', { name: /ID: 99 - Maria Pesquisadora/i });
    expect(teamLink.getAttribute('href')).toBe('/horizon/researchers?openId=99');

    // initiative_type has no dedicated entity page route, so it must NOT render a broken link
    expect(screen.queryByRole('link', { name: /ID: 2 - Pesquisa Aplicada/i })).toBeNull();
    expect(screen.getByText(/Pesquisa Aplicada/)).toBeTruthy();
  });
});

describe('EntityPage (apiPrefix and basePath support)', () => {
  beforeEach(() => {
    apiFetch.mockReset();
  });

  it('prefixes API requests with apiPrefix="/horizon-v2" when configured', async () => {
    apiFetch.mockResolvedValueOnce({
      items: [{ id: 1, name: 'Iniciativa V2' }],
      total: 1,
    });

    render(
      <EntityPage
        entity="initiatives"
        apiPrefix="/horizon-v2"
        basePath="/horizon"
        columns={['id', 'name']}
        fields={[
          { name: 'id', label: 'ID', type: 'number' },
          { name: 'name', label: 'Nome', type: 'text' },
        ]}
      />,
    );

    await waitFor(() => {
      expect(apiFetch).toHaveBeenCalledWith(
        '/horizon-v2/initiatives?limit=50&offset=0',
        expect.any(Object),
      );
    });
  });
});
