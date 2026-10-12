# herdr-workspace-jump

Two pieces of workspace navigation [herdr](https://herdr.dev) has no built-in for:

- **Create-or-focus by label.** `herdr workspace create` is not idempotent and `herdr workspace
  focus` takes an id rather than a label, so a quick-jump chord needs to resolve the label against
  the live workspace list first and create the workspace only when no match comes back.
- **A most-recently-used toggle.** herdr ships `last_pane` and no workspace equivalent. The
  `workspace.focused` event hook records every focus change, mouse and picker included, which is
  what makes the toggle correct where a key-bound script was not: a script only sees the switches
  routed through itself.
- **A one-letter pick popup.** A herdr binding is a single chord, so "prefix, then a letter" has
  to read its second key somewhere. `pick` reads it inside a herdr popup and jumps to whichever
  workspace that letter names.

It talks to herdr over `HERDR_SOCKET_PATH` with newline-delimited JSON, and falls back to the
`herdr` CLI at `HERDR_BIN_PATH` whenever the socket fails.

## Install

**This plugin ships no `herdr-plugin.toml`, so `herdr plugin install` cannot install it.** The jump
actions are one per workspace you declare, so the manifest is a function of your configuration
rather than a file that can be committed once for everybody. It is built, generated and linked
instead. A Rust toolchain has to be on the machine.

```bash
git clone https://github.com/webdavis/herdr-workspace-jump.git ~/.local/share/herdr-workspace-jump
cd ~/.local/share/herdr-workspace-jump
cargo build --release --locked
```

Declare your workspaces in `~/.config/herdr/plugins/config/herdr-workspace-jump/config.toml`. The
key is the label herdr shows, which is case sensitive, and the value is either the working
directory on its own or a table naming that directory and the key the pick popup selects it with:

```toml
[workspaces]
homelab = "~/workspaces/homelab"
"casually-concerned" = "~/workspaces/casually-concerned"
Ivy = { dir = "~/workspaces/Ivy", key = "v" }
```

A leading `~` is expanded when the jump runs, so it can stay in the file.

`key` defaults to the label's first character lowercased, which is why `homelab` picks on `h` and
`Ivy` would pick on `i` without the `v` above. It has to be exactly one printable ASCII character
other than `/`, which starts a search in the popup, and two workspaces resolving to the same key are
refused when the config is read, in one sentence naming both labels. `generate` reads the same file,
so that refusal fails a manifest render as much as a popup: a new workspace whose key is already
taken means giving one of the two its own `key`.

Then render the manifest and link the directory:

```bash
./target/release/herdr-workspace-jump generate --output .
herdr plugin link ~/.local/share/herdr-workspace-jump
```

Re-run `generate`, and re-link, whenever the config changes. The manifest is build output rather
than source, and is deliberately not committed.

`generate` reads the config path above unless you point it elsewhere: `--config <file>` names one
file, and `HERDR_PLUGIN_CONFIG_DIR`, which herdr itself sets when it runs a plugin, names the
directory the `config.toml` sits in.

Each workspace gets the action id `jump_` plus its label, lowercased with every character outside
`a-z0-9` replaced by an underscore, so `Ivy` is `jump_ivy` and `casually-concerned` is
`jump_casually_concerned`. Two labels deriving the same id is refused rather than dropping one
silently. The `last_workspace` action and the `record` event hook are always rendered and need no
configuration.

Then bind the actions in `~/.config/herdr/config.toml`:

```toml
[[keys.command]]
key = "prefix+ctrl+\\"
type = "plugin_action"
command = "herdr-workspace-jump.last_workspace"
```

## The pick popup

`pick` is laid out so nothing moves while you use it. The top row is the search prompt, a faint `/
search` until `/` is pressed, then what has been typed, with how many workspaces it kept at the
right edge. Below a blank row come the workspaces, the key right-aligned beside the label in the
terminal's bold blue, with the selected row on a dark grey bar, the way fzf marks its current line,
and the workspace the popup opened from faint and marked `current`. The bottom row is a faint legend
of the keys that move. A workspace's key jumps to it at once. `ctrl+n`, `ctrl+p` and the arrow keys
move the bar, enter jumps to the row under it, and a list taller than the popup scrolls to keep that
row in view. While searching, the key column goes blank and what is typed keeps only the workspaces
whose labels hold those letters in that order, ignoring case, best match first: a run of the typed
letters beats a scatter, an early match beats a late one and a short label beats a long one. The
search field edits like herdr's own fields, with the keys below, and backspace on an empty search
ends it. `esc`, ctrl-c and ctrl-g always close the popup without jumping. Outside a search, so does
any key no workspace claims, `q` included.

| Search keys | What they do |
| --- | --- |
| Left / Right, ctrl+b / ctrl+f | Move one character |
| Home / End, ctrl+a / ctrl+e | Move to the beginning or end |
| alt+b / alt+f | Move one word |
| Backspace, ctrl+h | Delete the previous character |
| Delete, ctrl+d | Delete the next character |
| ctrl+u / ctrl+k | Cut to the beginning or end |
| ctrl+w, alt+backspace, ctrl+backspace | Cut the previous word |
| alt+d | Cut the next word |
| ctrl+y | Insert the last cut text |

herdr takes its prefix before the popup sees it, so when the prefix is `ctrl+d`, Delete is the way
to delete forward. alt shortcuts need the terminal to send alt rather than compose characters, which
on macOS is Ghostty's `macos-option-as-alt`. herdr's direct chords, the ones bound without the
prefix, do reach the popup.

Bind it as a popup rather than a plugin action:

```toml
[[keys.command]]
key = "prefix+ctrl+o"
type = "popup"
command = "~/.local/share/herdr-workspace-jump/target/release/herdr-workspace-jump pick"
description = "pick a workspace"
width = 40
height = 16
```

herdr runs a custom command keybinding through a shell, so the leading `~` above is expanded and an
absolute path is not required. `width` and `height` are terminal cells and include herdr's border,
which takes three columns and two rows, so a 40 by 16 popup gives the menu 37 columns and 14 rows.
Four of those rows are the prompt, the legend and the blank rows around the list, which leaves ten
for workspaces; a longer list scrolls, or the binding grows a row per workspace. Leave `width` and
`height` out and herdr opens the popup at half its pane area each way, so in a large terminal the
bar stretches and a gap opens above the legend. A label too long for its row is cut short with an
ellipsis, and a row with no room for `current` leaves the mark off, so the list, the hint and the
legend never wrap.

`pick` is a popup command rather than a plugin action, so `generate` does not render it into the
manifest and nothing needs regenerating when the binding changes. It reads the same config file the
manifest comes from and has no `--config` flag; `HERDR_PLUGIN_CONFIG_DIR` is what points it
elsewhere.

The jump does not happen inside the popup. herdr may restore focus to the pane that opened the
popup once the popup's command exits, which would undo it, so `pick` spawns a detached
`jump <label> <cwd> --after-pid <pid>` and exits at once. That child waits for the popup to be gone
before it asks herdr for anything, and gives up waiting after two seconds.

## Developing

`scripts/preview-popup` builds the release binary and opens the pick popup from this checkout in a
throwaway herdr session, with your own theme, so a change to the popup is seen in a real popup
before it is pushed and the installed clone rebuilt. Run it from a herdr pane and the popup opens by
itself; outside herdr it prints the chord. The session's prefix is `ctrl+x`, so it neither collides
with yours nor takes `ctrl+b` from the search: `ctrl+x ctrl+o` reopens the popup, and typing `exit`
or `ctrl+x q` leaves and tears the session down, which the shell inside says as it opens. A pick
inside it jumps within that throwaway session, which exercises the whole flow without touching the
real one. `HERDR_PLUGIN_CONFIG_DIR` is forwarded when set, and `PREVIEW_WIDTH` and `PREVIEW_HEIGHT`
size the popup. It writes only under `target/`, jump history included, and herdr's own sessions
directory.

The pick popup is put together from parts that each sit behind a trait, so each is tested alone with
a small fake, and any one can be replaced where the parts are chosen, in the CLI's `pick`:

- `Matcher` ranks items by how well their labels match a search. `Fuzzy` is the one in use.
- `TextField` edits the search. `ReadlineField` is the one in use.
- `Keymap` turns keys into the menu's commands. `ReadlineKeymap` is the one in use.
- `Menu` holds the popup's state, takes commands, and describes what to show as a `Frame`, without
  knowing keys or terminals.
- `pick`, in the application crate, runs the popup's loop against a `KeySource` and a `Screen`. The
  adapters crate gives it the popup's terminal as `TerminalKeys` and `TerminalScreen`, the only code
  that knows ANSI.

## License

MIT
