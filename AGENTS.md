# AGENTS.md

## Project Overview

Keyway is a cross-platform quick launcher designed for a keyboard-first productivity workflow, inspired by applications such as Raycast, uTools, and ZLaunch.

Key characteristics:

- Keyboard-first: core interactions should support keyboard-driven activation, search, selection, and execution.
- Cross-platform: desktop behavior should remain consistent across Windows, macOS, Linux, and other supported environments.
- Extensible: commands, actions, and extension modules compose the product surface and make new capabilities easy to add.
- Sandboxed: extensions and external capabilities should run behind isolation boundaries to reduce impact on the main process and user environment.

## Architecture

[ARCHITECTURE.md](./ARCHITECTURE.md) is the source of truth for repository structure, crate responsibilities, and crate ownership decisions. Before adding files or moving behavior, use it to decide where the code belongs.

When a package under `crates` is added, removed, renamed, or has its responsibility changed, update [ARCHITECTURE.md](./ARCHITECTURE.md) in the same change.

## Technical Perfectionism Rule

When unreasonable design, redundant abstractions, duplicated logic, dead pathways, or unnecessary code is discovered in the scope being touched or reviewed, do not preserve it for convenience. Treat root-cause cleanup as required work: remove the redundancy, simplify the design, and refactor boldly enough to leave the code technically clean instead of applying a narrow patch over a known flaw.

## Development Workflow

1. Identify the requested capability.
2. Confirm the owning crate already has the right abstraction; only add a new crate when no existing crate fits.
3. Update code in the smallest appropriate package.
4. Update documentation when structure, public API, ownership, or rules change.
5. Run validation commands before reporting completion.
