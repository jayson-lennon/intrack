# intrack

A lightweight, git-native issue tracker and task list. All data, history, and conversations are stored as commit events directly in the repo. No external services, databases, or vendors.

## Features

- **Event Sourced Storage**: Issues, tasks, comments, and state changes are immutable events appended to files in your repo (e.g., `events.jsonl`).
- **Full Git History**: Audit trails, threaded conversations, and diffs are baked into Git. `git blame`, `git log --follow`, or `git show` reveals all.
- **Vim-Style TUI**: Terminal UI with intuitive `hjkl` navigation, blazing-fast issue creation, editing, and commenting.
- **`$EDITOR`**: All text-based input uses your `$EDITOR` to minimize friction.
- **Lightweight**: Plain text events, no bloat.

## Installation

**Prerequisites**: Git, Rust, and a terminal.

```bash
git clone https://github.com/jayson-lennon/intrack.git
cd intrack
cargo install --path .
```

### Arch Package

```bash
git clone https://github.com/jayson-lennon/intrack.git
cd intrack
makepkg -si
```


## Quick Start

Run in any Git repo:

```bash
intrack
```

## Keybindings (Modal TUI)

| **Action**              | **Keys**                                                                 |
|-------------------------|--------------------------------------------------------------------------|
| Help                    | `?`                                                                      |
| Navigate                | `j/k`                                                                    |
| New Issue               | `n` (new) or `a` (add)                                                   |
| Comment                 | `enter` on issue → `n` (new) or `a` (add)                                |
| Sorting                 | `shift+h/l` change sort column, `shift+j/k` sort descending/ascending, `c` change displayed columns |
| Toggle open/closed      | `s` (status)                                                             |
| Search                  | `/`                                                                      |
| Quit                    | `q`                                                                      |


## License

GPLv3. See [LICENSE](LICENSE).

