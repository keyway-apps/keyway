# ARCHITECTURE.md

## Overview

Keyway is a Rust and GPUI desktop application workspace. The application package, `keyway`, provides the `keyway` executable and currently initializes logging, internationalization, the module system, and the GPUI application runtime.

The `crates` directory is split into foundation capabilities and feature modules. Foundation crates provide shared boundaries such as paths, logging, collection types, module and command domain types, and extensions. Feature crates implement concrete quick-launch capabilities such as calculator, clipboard, file search, screenshots, settings, and store functionality. Package names match their directories and do not use a product-specific prefix.

## Documentation Rule

When a package under `crates` is added, removed, renamed, or has its responsibility changed, this document must be updated in the same change. If the package should participate in builds, also check the root `Cargo.toml` entries for `workspace.members` and `workspace.dependencies`.

## Root Package

| Path | Package | Responsibility |
| --- | --- | --- |
| `crates/keyway/src/main.rs` | `keyway` | Single-file process entry point for the `keyway` executable; initializes GPUI and enabled modules. |

## Workspace Crates

### Foundation Crates

| Path | Package | Responsibility |
| --- | --- | --- |
| `crates/assets` | `assets` | Embeds and exposes application assets to GPUI. |
| `crates/cli` | `cli` | Auxiliary CLI package and future command-line tooling boundary. |
| `crates/collections` | `collections` | Provides common collection aliases and re-exports, currently defaulting to `rustc_hash` `FxHashMap` and `FxHashSet`. |
| `crates/config` | `config` | Owns configuration structures, loading, validation, and persistence boundaries. |
| `crates/db` | `db` | Persistence and database access layer boundary. |
| `crates/extension` | `extension` | Extension system boundary, including extension manifests, loading, runtime contracts, and sandbox integration. |
| `crates/i18n` | `i18n` | Internationalization entry point; wraps `rust-i18n` locale initialization and re-exports translation macros. |
| `crates/ipc` | `ipc` | Owns the typed tarpc IPC protocol, local endpoint policy integration, daemon client helpers, and server transport adapters for daemon requests. |
| `crates/ktracing` | `ktracing` | Logging and tracing initialization with environment filter support, stderr output, and file logging. |
| `crates/module` | `module` | Owns module lifecycle and grouping; `ModuleContext` owns a scoped `CommandRegistry` for command models and actions. See [`docs/module-design.md`](./docs/module-design.md). |
| `crates/net` | `net` | Keyway-owned local IPC transport shim; exposes UnixListener/UnixStream types backed by Unix domain sockets on Unix platforms and `uds_windows` on Windows. |
| `crates/onboarding` | `onboarding` | First-run setup, permission guidance, and initialization experience. |
| `crates/paths` | `paths` | Resolves cross-platform application, configuration, data, cache, log, and extension directories. |
| `crates/theme` | `theme` | Application theme definitions and theme integration boundary. |
| `crates/ui` | `ui` | Shared application UI components and presentation primitives. |
| `crates/util` | `util` | Shared lightweight utility macros and helpers that do not belong to a more specific foundation crate. |
| `crates/window` | `window` | Window lifecycle, window abstraction, OS window primitives, and UI shell boundary. |
| `crates/workspace` | `workspace` | Main application workspace state, command palette/search, layout, and navigation boundary; it consumes but does not own the Command model. |

### Feature Crates

| Path | Package | Responsibility |
| --- | --- | --- |
| `crates/agent` | `agent` | Agent/AI-related commands and workflows. |
| `crates/calculator` | `calculator` | Calculator commands, expression evaluation, and result presentation. |
| `crates/clipboard` | `clipboard` | Clipboard history, search, and paste-related capabilities. |
| `crates/devtools` | `devtools` | Developer-focused quick tools. |
| `crates/emoji` | `emoji` | Emoji search, selection, and input capabilities. |
| `crates/file-search` | `file_search` | File indexing, search, and open-related capabilities. |
| `crates/notes` | `notes` | Notes, quick capture, and related commands. |
| `crates/screenshot` | `screenshot` | Screenshot capture, processing, and follow-up actions. |
| `crates/settings` | `settings` | Settings UI, preference editing, and configuration entry points. |
| `crates/store` | `store` | Extension/feature store, installation sources, and package management entry point. |
