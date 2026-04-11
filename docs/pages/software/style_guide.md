---
title: Style Guidelines
layout: default
parent: Software
nav_order: 1
---

# Style Guidelines

This document defines the conventions and standards for all software
in this project. Consistency across code, documentation, branches,
and issues makes the codebase easier to navigate and maintain.

---

## Table of Contents

- [Rust Code Style](#rust-code-style)
- [Documentation Style](#documentation-style)
- [Naming Conventions](#naming-conventions)
- [Branch Conventions](#branch-conventions)
- [Issue Conventions](#issue-conventions)

---

## Rust Code Style

Follow the standard Rust style enforced by `rustfmt`. Run it before every commit:

```bash
cargo fmt
```

### General Rules

- Use `snake_case` for variables, functions, and modules.
- Use `PascalCase` for types, structs, enums, and traits.
- Use `SCREAMING_SNAKE_CASE` for constants and statics.
- Prefer explicit types in public APIs; let type inference handle local variables.
- Avoid `unwrap()` and `expect()` in production code paths. Use proper error propagation with `?` or match on `Result`/`Option`.
- Keep functions short and single-purpose. If a function doesn't fit on one screen, consider splitting it.
- Prefer `match` over long `if-else` chains for exhaustive pattern matching.

---

## Documentation Style

All documentation is written in Markdown and lives in the `docs/` directory, processed by Jekyll with the Just the Docs theme.

### File Frontmatter

Every `.md` file must include:

```yaml
---
title: Page Title
layout: default
parent: Parent Section   # omit if top-level
nav_order: N
---
```

### Code Blocks

Always specify the language for syntax highlighting:

````markdown
```rust
// embedded Rust example
```

```bash
# shell commands
```

```c
// C/C++ or linker examples
```
````

### Headings

- `#` — page title only (one per file).
- `##` — major sections.
- `###` — subsections.
- Do not skip heading levels.
- Use sentence case: *"Error handling"*, not *"Error Handling"*.

### Callouts (Just the Docs)

Use these consistently to flag important information:

```markdown
{: .warning }
This operation resets all...

{: .note }
This only applies to...

{: .important }
Changing this register requires disabling...
```

---

## Naming Conventions

### Modules

Avoid module names like `utils`, `helpers`, or `misc` — these become dumping grounds.

### Constants and Statics

```rust
const MAX_PACKET_SIZE: usize = 256;
static DEVICE_ID: u32 = 0xDEAD_BEEF;
```

Use `_` separators in large numeric literals for readability: `0x0000_FFFF`, `1_000_000`.

### Booleans

Prefix boolean variables and functions with `is_`, `has_`, or `was_`:

```rust
let is_ready: bool = ...;
fn has_pending_data(&self) -> bool { ... }
```

### Acronyms in Names

Treat acronyms as words. Use `Uart`, `Spi`, `Dma`, not `UART`, `SPI`, `DMA` in type names. This keeps `rustfmt` and autocomplete happy:

```rust
struct UartDriver { ... }   // ✓
struct UARTDriver { ... }   // ✗
```

---

## Branch Conventions

### Format

```
<type>/<short-description>
```

Use lowercase and hyphens. No spaces, no underscores, no uppercase.

### Types

| Type | When to use |
|---|---|
| `feat/` | New functionality |
| `fix/` | Bug fix |
| `refactor/` | Code restructuring without behaviour change |
| `docs/` | Documentation only |
| `test/` | Adding or updating tests |
| `chore/` | Tooling, CI, dependency updates |
| `hw/` | Hardware bring-up, board support, linker changes |

### Examples

```
feat/uart-dma-rx
fix/spi-clock-polarity
docs/style-guide
hw/stm32g4-bsp-init
chore/update-embassy
```

---

## Issue Conventions

### Title Format

```
[Type] Short imperative description
```

Use the imperative mood as if completing the sentence *"This issue will…"*

### Types

| Label | Description |
|---|---|
| `[bug]` | Something is broken or behaves incorrectly |
| `[feat]` | A new feature or capability |
| `[refactor]` | Internal restructuring |
| `[docs]` | Documentation gap or error |
| `[hw]` | Hardware-related issue (pin conflicts, timing, etc.) |
| `[chore]` | Maintenance, dependency bump, CI |
| `[question]` | Needs discussion or decision before work begins |

### Examples

```
[bug] UART RX drops bytes at 921600 baud
[feat] Add watchdog timer support
[hw] SPI2 MISO conflicts with LED GPIO on rev B board
[docs] Document flash memory layout
```
