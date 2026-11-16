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

import AnalyticsDashboard from '../../src/components/AnalyticsDashboard.jsx';
import TableAnalyticsPanel from '../../src/components/TableAnalyticsPanel.jsx';

describe('AnalyticsDashboard & TableAnalyticsPanel (SEP-040 FR-006..FR-010)', () => {
  beforeEach(() => {
    apiFetchMock.mockReset();
  });

  afterEach(() => {
    cleanup();
  });

  it('renders empty state when loaded is false', async () => {
    apiFetchMock.mockResolvedValue({
      scope: 'horizon-v2',
      entity: null,
      title: 'Dashboard Analítico — Horizon',
      loaded: false,
      total_rows: 0,
      kpis: [],
      charts: [],
    });

    render(<AnalyticsDashboard endpoint="/horizon-v2/analytics" />);
    await waitFor(() => {
      expect(screen.getByTestId('analytics-empty-state')).toBeDefined();
    });
    expect(screen.getByText(/Nenhum dado carregado/i)).toBeDefined();
  });

  it('renders KPIs and chart sections (project status, fellowships, institutions) when loaded', async () => {
    apiFetchMock.mockResolvedValue({
      scope: 'horizon-v2',
      entity: null,
      title: 'Dashboard Analítico — Horizon',
      loaded: true,
      total_rows: 70545,
      kpis: [
        {
          label: 'Projetos Ativos',
          value: '1.943',
          sublabel: '12,5% de 15.546 iniciativas',
          tone: 'emerald',
        },
        {
          label: 'Investimento Mensal em Bolsas',
          value: 'R$ 324.920,00',
          sublabel: '473 bolsas remuneradas',
          tone: 'amber',
        },
      ],
      charts: [
        {
          id: 'project_status',
          title: 'Status dos Projetos (Iniciativas: Ativos vs. Inativos)',
          subtitle: 'Total de 15.546 iniciativas',
          kind: 'donut',
          items: [
            {
              label: 'Ativo (Em Execução)',
              count: 1084,
              secondary_value: null,
              formatted_value: '1.084 projetos',
              percentage: 7.0,
            },
          ],
        },
        {
          id: 'top_supervisors',
          title: 'Quanto Supervisionam os Pesquisadores Orientadores',
          subtitle: null,
          kind: 'financial_table',
          items: [
            {
              label: 'Prof. Thiago',
              count: 5,
              secondary_value: 3500.0,
              formatted_value: 'R$ 3.500,00/mês (5 bolsas)',
              percentage: 100.0,
            },
          ],
        },
      ],
    });

    render(<AnalyticsDashboard endpoint="/horizon-v2/analytics" />);
    await waitFor(() => {
      expect(screen.getByText('Projetos Ativos')).toBeDefined();
    });
    expect(screen.getByText('1.943')).toBeDefined();
    expect(screen.getByText('R$ 324.920,00')).toBeDefined();
    expect(screen.getByText(/Status dos Projetos/i)).toBeDefined();
    expect(screen.getByText(/Prof\. Thiago/i)).toBeDefined();
  });

  it('toggles TableAnalyticsPanel and fetches entity-specific charts on demand', async () => {
    apiFetchMock.mockResolvedValue({
      scope: 'horizon-v2',
      entity: 'initiatives',
      title: 'Gráficos da Tabela — Iniciativas (Projetos)',
      loaded: true,
      total_rows: 15546,
      kpis: [
        {
          label: 'Projetos Ativos',
          value: '1.943',
          sublabel: '12,5% do total',
          tone: 'emerald',
        },
      ],
      charts: [
        {
          id: 'initiatives_status',
          title: 'Status das Iniciativas (Ativos vs. Inativos)',
          subtitle: null,
          kind: 'donut',
          items: [
            {
              label: 'Ativo (Em Execução)',
              count: 1084,
              secondary_value: null,
              formatted_value: '1.084 projetos',
              percentage: 7.0,
            },
          ],
        },
      ],
    });

    render(<TableAnalyticsPanel endpoint="/horizon-v2/analytics?entity=initiatives" label="Iniciativas" />);
    const toggleBtn = screen.getByTestId('toggle-table-analytics-btn');
    fireEvent.click(toggleBtn);

    await waitFor(() => {
      expect(apiFetchMock).toHaveBeenCalledWith('/horizon-v2/analytics?entity=initiatives');
      expect(screen.getByText(/Status das Iniciativas/i)).toBeDefined();
    });
  });

  it('re-fetches analytics with serialized filters when a rule is added in FilterBuilderBar (SEP-041 T016)', async () => {
    apiFetchMock.mockResolvedValue({
      scope: 'horizon-v2',
      entity: 'initiatives',
      title: 'Gráficos da Tabela — Iniciativas',
      loaded: true,
      total_rows: 100,
      kpis: [],
      charts: [],
      available_entities: [
        {
          entity: 'initiatives',
          label: 'initiatives',
          columns: ['status', 'campus_name', 'research_groups'],
        },
      ],
    });

    render(<AnalyticsDashboard endpoint="/horizon-v2/analytics?entity=initiatives" />);
    await waitFor(() => {
      expect(screen.getByTestId('filter-builder-bar')).toBeDefined();
    });

    fireEvent.change(screen.getByTestId('filter-column-select'), {
      target: { value: 'status' },
    });
    fireEvent.change(screen.getByTestId('filter-operator-select'), {
      target: { value: 'equals' },
    });
    fireEvent.change(screen.getByTestId('filter-value-input'), {
      target: { value: 'Active' },
    });
    fireEvent.click(screen.getByTestId('add-filter-rule-btn'));

    const expectedSpec = JSON.stringify({
      logic: 'and',
      rules: [{ field: 'status', operator: 'equals', value: 'Active' }],
    });
    await waitFor(() => {
      expect(apiFetchMock).toHaveBeenCalledWith(
        `/horizon-v2/analytics?entity=initiatives&filters=${encodeURIComponent(expectedSpec)}`
      );
    });
  });

  it('creates a custom chart with sum metric via CustomChartModal and deletes custom charts (SEP-041 T020)', async () => {
    apiFetchMock.mockImplementation(async (url, opts) => {
      if (url === '/custom-charts' && opts?.method === 'POST') {
        return { id: 77, ...JSON.parse(opts.body) };
      }
      if (url === '/custom-charts/77' && opts?.method === 'DELETE') {
        return null;
      }
      return {
        scope: 'horizon-v2',
        entity: 'initiatives',
        title: 'Gráficos da Tabela — Iniciativas',
        loaded: true,
        total_rows: 50,
        kpis: [],
        charts: [
          {
            id: 'custom_chart_77',
            title: 'Bolsistas por Campus',
            subtitle: 'Soma de students • 1 filtro(s) (AND)',
            kind: 'bar',
            is_custom: true,
            custom_id: 77,
            items: [
              {
                label: 'Serra',
                count: 14,
                secondary_value: 14.0,
                formatted_value: '14 (soma: students)',
                percentage: 100.0,
              },
            ],
          },
        ],
        available_entities: [
          {
            entity: 'initiatives',
            label: 'initiatives',
            columns: ['campus_name', 'status', 'students'],
          },
        ],
      };
    });

    render(<AnalyticsDashboard endpoint="/horizon-v2/analytics?entity=initiatives" />);
    await waitFor(() => {
      expect(screen.getByTestId('open-custom-chart-modal-btn')).toBeDefined();
    });

    // Open modal and create custom chart with sum metric
    fireEvent.click(screen.getByTestId('open-custom-chart-modal-btn'));
    expect(screen.getByTestId('custom-chart-modal')).toBeDefined();

    fireEvent.change(screen.getByTestId('custom-chart-title-input'), {
      target: { value: 'Bolsistas por Campus' },
    });
    fireEvent.change(screen.getByTestId('custom-chart-groupby-select'), {
      target: { value: 'campus_name' },
    });
    fireEvent.change(screen.getByTestId('custom-chart-metric-select'), {
      target: { value: 'sum' },
    });
    fireEvent.change(screen.getByTestId('custom-chart-metric-field-select'), {
      target: { value: 'students' },
    });
    fireEvent.click(screen.getByTestId('save-custom-chart-btn'));

    await waitFor(() => {
      expect(apiFetchMock).toHaveBeenCalledWith(
        '/custom-charts',
        expect.objectContaining({ method: 'POST' })
      );
    });

    // Delete custom chart
    const deleteBtn = screen.getByTestId('delete-custom-chart-77');
    fireEvent.click(deleteBtn);
    await waitFor(() => {
      expect(apiFetchMock).toHaveBeenCalledWith('/custom-charts/77', {
        method: 'DELETE',
      });
    });
  });
});

