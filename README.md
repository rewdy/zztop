# zztop

A tiny companion to [`zsh-z`](https://github.com/agkozak/zsh-z): run `zz` to see your top 5 frecency-ranked directories, pick one with arrow keys, and jump to it.

`zztop` doesn't replace `z` — it reads `zsh-z`'s datafile and never writes to it.

```
$ zz
❯ 1  ~/Workspace/experiments/zztop
  2  ~/Workspace/experiments/worktree-cli
  3  ~/Workspace/personal/aeraline
  4  ~/.claude/plugins/cache
  5  ~/Workspace/dotfiles

  [↑↓ move, ⏎ select, esc cancel]
```

## Install

```sh
cargo install --git https://github.com/rewdy/zztop
```

Or from a local clone:

```sh
cargo install --path .
```

Either way, this puts the `zztop` binary on your `PATH` (typically `~/.cargo/bin`).

## Setup

Add this to your `~/.zshrc`:

```sh
zz() {
  local dir
  dir="$(zztop)" && cd "$dir"
}
```

The function captures the binary's stdout (the chosen path) and `cd`s your current shell into it. On cancel or error, the `&&` short-circuits and your shell stays put.

## Usage

```sh
zz          # show top 5, arrow keys to move, Enter to jump, Esc to cancel
```

That's it.

## How it works

- Reads `$_Z_DATA` if set, otherwise `~/.z`.
- Parses `zsh-z`'s `path|rank|time` format.
- Ranks by `zsh-z`'s frecency formula (recency × frequency).
- Skips directories that no longer exist on disk.
- Renders an arrow-key picker on stderr; prints the chosen absolute path on stdout.

## Requires

- A working [`zsh-z`](https://github.com/agkozak/zsh-z) installation (so `~/.z` has data to read).
- zsh.

## License

MIT.
