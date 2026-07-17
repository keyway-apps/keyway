# ARCHITECTURE.md

## Overview

Keyway is a Rust and GPUI desktop application workspace. The root package, `keyway`, provides the application entry point and currently initializes logging, internationalization, and the GPUI application runtime.

The `crates` directory is split into foundation capabilities and feature modules. Foundation crates provide shared boundaries such as paths, logging, collection types, commands, actions, and extensions. Feature crates implement concrete quick-launch capabilities such as calculator, clipboard, file search, screenshots, settings, and store functionality.

## Documentation Rule

When a package under `crates` is added, removed, renamed, or has its responsibility changed, this document must be updated in the same change. If the package should participate in builds, also check the root `Cargo.toml` entries for `workspace.members` and `workspace.dependencies`.

## Root Package

| Path | Package | Responsibility |
| --- | --- | --- |
| `crates/keyway/src/main.rs` | `keyway` | Single-file process entry point; handles local bootstrap commands, probes and starts the internal daemon when needed, waits for IPC readiness, forwards the original command through local IPC, and initializes GPUI plus enabled modules only in the spawned daemon role. |

## Workspace Crates

### Foundation Crates

| Path | Package | Responsibility |
| --- | --- | --- |
| `crates/action` | `action` | Defines typed action identity, GPUI-compatible action construction, action metadata/schema, and action dispatch boundaries reused by commands, hotkeys, extensions, and feature modules. |
| `crates/bin` | `bin` | Auxiliary binary entry package; currently a placeholder entry point for future CLI or development tools. |
| `crates/collections` | `collections` | Provides common collection aliases and re-exports, currently defaulting to `rustc_hash` `FxHashMap` and `FxHashSet`. |
| `crates/command` | `command` | Owns the serializable Command domain model, search-panel visibility/input invariants, argument and output schemas, CLI exposure metadata, schema versioning, and the validated Command Catalog/registry. See [`docs/command-design.md`](./docs/command-design.md). |
| `crates/config` | `config` | Owns configuration structures, loading, validation, and persistence boundaries. |
| `crates/core` | `core` | Holds shared core domain types, daemon lifecycle logic, keymap/global-hotkey runtime services, Command execution bindings, and dispatch orchestration; consumes the validated Command Catalog but does not own the Command model. |
| `crates/db` | `db` | Persistence and database access layer boundary. |
| `crates/extension` | `extension` | Extension system boundary, including extension manifests, loading, runtime contracts, and sandbox integration. |
| `crates/hotkey` | `hotkey` | Transitional placeholder; default bindings reference stable Command IDs, while keymap/global-hotkey runtime services belong to `core`. |
| `crates/i18n` | `i18n` | Internationalization entry point; wraps `rust-i18n` locale initialization and re-exports translation macros. |
| `crates/ipc` | `ipc` | Owns the typed tarpc IPC protocol, local endpoint policy integration, daemon client helpers, and server transport adapters for daemon requests. |
| `crates/net` | `net` | Keyway-owned local IPC transport shim; exposes UnixListener/UnixStream types backed by Unix domain sockets on Unix platforms and `uds_windows` on Windows. This crate intentionally stays small instead of depending on Zed's internal `net` crate. |
| `crates/onboarding` | `onboarding` | First-run setup, permission guidance, and initialization experience. |
| `crates/paths` | `paths` | Resolves cross-platform application, configuration, data, cache, log, and extension directories. |
| `crates/ktracing` | `ktracing` | Logging and tracing initialization with environment filter support, stderr output, and file logging. |
| `crates/util` | `util` | Shared lightweight utility macros and helpers, including internal module inclusion helpers, that do not belong to a more specific foundation crate. |
| `crates/window` | `window` | Window lifecycle, window abstraction, OS window primitives, and UI shell boundary. |
| `crates/workspace` | `workspace` | Main application workspace state, command palette/search over search-visible Command descriptors, workspace-scoped action handlers, view host placement, layout, and navigation boundary; it consumes but does not own the Command model. |

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
