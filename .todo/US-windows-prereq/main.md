# US-windows-prereq: Prerequisites for Windows Support

## Theme

Preparatory changes that must land before the `US-windows` story begins. These are independent improvements that happen to be prerequisites for clean Windows support.

## Sequencing

Tasks are independent and can be done in any order. All must be complete before starting `US-windows`.

```
01-terminals-settings-tab     ─┐
02-xdg-config-home            ─┤  All before US-windows
03-terminal-detection          ─┘
```

## Relationship to US-windows

- Task 01 (Terminals tab) unblocks `US-windows/07-settings-platform-compat` — the new modular, OS-aware tab design makes platform-conditional rendering straightforward.
- Task 02 (XDG) unblocks `US-windows/01-cargo-platform-deps` — proper XDG support means macOS config resolution is clean before adding Windows `%APPDATA%` alongside it.
- Task 03 (Terminal detection) investigates how to detect installed terminals on macOS, feeding into Task 01's expanded-row details.
