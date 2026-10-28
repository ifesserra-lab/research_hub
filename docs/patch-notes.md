# Notas de Versão (Patch Notes) 📜

Histórico completo de versões, melhorias, correções de bugs e novas funcionalidades do **Research Hub**.

---

## v1.5.0 (Atual — Espaço de Dados Egressos, CRUD nas 21 Tabelas & Sincronização GitHub de Pasta)
*Branch de Origem: `035-egressos-data-workspace`*

### Projeto Egressos (SEP-035)
- **Terceiro Projeto no Seletor (`/projects`)**: Adição do cartão **Egressos** ao lado de **Horizon** e **SRC**, com rotas dedicadas `/egressos` (Controle de Dados e Sincronização GitHub) e `/egressos/tabelas` (Curadoria das 21 Tabelas).
- **Esquema Relacional Isolado**: Migração `003_egressos_tables.sql` (`egressos_files`, `egressos_rows`, `egressos_meta`) completamente desacoplada das tabelas do Horizon e do SRC.
- **Importação e Exportação Fiel dos 21 Arquivos JSON (`dados/*.json`)**:
  - Validação atômica pré-importação exigindo a presença e integridade sintática dos 21 arquivos `.json` do projeto Egressos.
  - Criação automática de snapshot de segurança (`egressos-pre-import-*.db`) antes de substituir os dados locais.
  - Exportação *lossless* para pasta local mantendo todos os 21 arquivos `.json` separados, preservando tipos numéricos, booleanos, nulos, objetos escalares (`overview`) e metadados de cabeçalho (*envelope*).
- **21 Abas Dedicadas + Sub-Abas e CRUD Completo**:
  - 21 abas primárias mapeadas 1:1 para cada arquivo `.json` e sub-abas automáticas para arquivos com múltiplos arrays internos.
  - Busca textual instantânea, ordenação por qualquer coluna, paginação com salto direto, criação, edição e exclusão de registros, e editor dedicado para os metadados de cabeçalho (*envelope*) de cada arquivo.
  - **Propagação Opcional Entre Tabelas (`propagate`)**: Opção explícita no formulário para o curador escolher entre aplicar a edição/exclusão apenas na tabela atual ou propagar automaticamente os campos compartilhados de **Egresso** (`Alumni`) ou **Empresa** (`Company`) nas tabelas relacionadas.
- **Sincronização GitHub em Nível de Diretório (`dados/`)**:
  - Download remoto dos 21 arquivos `.json` diretamente do diretório `dados/` no repositório/branch configurado no GitHub.
  - Upload atômico dos 21 arquivos `.json` em um **único commit Git** via Git Data API (criação dos 21 blobs + tree + commit + atualização da ref).
- **Compatibilidade Linux / Ambientes Snap**:
  - Sanitização automática de variáveis de ambiente GSettings/GTK contaminadas pelo VS Code Snap (`sanitize_snap_gtk_env`) no núcleo Rust e no `Makefile`.

---

## v1.4.0 – v1.4.1 (Sincronização GitHub & Pacotes Aninhados)
*Branch de Origem: `034-github-sync`*

### Sincronização com GitHub (SEP-034)
- **Download Remoto de Exports**: Importação direta de pacotes canônicos a partir de URLs do GitHub (repositórios públicos ou privados com token configurado). Suporte a URLs diretas de arquivos (`raw.githubusercontent.com`) e assets de releases do GitHub.
- **Upload Atômico de Exports**: Envio do pacote canônico do Horizon diretamente para branches do GitHub via Git Data API (criação direta de commits atômicos sem necessidade de clonar o repositório).
- **Gerenciamento Seguro de Token**: Armazenamento local de Personal Access Token (PAT) do GitHub em arquivo com permissão POSIX restrita `0600`. Modal interativo com teste de validade e escopos, mascaramento do token e remoção completa com zero vestígios.
- **Proteção de Privacidade (LGPD / PII)**: Bloqueio estrito de envio de dados do projeto SRC para repositórios remotos, garantindo que CPFs e e-mails de participantes nunca sejam expostos na nuvem.
- **Limite Operacional e Segurança**: Suporte a transferências de pacotes de até 100 MB com validação prévia de tamanho e criação de snapshots automáticos antes de qualquer substituição de base.
- **Auditoria de Não-Vazamento**: 100% dos fluxos de erro auditados — tokens e dados pessoais nunca são emitidos em logs, mensagens de erro ou telemetria.

### Pacotes Canônicos Aninhados
- Suporte para ingestão transparente de arquivos ZIP aninhados ou pastas com subdiretórios no pacote canônico.

---

## v1.3.3 (Curadoria SRC & Seletor de Projetos)
*Branch de Origem: `033-src-data-tab`*

### Seletor Multi-Projeto
- Nova página inicial `/projects` pós-autenticação para alternar entre os projetos **Horizon** (15 tabelas canônicas) e **SRC** (Extensão e Ensino).
- Isolamento estrito de bases de dados, tabelas, formulários e snapshots de segurança entre os dois projetos.

### Domínio de Extensão e Ensino (SRC)
- Tabela dedicada `src_acoes` e migração relacional `002_src_tables.sql`.
- Importação e exportação de arquivo JSON consolidado do SRC (`src_consolidado.json`), preservando chaves e ordem original dos objetos.
- Editor visual especializado de **Participações** aninhadas às ações (Público-alvo e Equipe de execução).
- Vínculo textual com programas guarda-chuva e proteção contra exclusão de programas com ações dependentes.

---

## v1.3.0 (Novo Formato Canônico Somente JSON & Aba Campus)
*Branch de Origem: `032-canonical-export-json`*

### Ingestão de Pacote Somente JSON (SEP-032)
- Adaptação completa da importação e exportação para o novo padrão do DataLake (arquivos `{tabela}_canonical.json` na raiz do ZIP, dispensando arquivos Parquet).
- Compatibilidade retroativa mantida para pacotes legados contendo Parquet.
- Módulo de inferência e fidelidade de tipos JSON para round-trip perfeito (preserva tipos originais booleanos, inteiros, decimais, nulls e listas de relacionamentos).

### Correção da Aba Campus
- Remoção do campo aninhado fantasma "Campus (Vínculos)" herdado do bug upstream do DataLake.

---

## v1.2.0 (Estabilidade do Fluxo de Curadoria & Desduplicação)
*Branches de Origem: `030-search-by-id` e `031-bugfixes-curation-workflow`*

### Busca por Identificador (SEP-030)
- Campo de busca com suporte a busca direta por ID numérico em todas as entidades do Horizon.

### Correções Críticas do Fluxo de Curadoria (SEP-031)
- **B-01**: Correção do formulário de edição para evitar estado obsoleto de registros anteriores ao alternar alvos de visualização.
- **B-02**: Reposicionamento automático da paginação para a última página válida após exclusão de registros da última página.
- **B-03**: Limpeza automática da seleção de fusão após deleções, buscas ou troca de página, impedindo fusões de IDs inexistentes.
- **B-04**: Fusão aprimorada com suporte a campo de identificação por "título" (artigos, produções e premiações) além de "nome".
- **B-04b**: União transacional de listas de relacionamentos durante a fusão de registros, evitando perda de vínculos.
- **B-05**: Proteção contra condições de corrida (*race conditions*) nas requisições assíncronas de busca com debounce.
- **B-06**: Normalização canônica do nome de usuário em minúsculas na sessão.
- **B-07**: Verificação antecipada de autenticação da sessão antes da abertura de diálogos de arquivos no import/export.
- **B-08**: Validação atômica de pacote ZIP na importação, rejeitando pacotes sem nenhuma tabela canônica sem apagar dados existentes.

---

## v1.1.0 – v1.1.3 (Autenticação, Registro de Usuários & Acessibilidade)
*Branch de Origem: `029-user-registration`*

### Cadastro de Usuários (SEP-029)
- Nova tela `/register` para criação de contas de administrador.
- Hashing seguro de senhas com algoritmo `bcrypt` no núcleo Rust.

### Acessibilidade e Qualidade
- Melhorias de acessibilidade (`htmlFor`, associação de labels com inputs no formulário de autenticação).
- Suíte de testes unitários com ambiente JSDOM e cleanup de formulários.

---

## v1.0.9 (Paginação com Salto Direto)
*Branch de Origem: `028-table-page-nav`*

### Navegação de Páginas
- Campo numérico interativo de navegação direta para qualquer página da listagem, com validação e clamping de limites.

---

## v1.0.0 – v1.0.7 (Migração Rust + Tauri 2.0 & Decomissionamento Python)
*Branches de Origem: `rust-tauri-migration` (SEP-015 a SEP-026)*

### Reescrita Arquitetural Completa
- Decomissionamento definitivo do antigo backend em Python FastAPI e banco PostgreSQL.
- Criação da arquitetura desktop Tauri 2.0 em Rust com banco de dados SQLite embarcado de arquivo único.
- Eliminação de processos em segundo plano, portas de rede e serviços dependentes de servidor.
- Frontend em Astro + React + Tailwind compilado estaticamente e embutido no executável final.
- Pipelines de CI/CD para geração de binários nativos multiplataforma (Linux `.deb`, `.rpm`, `.AppImage`; Windows `.exe`, `.msi`; macOS `.dmg`, `.app`).
- Suporte a compilação conteinerizada com Docker base Ubuntu 24.04.
