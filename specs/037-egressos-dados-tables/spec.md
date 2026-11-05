# Feature Specification: Tabelas, CRUD e Sincronização do Repositório `egressos-dados` (`alunos.json` e `data/*.json`)

**Feature Branch**: `037-egressos-dados-tables`  
**Created**: 2026-11-05  
**Status**: Draft  
**Input**: User description: "agora vamos implementar a tabela do engressos-dados , eu adicionei o repositorio no pc em /home/thiagodeps/egressos-dados , tem a pasta de data com varias pastas e json e o de alunos , entao analise o que precisa ser adicionado e faz as tabelas"

## Contexto

Na feature anterior (`036-egressos-github-org-auth`), foi implementado o espaço restrito **Egressos — Dados** (`/egressos-dados`) protegido por autenticação via Token do GitHub exclusivo para membros ativos da organização **`ifesserra-lab`** (`https://github.com/orgs/ifesserra-lab/dashboard`).

Agora, esta feature conecta o espaço desbloqueado de **Egressos — Dados** ao conjunto completo de dados do repositório privado **`ifesserra-lab/egressos-dados`** (disponível localmente em `/home/thiagodeps/egressos-dados`).

A análise do repositório `/home/thiagodeps/egressos-dados` (e de seu catálogo oficial em `egressos_core.dados.CATALOGO`) identifica **39 datasets JSON canônicos** distribuídos em dois pontos da árvore do repositório:

1. **Na raiz do repositório (`alunos.json`)**:
   - `alunos` (`alunos.json`) — Cadastro nominal completo da coorte (50 alunos com `id`, `curso`, `nome`, `cargo_atual`, `empresa_atual`, `local_atual`, `area_atual`, `inicio_carreira_dev`, `ainda_em_tech`, `experiencias`, `educacao`, `url` etc.), acompanhado das seções `cursos` e `convencoes`.
2. **Na pasta `data/` (`data/*.json` — 38 arquivos JSON)**:
   - **11 datasets PII (dados pessoais, nominais e cruzamentos internos)**:
     - `perfis_linkedin` (`data/perfis_linkedin.json` — 176 perfis de formandos localizados no LinkedIn)
     - `perfis_conferidos` (`data/perfis_conferidos.json` — 56 conferências humanas de homônimos/formação)
     - `formandos_a_buscar` (`data/formandos_a_buscar.json` — fila de 280 formandos do SIGA BSI/ECA com matrícula e semestres, derivada de `data/formandos/*.xlsx`)
     - `_novos_reais` (`data/_novos_reais.json` — 6 registros da coorte nova com nomes reais)
     - `genero_map` (`data/genero_map.json` — mapa `id -> gênero` dos 50 egressos)
     - `horizon_pesquisa` (`data/horizon_pesquisa.json` — cruzamento nominal com projetos e bolsas de pesquisa do Horizon)
     - `lattes_orientacoes` (`data/lattes_orientacoes.json` — cruzamento nominal com orientações do Lattes)
     - `outros_labs` (`data/outros_labs.json` — participação em outros laboratórios além do LEDS)
     - `src_extensao` (`data/src_extensao.json` — cruzamento com ações e bolsas de extensão do SRC/FAPES)
     - `por_ano` (`data/por_ano.json` — histórico individual de empregadores e faixas salariais por ano)
     - `consolidado_raw` (`data/consolidado_raw.json` — série histórica consolidada antes da anonimização)
   - **6 datasets Internos (`interno` — insumos e estado do pipeline)**:
     - `_empresas_lista` (`data/_empresas_lista.json` — lista de 157 empresas para classificação)
     - `_novos_anon5` (`data/_novos_anon5.json` — rascunho anonimizado de 8 egressos)
     - `_novos_anon6` (`data/_novos_anon6.json` — rascunho anonimizado de 5 egressos)
     - `agg_pv` (`data/agg_pv.json` — agregado de valor presente por anos de experiência)
     - `fontes_estado` (`data/fontes_estado.json` — estado e hashes das fontes externas monitoradas pelo vigia)
     - `mapa_mundi_detalhe` (`data/mapa_mundi_detalhe.json` — malha geográfica detalhada em 50m com 779 caminhos)
   - **21 datasets Publicáveis (`publicavel` — saídas agregadas/anonimizadas e vitrines)**:
     - `egressos_perfil`, `vitrine`, `panorama`, `analise`, `consolidado`, `impacto`, `trajetoria`, `recorte_cursos`, `fapes_fomento`, `renda_por_senioridade`, `cargos_ao_longo_do_tempo`, `codigofonte_2026`, `codigofonte_historico`, `so_benchmarks`, `salario_minimo`, `ibge_series`, `empresas_porte`, `empresas_aliases`, `empresas_linkedin_data`, `empresas_linkedin_urls` e `mapa_mundi`.

## Clarifications

### Session 2026-11-05

- Q: Como devemos organizar as 39 tabelas e os controles de dados (importação/exportação local, snapshots e sincronização GitHub) dentro do espaço desbloqueado de `egressos-dados`? (FR-005) → A: **Opção A** — Duas páginas no menu lateral (`/egressos-dados` para Tabelas e `/egressos-dados/controle` para Controle de Dados), seguindo o mesmo padrão visual e de arquitetura do projeto `Egressos`, com ambas protegidas pelo login especial da organização `ifesserra-lab`.
- Q: Além dos 39 datasets JSON canônicos do repositório (`alunos.json` na raiz e os 38 arquivos `.json` em `data/`), como devemos tratar os arquivos brutos auxiliares presentes nas subpastas `alunos/*.txt` (4 textos brutos do LinkedIn) e `data/formandos/*.xlsx` (8 planilhas do SIGA)? (FR-003) → A: **Opção A** — Operar exclusivamente sobre os 39 datasets JSON canônicos (`alunos.json` + 38 arquivos `.json` em `data/`), que já consolidam os registros estruturados de alunos e formandos, preservando intactos no disco os arquivos brutos `alunos/*.txt`, `data/formandos/*.xlsx` e `data/*.md`.
- Q: No catálogo do repositório `egressos-dados`, cada dataset possui um papel (`entrada`, curado manualmente como `alunos.json` e `perfis_conferidos.json`, ou `saida`, gerado pelo pipeline); o curador deve poder realizar CRUD completo em todas as 39 tabelas JSON? (FR-006) → A: **Opção A** — CRUD completo em todas as 39 tabelas JSON (`entrada` e `saida`), exibindo na interface os badges informativos de papel (`entrada` / `saida`) e de privacidade (`pii` / `interno` / `publicavel`).
- Q: Na sincronização com o GitHub em `/egressos-dados/controle` (download e upload dos 39 arquivos JSON), como devemos tratar a credencial e o repositório remoto? (FR-010) → A: **Opção A** — Reutilizar automaticamente o token em memória do login especial de `egressos-dados` (`EgressosDadosAuth`) e pré-configurar `ifesserra-lab/egressos-dados` (branch `main`) como destino padrão, sem gravar o token em disco.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Navegar e executar CRUD completo nas 39 tabelas de `egressos-dados` (`alunos.json` + `data/*.json`) (Priority: P1) 🎯 MVP

Após desbloquear a aba **Egressos — Dados** (`/egressos-dados`) com um Token do GitHub válido de membro ativo da organização `ifesserra-lab`, o curador visualiza o espaço de tabelas completo do repositório `egressos-dados`. O curador pode navegar entre as **39 abas de arquivos JSON** (começando em destaque por `alunos` / `alunos.json` e pelas tabelas de coleta e cruzamento PII, seguidas pelos datasets internos e publicáveis), filtrar as abas por categoria (`Todos`, `PII / Alunos & Coleta`, `Internos`, `Publicáveis`), alternar entre as sub-seções internas de cada arquivo (ex.: `alunos`, `cursos` e `convencoes` dentro de `alunos.json`), pesquisar e ordenar registros por coluna, e executar **CRUD completo (criar, visualizar, editar e excluir)** sobre os registros e metadados de cada arquivo.

**Why this priority**: É o objetivo central solicitado pelo usuário — transformar o painel desbloqueado de `egressos-dados` em uma bancada completa de visualização e edição das tabelas de `alunos.json` e da pasta `data/`.

**Independent Test**: Autenticar-se na aba `/egressos-dados` como membro de `ifesserra-lab`, importar `/home/thiagodeps/egressos-dados`, abrir a tabela `alunos` (`alunos.json`) e tabelas de `data/` (como `perfis_linkedin`, `formandos_a_buscar`, `_novos_reais`, `genero_map` e `vitrine`), buscar um aluno, criar, editar e excluir um registro e verificar que as mudanças persistem na base dedicada de `egressos-dados`.

**Acceptance Scenarios**:

1. **Given** o curador autenticado na organização `ifesserra-lab` e com a base `egressos-dados` carregada, **When** ele acessa `/egressos-dados`, **Then** o sistema exibe as 39 abas correspondentes a `alunos.json` e aos 38 arquivos JSON de `data/`, identificando claramente a classificação de cada dataset (`pii`, `interno` ou `publicavel`) e seu caminho relativo (`alunos.json` ou `data/<arquivo>.json`).
2. **Given** a aba `alunos` (`alunos.json`) selecionada, **When** o curador visualiza a seção principal `alunos`, **Then** a tabela lista os 50 alunos com colunas legíveis (`id`, `nome`, `curso`, `cargo_atual`, `empresa_atual`, `local_atual`, `area_atual`, `inicio_carreira_dev`, `ainda_em_tech`), permite inspecionar/editar as listas aninhadas (`experiencias`, `educacao`, etc.) e alternar para as sub-seções `cursos` e `convencoes`.
3. **Given** qualquer uma das 39 tabelas de `egressos-dados` (seja baseada em objeto com seções como `alunos.json` e `perfis_linkedin.json`, array raiz de objetos como `_novos_reais.json` e `por_ano.json`, array raiz de textos como `_empresas_lista.json` ou mapa raiz por chave como `genero_map.json` e `empresas_porte.json`), **When** o curador cria, edita ou exclui um registro, **Then** a alteração é salva imediatamente na base local exclusiva de `egressos-dados` sem afetar a base do projeto público `Egressos`.
4. **Given** um registro de aluno ou empresa compartilhado entre múltiplas tabelas de `egressos-dados`, **When** o curador edita ou exclui o registro marcando a opção de propagar alterações, **Then** o sistema atualiza os campos equivalentes nas demais tabelas relacionadas dentro de `egressos-dados`.

---

### User Story 2 - Importar e exportar localmente o repositório `egressos-dados` (`alunos.json` + pasta `data/`) (Priority: P2)

Dentro do espaço desbloqueado de **Egressos — Dados**, o curador importa os dados diretamente a partir da pasta raiz do repositório local (ex.: `/home/thiagodeps/egressos-dados`). O sistema valida a presença e integridade de `alunos.json` na raiz e dos 38 arquivos `.json` dentro da subpasta `data/`, cria um snapshot de segurança exclusivo (`egressos-dados-*`) caso já exista base carregada e substitui atomicamente os dados de `egressos-dados`. Ao exportar localmente para uma pasta raiz do repositório, o sistema regrava `alunos.json` na raiz e todos os 38 arquivos `.json` dentro de `data/` preservando fielmente o formato raiz de cada arquivo (objeto com seções, array raiz de objetos/escalares ou mapa por chave) e seus metadados.

**Why this priority**: Permite carregar imediatamente os dados reais já existentes em `/home/thiagodeps/egressos-dados` na máquina do curador e devolver as alterações curadas para o diretório local sem perda de estrutura.

**Independent Test**: No painel `/egressos-dados`, importar `/home/thiagodeps/egressos-dados`, verificar a carga dos 39 arquivos JSON e suas contagens de registros, exportar para um diretório temporário e confirmar que tanto `alunos.json` na raiz quanto os 38 arquivos em `data/*.json` são recriados com equivalência semântica e estrutural completa (*round-trip*).

**Acceptance Scenarios**:

1. **Given** o diretório local `/home/thiagodeps/egressos-dados` contendo `alunos.json` na raiz e os 38 arquivos `.json` em `data/`, **When** o curador aciona a importação local em `/egressos-dados`, **Then** o sistema valida todos os 39 arquivos JSON, cria snapshot da base anterior (se houver) e carrega integralmente as 39 tabelas exibindo o total de arquivos e registros importados.
2. **Given** uma base `egressos-dados` carregada no aplicativo, **When** o curador aciona a exportação local informando a pasta raiz do repositório, **Then** o sistema grava `alunos.json` na raiz da pasta e os 38 arquivos `.json` dentro da subpasta `data/`, mantendo a estrutura raiz exata de cada arquivo (incluindo arrays na raiz como `_novos_reais.json` e `_empresas_lista.json` e objetos/mapas).
3. **Given** uma pasta informada na importação em que `alunos.json` ou qualquer um dos 38 arquivos obrigatórios de `data/` esteja ausente ou contenha JSON inválido, **When** o curador tenta importar, **Then** a operação é abortada atomicamente com mensagem clara indicando qual arquivo falhou, mantendo a base anterior de `egressos-dados` intacta.

---

### User Story 3 - Baixar e enviar os 39 arquivos JSON de `egressos-dados` para o repositório GitHub `ifesserra-lab/egressos-dados` (Priority: P3)

Na área desbloqueada de **Egressos — Dados**, o curador pode sincronizar (download e upload) os 39 arquivos JSON (`alunos.json` na raiz do repositório + os 38 arquivos em `data/*.json`) diretamente com o repositório privado da organização (`ifesserra-lab/egressos-dados`), reutilizando a credencial da sessão autenticada da aba (ou o token configurado) sem precisar redigitar o token a cada operação. No download, o sistema baixa os 39 arquivos da branch configurada, valida o conjunto completo, cria snapshot de segurança e atualiza a base local de `egressos-dados`. No upload, o sistema gera os 39 arquivos JSON a partir do estado curado atual e publica todas as alterações em um único commit atômico na branch configurada.

**Why this priority**: Completa o ciclo de trabalho remoto da equipe `ifesserra-lab`, permitindo baixar os JSONs diretamente do repositório privado no GitHub e publicar as edições das tabelas sem sair do aplicativo.

**Independent Test**: Com a aba `/egressos-dados` desbloqueada, executar o download do repositório `ifesserra-lab/egressos-dados` (branch `main`), verificar que os 39 arquivos (`alunos.json` + `data/*.json`) são carregados, editar um registro e executar o upload verificando que um único commit atômico atualiza `alunos.json` e `data/*.json` na branch alvo.

**Acceptance Scenarios**:

1. **Given** o curador autenticado na aba `/egressos-dados`, **When** ele aciona o download a partir do repositório GitHub (por padrão `ifesserra-lab/egressos-dados`, branch `main`), **Then** o sistema baixa `alunos.json` e os 38 arquivos de `data/`, valida todos os 39 arquivos, cria snapshot se necessário e substitui a base local de `egressos-dados`.
2. **Given** uma base `egressos-dados` carregada e curada na aba desbloqueada, **When** o curador confirma o envio (upload) para a branch configurada do repositório `ifesserra-lab/egressos-dados`, **Then** o sistema publica os 39 arquivos (`alunos.json` na raiz e os 38 arquivos em `data/*.json`) em um único commit atômico e exibe o identificador do commit gerado.
3. **Given** qualquer tentativa de chamar comandos de leitura, escrita, importação, exportação ou sincronização de `egressos-dados` sem que a sessão especial da organização `ifesserra-lab` esteja ativa, **When** a operação é invocada, **Then** o núcleo recusa imediatamente a execução com erro de autorização da organização (`org_unauthorized`).

---

### Edge Cases

- **Arquivos JSON cujo topo é uma lista (`RootArray`) e não um objeto**: Diferentemente dos 21 arquivos públicos do projeto `Egressos` (onde todos os JSONs têm um objeto `{}` na raiz), o repositório `egressos-dados` possui **7 arquivos cuja raiz é diretamente uma lista `[]`**: `data/_novos_reais.json`, `data/_novos_anon5.json`, `data/_novos_anon6.json`, `data/agg_pv.json`, `data/consolidado_raw.json`, `data/por_ano.json` (listas de objetos) e `data/_empresas_lista.json` (lista de strings). O sistema deve importar esses arquivos como tabela principal e reconstruir exatamente um array JSON `[]` na raiz ao exportar/enviar para o GitHub.
- **Arquivo na raiz (`alunos.json`) vs arquivos na subpasta (`data/*.json`)**: O dataset `alunos` reside na raiz do repositório (`alunos.json`), enquanto os outros 38 residem em `data/<nome>.json`. Caso o usuário selecione na importação local tanto a pasta raiz `/home/thiagodeps/egressos-dados` quanto a subpasta `/home/thiagodeps/egressos-dados/data`, o sistema deve localizar automaticamente `alunos.json` (na raiz ou no diretório pai de `data/`) e os 38 arquivos em `data/`.
- **Arquivos não-JSON presentes no repositório (`alunos/*.txt`, `data/formandos/*.xlsx`, `data/*.md`)**: Os arquivos auxiliares de texto bruto do LinkedIn em `alunos/*.txt`, as planilhas do SIGA em `data/formandos/*.xlsx` e os documentos `.md` em `data/` não são alterados nem apagados durante a importação ou exportação local dos 39 datasets JSON.
- **Seções vazias em arquivos JSON (`data/src_extensao.json`)**: Quando um arquivo possui listas ou dicionários vazios (como `encontrados: []`, `bolsistas_extensao: []`, `funcoes: {}` em `src_extensao.json`), o sistema preserva o formato original (lista vazia `[]` vs objeto vazio `{}`) durante a importação e exportação sem perdas.
- **Isolamento total entre os projetos `Egressos` (`/egressos`) e `Egressos — Dados` (`/egressos-dados`)**: Editar, importar ou sincronizar os 39 arquivos em `Egressos — Dados` jamais altera ou sobrescreve as 21 tabelas da base do projeto público `Egressos` (`/egressos`), e vice-versa.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O sistema MUST manter todas as operações de listagem, leitura, CRUD, importação, exportação e sincronização de `egressos-dados` estritamente protegidas pela sessão em memória da organização GitHub **`ifesserra-lab`** (`require_egressos_dados_session`), recusando qualquer chamada não autorizada com erro `org_unauthorized`.
- **FR-002**: O sistema MUST armazenar os dados de `egressos-dados` em tabelas SQLite próprias e isoladas (`egressos_dados_files` e `egressos_dados_rows`), garantindo que nenhuma operação em `egressos-dados` modifique ou misture registros com os projetos `Horizon`, `SRC` ou `Egressos` (público).
- **FR-003**: O sistema MUST disponibilizar na aba desbloqueada `/egressos-dados` o catálogo completo dos **39 datasets JSON** do repositório `ifesserra-lab/egressos-dados` (`alunos.json` na raiz + 38 arquivos em `data/*.json`), expondo para cada dataset sua chave (`file_key`), caminho relativo no repositório (`relative_path`, ex.: `alunos.json` ou `data/perfis_linkedin.json`), título amigável, papel (`entrada` ou `saida`) e classificação de privacidade (`pii`, `interno` ou `publicavel`).
- **FR-004**: O sistema MUST suportar na importação, no CRUD e na exportação sem perdas (*round-trip*) os **4 formatos estruturais** presentes nos 39 arquivos JSON de `egressos-dados`:
  1. `object_sections`: objeto JSON na raiz contendo uma ou mais seções (listas de objetos, listas de escalares ou mapas) + metadados escalares de cabeçalho (ex.: `alunos.json`, `data/perfis_linkedin.json`, `data/formandos_a_buscar.json`, `data/vitrine.json`).
  2. `root_array`: array JSON `[]` diretamente na raiz do arquivo, contendo objetos (ex.: `data/_novos_reais.json`, `data/_novos_anon5.json`, `data/_novos_anon6.json`, `data/agg_pv.json`, `data/consolidado_raw.json`, `data/por_ano.json`) ou valores escalares (ex.: `data/_empresas_lista.json`).
  3. `root_map`: objeto JSON na raiz mapeando chaves de entidade para objetos ou escalares (ex.: `data/genero_map.json`, `data/empresas_porte.json`, `data/empresas_aliases.json`, `data/empresas_linkedin_data.json`, `data/empresas_linkedin_urls.json`).
  4. `nested_map`: objeto JSON na raiz contendo metadados e sub-mapas indexados por chave (ex.: `data/codigofonte_historico.json`, `data/ibge_series.json`, `data/recorte_cursos.json`, `data/fontes_estado.json`).
- **FR-005**: O sistema MUST organizar o espaço desbloqueado de `egressos-dados` em duas páginas no menu lateral (`/egressos-dados` para **Tabelas** e `/egressos-dados/controle` para **Controle de Dados**, ambas protegidas pelo login especial de `ifesserra-lab`), permitindo em `/egressos-dados` filtrar as 39 abas por classificação do catálogo (`Todos`, `PII / Alunos & Coleta`, `Internos`, `Publicáveis`), buscar pelo nome/título do arquivo e alternar entre sub-seções internas quando um arquivo possuir mais de uma seção (ex.: `alunos`, `cursos` e `convencoes` em `alunos.json`).
- **FR-006**: O sistema MUST oferecer em todas as 39 tabelas de `egressos-dados` busca textual, ordenação por coluna e **CRUD completo** (criar registro, visualizar/editar campos escalares e estruturas JSON aninhadas como `experiencias` em `alunos.json`, excluir registro com confirmação e editar metadados de envelope do arquivo quando aplicável).
- **FR-007**: O sistema MUST permitir ao curador, ao editar ou excluir registros que compartilham vínculo de entidade em `egressos-dados` (grupo `Alumni` por `id`/`rotulo`/`label` e grupo `Company` por nome da empresa), optar entre aplicar a mudança apenas ao arquivo atual ou propagar os campos equivalentes para os demais arquivos relacionados de `egressos-dados`.
- **FR-008**: O sistema MUST permitir importar localmente o conjunto de 39 arquivos JSON a partir do diretório do repositório `egressos-dados` (aceitando tanto o caminho da raiz `/home/thiagodeps/egressos-dados` quanto a subpasta `/home/thiagodeps/egressos-dados/data` e resolvendo `alunos.json` na raiz e os 38 arquivos em `data/`), validando todos os 39 arquivos antes de alterar a base e criando um snapshot exclusivo (`egressos-dados-*`) caso já exista base carregada.
- **FR-009**: O sistema MUST permitir exportar localmente os 39 arquivos JSON para o diretório raiz escolhido pelo curador, gravando `alunos.json` na raiz e os 38 arquivos `.json` dentro da subpasta `data/` sem remover nem alterar outros arquivos existentes no diretório (como `alunos/*.txt` ou `data/formandos/*.xlsx`).
- **FR-010**: O sistema MUST permitir baixar (download) e enviar (upload em commit único atômico) os 39 arquivos JSON (`alunos.json` na raiz do repositório + 38 arquivos em `data/*.json`) contra o repositório GitHub configurado para `egressos-dados` (por padrão `ifesserra-lab/egressos-dados`, branch `main`), utilizando automaticamente o token validado na sessão de `egressos-dados`.

### Key Entities *(include if feature involves data)*

- **Catálogo de Datasets de `egressos-dados` (`EGRESSOS_DADOS_FILES` — 39 arquivos)**: Conjunto canônico dos 39 arquivos JSON do repositório `ifesserra-lab/egressos-dados`, contendo `file_key`, `file_name`, `relative_path` (`alunos.json` ou `data/<arquivo>.json`), `title`, `kind` (`object_sections`, `root_array`, `root_map`, `nested_map`), `publication` (`pii`, `interno`, `publicavel`), `role` (`entrada`, `saida`), `primary_section`, `sections`, `columns` e `entity_group`.
- **Arquivo Carregado de `egressos-dados` (`egressos_dados_files`)**: Registro persistido no SQLite representando cada um dos 39 arquivos JSON importados na base de `egressos-dados`, armazenando seu envelope de metadados (`meta_json`), ordem das chaves de topo (`top_keys_json`), contagem de linhas e carimbo de atualização.
- **Linha de Tabela de `egressos-dados` (`egressos_dados_rows`)**: Registro individual editável via CRUD pertencente a um arquivo (`file_key`) e seção (`section_key`) de `egressos-dados`, com chave identificadora (`row_key`), posição de ordenação (`row_index`), texto de busca (`search_text`) e payload completo (`raw_json`).
- **Snapshot e Configuração de Sincronização de `egressos-dados`**: Conjunto isolado de backups automáticos (`egressos-dados-*`) e parâmetros de repositório remoto (por padrão `repo: "ifesserra-lab/egressos-dados"`, `branch: "main"`) para sincronização atômica dos 39 arquivos via Git Data API.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A importação local do repositório `/home/thiagodeps/egressos-dados` carrega 100% dos **39 arquivos JSON** (`alunos.json` + 38 arquivos de `data/`) em menos de 10 segundos, disponibilizando todas as 39 tabelas e suas sub-seções para consulta e edição imediata.
- **SC-002**: Exportar localmente uma base `egressos-dados` recém-importada (sem edições) para um diretório produz `alunos.json` na raiz e os 38 arquivos em `data/*.json` com **100% de equivalência semântica e estrutural** aos arquivos originais de `/home/thiagodeps/egressos-dados` (incluindo preservação de arrays na raiz, mapas por chave, listas/objetos vazios e metadados).
- **SC-003**: O curador consegue filtrar as abas por classificação (`PII`, `Interno`, `Publicável`), abrir `alunos.json` ou qualquer arquivo de `data/` e realizar o ciclo completo de CRUD (criar, visualizar, editar e excluir um registro) em menos de 2 minutos, vendo a alteração refletida tanto na tabela quanto no arquivo JSON exportado.
- **SC-004**: 100% das operações de leitura, CRUD, importação, exportação e sincronização de `egressos-dados` permanecem inacessíveis sem o login especial ativo da organização `ifesserra-lab`, e 0% das operações realizadas em `egressos-dados` alteram dados dos projetos `Horizon`, `SRC` ou `Egressos` (público).
- **SC-005**: O download e o upload dos 39 arquivos JSON (`alunos.json` + `data/*.json`) contra o repositório GitHub concluem em um único fluxo atômico, mantendo a base local intacta em 100% dos cenários de falha de rede ou permissão.

## Assumptions

- O conjunto de tabelas de `egressos-dados` corresponde exatamente aos **39 datasets JSON** declarados no catálogo oficial do repositório `/home/thiagodeps/egressos-dados` (`egressos_core.dados.CATALOGO`): `alunos.json` na raiz e os 38 arquivos `.json` dentro de `data/`.
- Os arquivos auxiliares que não são JSON (`alunos/*.txt`, `data/formandos/*.xlsx` e `data/*.md`) são insumos brutos/documentação de coleta já consolidados nos respectivos datasets JSON (`alunos.json` e `data/formandos_a_buscar.json`) e não precisam ser transformados em tabelas independentes nem sobrescritos na exportação.
- Na sincronização com o GitHub em `/egressos-dados`, o token já autenticado e mantido em memória durante o login especial da aba (`EgressosDadosAuth`) é reutilizado automaticamente para realizar download e upload contra `ifesserra-lab/egressos-dados` (com possibilidade de o curador ajustar repositório/branch caso utilize um fork de teste da organização).
- O trabalho desta feature é desenvolvido na nova branch dedicada `037-egressos-dados-tables`.
