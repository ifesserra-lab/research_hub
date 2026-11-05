# Quickstart & Verification Guide: Tabelas, CRUD e Sincronização de `egressos-dados`

**Feature**: `037-egressos-dados-tables`  
**Date**: 2026-11-05  

## 1. Pré-requisitos

- Rust toolchain (`cargo`) e Node.js (`npm`) instalados.
- Repositório local de referência disponível em `/home/thiagodeps/egressos-dados` (contendo `alunos.json` na raiz e 38 arquivos `.json` em `data/`).
- Branch ativa: `037-egressos-dados-tables`.

## 2. Comandos de Verificação Automatizada (TDD)

```bash
# 1. Executar os testes unitários do front-end (Vitest)
cd /home/thiagodeps/research_hub/frontend
npm test

# 2. Gerar o build estático do Astro para o resolver do Tauri (25 rotas do menu)
npm run build

# 3. Executar todos os testes unitários e de integração em Rust
cd /home/thiagodeps/research_hub/src-tauri
cargo test
```

## 3. Cenários de Validação Ponta a Ponta

1. **Guarda da Organização `ifesserra-lab` (`FR-001`)**:
   - Sem autenticar na aba `/egressos-dados`, qualquer chamada aos comandos `egressos_dados_*` retorna `kind: "org_unauthorized"`.
2. **Importação e Round-Trip dos 39 Datasets (`FR-003`, `FR-004`, `FR-008`, `FR-009`, `SC-002`)**:
   - Na aba `/egressos-dados/controle`, importar `/home/thiagodeps/egressos-dados`.
   - Confirmar que os 39 arquivos JSON (`alunos.json` + 38 em `data/*.json`) são carregados nas tabelas `egressos_dados_files` e `egressos_dados_rows`.
   - Exportar para um diretório temporário e verificar que `alunos.json` na raiz e os 38 arquivos em `data/*.json` são 100% equivalentes semanticamente aos arquivos originais (incluindo os 7 arquivos `RootArray` e `genero_map.json`).
3. **Navegação, Filtros e CRUD nas 39 Tabelas (`FR-005`, `FR-006`, `FR-007`)**:
   - Na aba `/egressos-dados`, filtrar por `PII / Coorte & Coleta (12)`, `Internos (6)` e `Publicáveis (21)`.
   - Selecionar `alunos` (`alunos.json`), verificar que a coluna `"id"` exibe os IDs textuais (`"barbosa"`, `"gary"`, `"helen"`, etc.), alternar para as sub-seções `cursos` e `convencoes`, criar, editar (com propagação opcional) e excluir um registro.
4. **Sincronização GitHub com Token em Memória (`FR-010`)**:
   - Em `/egressos-dados/controle`, executar Download e Upload contra `ifesserra-lab/egressos-dados` reutilizando automaticamente o token da sessão `EgressosDadosAuth` e verificando que os 39 blobs (`alunos.json` e `data/*.json`) compõem um único commit atômico.
