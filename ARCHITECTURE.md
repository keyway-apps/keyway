# ARCHITECTURE.md

## Overview

Keyway is a Rust and GPUI desktop application workspace. The application package, `keyway_app`, provides the `keyway` executable and currently initializes logging, internationalization, and the GPUI application runtime.

The `crates` directory is split into foundation capabilities and feature modules. Foundation crates provide shared boundaries such as paths, logging, collection types, core domain types, and extensions. Feature crates implement concrete quick-launch capabilities such as calculator, clipboard, file search, screenshots, settings, and store functionality. Package names use the `keyway_` prefix so workspace-owned crates remain recognizable in source code, dependency trees, and diagnostics.

## Documentation Rule

When a package under `crates` is added, removed, renamed, or has its responsibility changed, this document must be updated in the same change. If the package should participate in builds, also check the root `Cargo.toml` entries for `workspace.members` and `workspace.dependencies`.

## Root Package

| Path | Package | Responsibility |
| --- | --- | --- |
| `crates/keyway/src/main.rs` | `keyway_app` | Single-file process entry point for the `keyway` executable; initializes GPUI and enabled modules. |

## Workspace Crates

### Foundation Crates

| Path | Package | Responsibility |
| --- | --- | --- |
| `crates/assets` | `keyway_assets` | Embeds and exposes application assets to GPUI. |
| `crates/cli` | `keyway_cli` | Auxiliary CLI package and future command-line tooling boundary. |
| `crates/collections` | `keyway_collections` | Provides common collection aliases and re-exports, currently defaulting to `rustc_hash` `FxHashMap` and `FxHashSet`. |
| `crates/config` | `keyway_config` | Owns configuration structures, loading, validation, and persistence boundaries. |
| `crates/core` | `keyway_core` | Owns the shared Command model, command-provider contract, and command registry. See [`docs/command-design.md`](./docs/command-design.md). |
| `crates/db` | `keyway_db` | Persistence and database access layer boundary. |
| `crates/extension` | `keyway_extension` | Extension system boundary, including extension manifests, loading, runtime contracts, and sandbox integration. |
| `crates/i18n` | `keyway_i18n` | Internationalization entry point; wraps `rust-i18n` locale initialization and re-exports translation macros. |
| `crates/ipc` | `keyway_ipc` | Owns the typed tarpc IPC protocol, local endpoint policy integration, daemon client helpers, and server transport adapters for daemon requests. |
| `crates/net` | `keyway_net` | Keyway-owned local IPC transport shim; exposes UnixListener/UnixStream types backed by Unix domain sockets on Unix platforms and `uds_windows` on Windows. |
| `crates/onboarding` | `keyway_onboarding` | First-run setup, permission guidance, and initialization experience. |
| `crates/paths` | `keyway_paths` | Resolves cross-platform application, configuration, data, cache, log, and extension directories. |
| `crates/ktracing` | `keyway_ktracing` | Logging and tracing initialization with environment filter support, stderr output, and file logging. |
| `crates/theme` | `keyway_theme` | Application theme definitions and theme integration boundary. |
| `crates/ui` | `keyway_ui` | Shared application UI components and presentation primitives. |
| `crates/util` | `keyway_util` | Shared lightweight utility macros and helpers that do not belong to a more specific foundation crate. |
| `crates/window` | `keyway_window` | Window lifecycle, window abstraction, OS window primitives, and UI shell boundary. |
| `crates/workspace` | `keyway_workspace` | Main application workspace state, command palette/search, layout, and navigation boundary; it consumes but does not own the Command model. |

### Feature Crates

| Path | Package | Responsibility |
| --- | --- | --- |
| `crates/agent` | `keyway_agent` | Agent/AI-related commands and workflows. |
| `crates/calculator` | `keyway_calculator` | Calculator commands, expression evaluation, and result presentation. |
| `crates/clipboard` | `keyway_clipboard` | Clipboard history, search, and paste-related capabilities. |
| `crates/devtools` | `keyway_devtools` | Developer-focused quick tools. |
| `crates/emoji` | `keyway_emoji` | Emoji search, selection, and input capabilities. |
| `crates/file-search` | `keyway_file_search` | File indexing, search, and open-related capabilities. |
| `crates/notes` | `keyway_notes` | Notes, quick capture, and related commands. |
| `crates/screenshot` | `keyway_screenshot` | Screenshot capture, processing, and follow-up actions. |
| `crates/settings` | `keyway_settings` | Settings UI, preference editing, and configuration entry points. |
| `crates/store` | `keyway_store` | Extension/feature store, installation sources, and package management entry point. |
