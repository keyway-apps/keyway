# ARCHITECTURE.md

## Overview

Keyway is a Rust and GPUI desktop application workspace. The root package, `keyway`, provides the application entry point and currently initializes logging, internationalization, and the GPUI application runtime.

The `crates` directory is split into foundation capabilities and feature modules. Foundation crates provide shared boundaries such as paths, logging, collection types, commands, actions, and extensions. Feature crates implement concrete quick-launch capabilities such as calculator, clipboard, file search, screenshots, settings, and store functionality.

## Documentation Rule

When a package under `crates` is added, removed, renamed, or has its responsibility changed, this document must be updated in the same change. If the package should participate in builds, also check the root `Cargo.toml` entries for `workspace.members` and `workspace.dependencies`.

## Root Package

| Path | Package | Responsibility |
| --- | --- | --- |
| `src/main.rs` | `keyway` | Desktop application entry point; initializes `ktracing` and `i18n`, starts the GPUI application, and directly calls enabled first-party module `init(cx)` functions in daemon mode. |

## Workspace Crates

### Foundation Crates

| Path | Package | Responsibility |
| --- | --- | --- |
| `crates/action` | `action` | Defines typed action identity, GPUI-compatible action construction, action metadata/schema, and action dispatch boundaries reused by commands, hotkeys, extensions, and feature modules. |
| `crates/bin` | `bin` | Auxiliary binary entry package; currently a placeholder entry point for future CLI or development tools. |
| `crates/collections` | `collections` | Provides common collection aliases and re-exports, currently defaulting to `rustc_hash` `FxHashMap` and `FxHashSet`. |
| `crates/command` | `command` | Transitional placeholder; target design folds command metadata/search into `workspace::contributions` unless a real implementation-only boundary is justified. |
| `crates/config` | `config` | Owns configuration structures, loading, validation, and persistence boundaries. |
| `crates/core` | `core` | Holds shared core domain types, daemon lifecycle logic, keymap/global-hotkey runtime services, and action dispatch orchestration; consumes the workspace `Contribution` entity through `ContributionHandle` but does not own its model. |
| `crates/db` | `db` | Persistence and database access layer boundary. |
| `crates/extension` | `extension` | Extension system boundary, including extension manifests, loading, runtime contracts, and sandbox integration. |
| `crates/hotkey` | `hotkey` | Transitional placeholder; target design folds default hotkey metadata into `workspace::contributions` and keymap/global-hotkey runtime services into `core`. |
| `crates/i18n` | `i18n` | Internationalization entry point; wraps `rust-i18n` locale initialization and re-exports translation macros. |
| `crates/ipc` | `ipc` | Owns the typed tarpc IPC protocol, local endpoint policy integration, daemon client helpers, and server transport adapters for daemon requests. |
| `crates/net` | `net` | Keyway-owned local IPC transport shim; exposes UnixListener/UnixStream types backed by Unix domain sockets on Unix platforms and `uds_windows` on Windows. This crate intentionally stays small instead of depending on Zed's internal `net` crate. |
| `crates/onboarding` | `onboarding` | First-run setup, permission guidance, and initialization experience. |
| `crates/paths` | `paths` | Resolves cross-platform application, configuration, data, cache, log, and extension directories. |
| `crates/ktracing` | `ktracing` | Logging and tracing initialization with environment filter support, stderr output, and file logging. |
| `crates/window` | `window` | Window lifecycle, window abstraction, OS window primitives, and UI shell boundary. |
| `crates/workspace` | `workspace` | Main application workspace state, singleton `workspace::contributions::Contribution` entity plus `ContributionHandle`, command palette/search, workspace-scoped action handlers, view host placement, layout, and navigation boundary. |

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
