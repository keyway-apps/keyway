# ARCHITECTURE.md

## Overview

Keyway is a Rust and GPUI desktop application workspace. The root package, `keyway`, provides the application entry point and currently initializes logging, internationalization, and the GPUI application runtime.

The `crates` directory is split into foundation capabilities and feature modules. Foundation crates provide shared boundaries such as paths, logging, collection types, commands, actions, and extensions. Feature crates implement concrete quick-launch capabilities such as calculator, clipboard, file search, screenshots, settings, and store functionality.

## Documentation Rule

When a package under `crates` is added, removed, renamed, or has its responsibility changed, this document must be updated in the same change. If the package should participate in builds, also check the root `Cargo.toml` entries for `workspace.members` and `workspace.dependencies`.

## Root Package

| Path | Package | Responsibility |
| --- | --- | --- |
| `src/main.rs` | `keyway` | Desktop application entry point; initializes `kw_tracing` and `kw_i18n`, then starts the GPUI application. |

## Workspace Crates

### Foundation Crates

| Path | Package | Responsibility |
| --- | --- | --- |
| `crates/action` | `kw_action` | Defines the action model and execution boundary reused by commands, extensions, and feature modules. |
| `crates/bin` | `kw_bin` | Auxiliary binary entry package; currently a placeholder entry point for future CLI or development tools. |
| `crates/collections` | `kw_collections` | Provides common collection aliases and re-exports, currently defaulting to `rustc_hash` `FxHashMap` and `FxHashSet`. |
| `crates/command` | `kw_command` | Defines command metadata, search, matching, and execution abstractions. |
| `crates/config` | `kw_config` | Owns configuration structures, loading, validation, and persistence boundaries. |
| `crates/core` | `kw_core` | Holds shared core domain types and orchestration logic across modules; currently a skeleton crate. |
| `crates/db` | `kw_db` | Persistence and database access layer boundary. |
| `crates/extension` | `kw_extension` | Extension system boundary, including extension manifests, loading, runtime contracts, and sandbox integration. |
| `crates/hotkey` | `kw_hotkey` | Hotkey registration, parsing, and dispatch boundary for keyboard-first interaction. |
| `crates/i18n` | `kw_i18n` | Internationalization entry point; wraps `rust-i18n` locale initialization and re-exports translation macros. |
| `crates/onboarding` | `kw_onboarding` | First-run setup, permission guidance, and initialization experience. |
| `crates/paths` | `kw_paths` | Resolves cross-platform application, configuration, data, cache, log, and extension directories. |
| `crates/tracing` | `kw_tracing` | Logging and tracing initialization with environment filter support, stderr output, and file logging. |
| `crates/window` | `kw_window` | Window lifecycle, window abstraction, and UI shell boundary. |
| `crates/workspace` | `kw_workspace` | Main application workspace state, layout, and navigation boundary. |

### Feature Crates

| Path | Package | Responsibility |
| --- | --- | --- |
| `crates/agent` | `kw_agent` | Agent/AI-related commands and workflows. |
| `crates/calculator` | `kw_calculator` | Calculator commands, expression evaluation, and result presentation. |
| `crates/clipboard` | `kw_clipboard` | Clipboard history, search, and paste-related capabilities. |
| `crates/devtools` | `kw_devtools` | Developer-focused quick tools. |
| `crates/emoji` | `kw_emoji` | Emoji search, selection, and input capabilities. |
| `crates/file-search` | `kw_file_search` | File indexing, search, and open-related capabilities. |
| `crates/notes` | `kw_notes` | Notes, quick capture, and related commands. |
| `crates/screenshot` | `kw_screenshot` | Screenshot capture, processing, and follow-up actions. |
| `crates/settings` | `kw_settings` | Settings UI, preference editing, and configuration entry points. |
| `crates/store` | `kw_store` | Extension/feature store, installation sources, and package management entry point. |
