# Quickstart & Validation Guide: `038-github-branch-upload-src`

**Feature**: `038-github-branch-upload-src`  
**Date**: 2026-11-05  

## 1. Pré-requisitos

- Ambiente Linux com Rust (`cargo`) e Node.js (`npm`).
- Branch ativa: `038-github-branch-upload-src`.

## 2. Comandos de Validação Automatizada

```bash
# 1. Testes unitários do front-end (Vitest)
cd frontend && npm test

# 2. Verificação de tipos e build estático Astro
cd frontend && npm run build

# 3. Testes unitários e de integração HTTP (wiremock) em Rust
cd ../src-tauri && cargo test
```

## 3. Cenários Verificados pela Suíte

1. **Criação de nova branch no upload (`US1`, `FR-004`..`FR-007`)**:
   - Nos 4 projetos (`horizon`, `src`, `egressos`, `egressos_dados`), enviar com `create_new_branch: true` lê o `HEAD` da branch base (`main` ou outra escolhida), publica o(s) blob(s), árvore e commit e cria `refs/heads/<nova-branch>` via `POST /repos/{owner}/{repo}/git/refs`, preservando a branch base inalterada.
2. **Listagem e seleção de branches além da `main` (`US2`, `FR-006`)**:
   - `GET /repos/{owner}/{repo}/branches` lista todas as branches existentes do repositório, permitindo selecionar qualquer branch como base ou como destino direto no modo "Enviar na branch selecionada".
3. **Liberação do upload do projeto SRC (`US3`, `FR-001`..`FR-003`)**:
   - `check_upload_allowed("src")` retorna `Ok(())`, `validate_path("src", ...)` exige `.json`, `/src` renderiza `SyncPanel` com `allowUpload={true}` e o upload publica o JSON consolidado do SRC idêntico ao export manual local.
