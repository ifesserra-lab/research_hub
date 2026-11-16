/**
 * @vitest-environment jsdom
 */
import React, { useState } from 'react';
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, cleanup } from '@testing-library/react';
import FilterBuilderBar from '../../src/components/FilterBuilderBar.jsx';

afterEach(() => {
  cleanup();
});

function StatefulHarness({ initialSpec = { logic: 'and', rules: [] }, onCreateChart = null }) {
  const [spec, setSpec] = useState(initialSpec);
  return (
    <FilterBuilderBar
      columns={[
        { key: 'status', label: 'Status' },
        { key: 'campus_name', label: 'Campus' },
        { key: 'email', label: 'E-mail' },
      ]}
      filterSpec={spec}
      onChange={setSpec}
      onCreateChart={onCreateChart}
    />
  );
}

describe('FilterBuilderBar (SEP-041 T008)', () => {
  it('adds and removes column filter rules and displays active filter chips', () => {
    render(<StatefulHarness />);

    const colSelect = screen.getByTestId('filter-column-select');
    const opSelect = screen.getByTestId('filter-operator-select');
    const valInput = screen.getByTestId('filter-value-input');
    const addBtn = screen.getByTestId('add-filter-rule-btn');

    fireEvent.change(colSelect, { target: { value: 'status' } });
    fireEvent.change(opSelect, { target: { value: 'equals' } });
    fireEvent.change(valInput, { target: { value: 'Active' } });
    fireEvent.click(addBtn);

    expect(screen.getByTestId('active-filters-count').textContent).toContain('1 filtro ativo');
    expect(screen.getByTestId('filter-chip-0').textContent).toContain('Status');
    expect(screen.getByTestId('filter-chip-0').textContent).toContain('Igual a');
    expect(screen.getByTestId('filter-chip-0').textContent).toContain('Active');

    // Add a unary filter (is_empty) that does not require a text input
    fireEvent.change(colSelect, { target: { value: 'email' } });
    fireEvent.change(opSelect, { target: { value: 'is_empty' } });
    expect(screen.queryByTestId('filter-value-input')).toBeNull();
    fireEvent.click(addBtn);

    expect(screen.getByTestId('active-filters-count').textContent).toContain('2 filtros ativos');
    expect(screen.getByTestId('filter-chip-1').textContent).toContain('Está vazio');

    // Remove first chip
    fireEvent.click(screen.getByTestId('remove-filter-chip-0'));
    expect(screen.getByTestId('active-filters-count').textContent).toContain('1 filtro ativo');

    // Clear all
    fireEvent.click(screen.getByTestId('clear-filters-btn'));
    expect(screen.queryByTestId('active-filters-count')).toBeNull();
  });

  it('toggles between E (AND) and OU (OR) combination logic and invokes onCreateChart', () => {
    const onCreateChart = vi.fn();
    render(
      <StatefulHarness
        initialSpec={{
          logic: 'and',
          rules: [{ field: 'status', operator: 'equals', value: 'Active' }],
        }}
        onCreateChart={onCreateChart}
      />
    );

    const toggleBtn = screen.getByTestId('filter-logic-toggle');
    expect(toggleBtn.textContent).toContain('E (AND)');

    fireEvent.click(toggleBtn);
    expect(toggleBtn.textContent).toContain('OU (OR)');

    const createChartBtn = screen.getByTestId('create-chart-from-filters-btn');
    fireEvent.click(createChartBtn);
    expect(onCreateChart).toHaveBeenCalledWith({
      logic: 'or',
      rules: [{ field: 'status', operator: 'equals', value: 'Active' }],
    });
  });
});
