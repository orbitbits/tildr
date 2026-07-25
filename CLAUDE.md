# Tildr

## Objetivo

Tildr é um gerenciador declarativo do diretório HOME escrito em Rust.

## Linguagem

- Rust stable
- edition = 2024

## Regras

- Nunca usar unwrap().
- Preferir anyhow::Result.
- Evitar código duplicado.
- Priorizar módulos pequenos.
- Escrever documentação pública.
- Seguir Clippy.
- Nunca dê commit sem autorização.

## Documentação

A documentação é arquivos em Markdown (.md), está em:

- docs/site/*.md
- docs/man/*.md

Sempre que alterar o código .rs, analisar se é necessário atualizar a documentação.

## Testes

Sempre executar:

cargo fmt
cargo clippy
cargo test

antes de finalizar alterações.

## Estilo

- Preferir iteradores.
- Evitar alocação desnecessária.
