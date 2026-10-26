# Contract: Pasta `dados/` do Projeto Egressos (`egressos-dados-json.md`)

**Date**: 2026-10-22 | **Feature**: [spec.md](../spec.md)

## 1. Formato de Entrada e Saída

O intercâmbio de dados do projeto **Egressos** ocorre sempre como um conjunto de **21 arquivos JSON independentes** em um diretório (localmente em `/home/thiagodeps/egressos/dados` ou outra pasta escolhida pelo curador; remotamente na pasta configurada do repositório GitHub, por padrão `dados/`).

### Lista Canônica Obrigatória (21 arquivos)

1. `analise.json`
2. `cargos_ao_longo_do_tempo.json`
3. `codigofonte_2026.json`
4. `codigofonte_historico.json`
5. `consolidado.json`
6. `egressos_perfil.json`
7. `empresas_aliases.json`
8. `empresas_linkedin_data.json`
9. `empresas_linkedin_urls.json`
10. `empresas_porte.json`
11. `fapes_fomento.json`
12. `ibge_series.json`
13. `impacto.json`
14. `mapa_mundi.json`
15. `panorama.json`
16. `recorte_cursos.json`
17. `renda_por_senioridade.json`
18. `salario_minimo.json`
19. `so_benchmarks.json`
20. `trajetoria.json`
21. `vitrine.json`

## 2. Garantias de Validação na Importação (`FR-005`)

- **C1 (Presença dos 21 arquivos)**: Se o diretório informado não existir ou faltar qualquer um dos 21 arquivos acima, a importação falha com `AppError::Validation` listando os arquivos ausentes.
- **C2 (Arquivos extras ignorados)**: Arquivos ocultos ou não pertencentes à lista canônica (ex.: `.DS_Store`, `README.md`) presentes na pasta são ignorados silenciosamente.
- **C3 (Validação estrutural por arquivo)**: Cada arquivo deve ser um objeto JSON válido (`serde_json::Value::Object`) e conter sua seção primária com o tipo esperado (array ou objeto/mapa conforme `data-model.md`). Se qualquer arquivo contiver sintaxe JSON inválida ou tipo raiz incompatível, a importação falha informando o nome do arquivo defeituoso.
- **C4 (Atomicidade e Snapshot)**: A substituição das tabelas `egressos_files` e `egressos_rows` ocorre em uma única transação SQLite. Caso já existissem dados do Egressos carregados, um snapshot `snapshots/egressos-pre-import-<ts>.db` é criado antes da escrita (`FR-006`).

## 3. Garantias de Fidelidade na Exportação (`SC-003`, `FR-010`)

- **C5 (Round-trip sem perdas)**: Importar `/home/thiagodeps/egressos/dados` e exportar para um diretório de saída sem realizar edições produz 21 arquivos `.json` cujo valor JSON parseado (`serde_json::Value`) é **100% idêntico** ao arquivo de entrada correspondente (mesmas chaves, mesmos elementos de lista na mesma ordem, mesmos tipos numéricos/booleanos/nulos e acentuação UTF-8 preservada).
- **C6 (Formatação legível)**: Os arquivos JSON exportados localmente e enviados ao GitHub são serializados com indentação de 2 espaços (`serde_json::to_string_pretty`) e quebra de linha final (`\n`), mantendo diffs limpos no Git.
