# herdr-picker

Two pieces of workspace navigation [herdr](https://herdr.dev) has no built-in for:

- **Create-or-focus by label.** `herdr workspace create` is not idempotent and `herdr workspace
  focus` takes an id rather than a label, so a quick-jump chord needs to resolve the label against
  the live workspace list first and create the workspace only when no match comes back.
- **A most-recently-used toggle.** herdr ships `last_pane` and no workspace equivalent. The
  `workspace.focused` event hook records every focus change, mouse and picker included, which is
  what makes the toggle correct where a key-bound script was not: a script only sees the switches
  routed through itself.
- **A one-letter pick popup.** A herdr binding is a single chord, so "prefix, then a letter" has
  to read its second key somewhere. `pick` reads it inside a herdr popup that sizes itself to the
  herdr window, and jumps to whichever workspace that letter names.

It talks to herdr over `HERDR_SOCKET_PATH` with newline-delimited JSON. A jump or a toggle falls
back to the `herdr` CLI at `HERDR_BIN_PATH` when the socket fails. The pick popup only asks herdr
which workspaces are open; when herdr does not answer, the popup still opens, marks nothing as open,
and says so on the row above the legend.

## Install

**This plugin ships no `herdr-plugin.toml`, so `herdr plugin install` cannot install it.** The jump
actions are one per workspace you declare, so the manifest is a function of your configuration
rather than a file that can be committed once for everybody. It is built, generated and linked
instead. A Rust toolchain has to be on the machine.

```bash
git clone https://github.com/webdavis/herdr-picker.git ~/.local/share/herdr-picker
cd ~/.local/share/herdr-picker
cargo build --release --locked
```

Declare your workspaces in `~/.config/herdr/plugins/config/herdr-picker/config.toml`. The
key is the label herdr shows, which is case sensitive, and the value is either the working
directory on its own or a table naming that directory and the key the pick popup selects it with:

```toml
[workspaces]
homelab = "~/workspaces/homelab"
"casually-concerned" = "~/workspaces/casually-concerned"
Ivy = { dir = "~/workspaces/Ivy", key = "v" }
```

A leading `~` is expanded when the jump runs, so it can stay in the file.

A `[picker]` table switches on two ways of drawing the rows, both off without it. `leaders` runs
faint dots from each name to its directory, and `stripes` shades every other row. Each is `true` or
`false`, and either or both can be on. `theme` names the popup's colors; `catppuccin-mocha` is the
only one so far and the default:

```toml
[picker]
leaders = true
stripes = true
theme = "catppuccin-mocha"
```

`key` defaults to the label's first character lowercased, which is why `homelab` picks on `h` and
`Ivy` would pick on `i` without the `v` above. It has to be exactly one printable ASCII character
other than `/`, which starts a search in the popup, and two workspaces resolving to the same key are
refused when the config is read, in one sentence naming both labels. `generate` reads the same file,
so that refusal fails a manifest render as much as a popup: a new workspace whose key is already
taken means giving one of the two its own `key`.

Then render the manifest and link the directory:

```bash
./target/release/herdr-picker generate --output .
herdr plugin link ~/.local/share/herdr-picker
```

Re-run `generate`, and re-link, whenever the config changes or the plugin is updated, since an
update can add actions, as the self-sizing picker did. The manifest is build output rather than
source, and is deliberately not committed.

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
command = "herdr-picker.last_workspace"
```

## The pick popup

`pick` is laid out so nothing moves while you use it. The top row is the search prompt, a faint `/
search` until `/` is pressed, then what has been typed, with how many workspaces it kept at the
right edge; until a letter is typed it hints `type to filter` after the cursor. Below a blank row
come the workspaces, one per row: the key right-aligned in bold blue, the name, in
green when herdr already has that workspace open, and its directory in faint text in a column after
the widest name offered, and, if switched on, leaders running across the gap or every other row
shaded. The selected row sits on a dark grey bar, the way fzf marks its current line. The
workspace the popup opened from is left out, since there is nowhere to jump; its key puts `already
in` and its name in red on the row above the legend until the next key; when herdr did not answer at
all, a faint note says so on that row instead. The bottom row is a faint legend of the keys that
move. The popup paints its own background and text in Catppuccin Mocha, whatever the terminal's
own colors are, so it looks the same everywhere. A workspace's key jumps to it at once. `ctrl+n`,
`ctrl+p` and the arrow keys move the bar, enter jumps to the row under it, and a list taller than
the popup scrolls to keep that row in view.
While searching, the key column goes blank and what is typed keeps only the workspaces whose labels
hold those letters in that order, ignoring case, best match first, with the matched letters lit in
bold blue: a run of the typed letters beats a scatter, an early match beats a late one and a short
label beats a long one. The search field edits like herdr's own fields, with the keys below, and
backspace on an empty search ends it. `esc`, ctrl-c and ctrl-g always close the popup without
jumping. Outside a search, so does any key no workspace claims, `q` included.

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

Bind its plugin action:

```toml
[[keys.command]]
key = "prefix+ctrl+o"
type = "plugin_action"
command = "herdr-picker.pick"
description = "pick a workspace"
```

The popup sizes itself, so the binding names no width or height. It takes 40% of the herdr window
each way, at least 60 columns by 14 rows and at most 100 by 30. It grows wider only when the message
refusing a workspace's key, or the legend, needs more room, and never past the window; a name or a
directory never widens it on its own. Inside it, herdr's border takes three columns and two rows,
and four rows go to the prompt, the legend and the blank rows around the list. A directory too long
for what is left of its row is cut short with an ellipsis, or left out, and so is a label too long
for its row or a message above the legend, so the list, the hint and the legend never wrap.

The action runs `herdr-picker popup`. That asks herdr for the window's size and opens the
manifest's `pick` pane as a popup at the size worked out from it, telling the pane which workspace
the key was pressed in. The pane runs `herdr-picker pick`, which reads the same config file
the manifest comes from and has no `--config` flag; `HERDR_PLUGIN_CONFIG_DIR` is what points it
elsewhere. A `type = "popup"` binding that runs `pick` itself still works, at whatever size that
binding gives.

The jump does not happen inside the popup. herdr may restore focus to the pane that opened the
popup once the popup's command exits, which would undo it, so `pick` spawns a detached
`jump <label> <cwd> --after-pid <pid>` and exits at once. That child waits for the popup to be gone
before it asks herdr for anything, and gives up waiting after two seconds.

## Pickers over any pane

The pickers' shell keys, such as `ctrl-t f`, have a herdr twin: `prefix+ctrl+t f`. `generate` renders
one `launch_` action per first key the pickers share, `launch_ctrl_t` and `launch_ctrl_g` with the
built-in keys. Each opens a popup over the focused pane, drawn like the pick popup, listing the
pickers on that key by the key that follows it. Typing that key starts the picker; enter searches
the list. A picker whose second key holds ctrl, like the bookmark, is not offered there.

```toml
[[keys.command]]
key = "prefix+ctrl+t"
type = "plugin_action"
command = "herdr-picker.launch_ctrl_t"

[[keys.command]]
key = "prefix+ctrl+g"
type = "plugin_action"
command = "herdr-picker.launch_ctrl_g"
```

The picker opens in its own popup in that pane's folder, and its result goes to that pane the way
what runs there takes it:

| Result | Shell | Claude or Codex |
| --- | --- | --- |
| Text or paths | typed | typed, paths as `@path` for Claude |
| A command to finish | typed | `! <command>` typed |
| A folder | `cd <folder>` run | `! cd <folder>` sent |
| A command to run | run | `! <command>` sent |
| An editor, `ssh`, `sudo`, `docker exec` | run | run in a new pane to the right |

Before typing into an agent, herdr-picker reads its input line. If it holds typing, or the agent
waits on a question, nothing is sent and a herdr notification says why; the draft is never
cleared. Any other agent is refused the same way.

On a row that runs, edits or changes folder, alt+h, alt+j, alt+k and alt+l open the result in a new
pane left of, below, above or right of the pane underneath, and alt+n in a new tab. They work from
the shell keys inside herdr too, and say so in the footer outside it.

The `record` hook also runs on every pane focus and keeps the last 200 places, each pane and folder
once, in `places` in the plugin's state folder, for a picker of recent places.

## Developing

`just preview` builds the plugin and opens the picker in a throwaway herdr with your theme and
workspaces, so a change is seen in a real popup before it ships. Run it from a herdr pane. `ctrl+x
ctrl+o` reopens the popup; `exit` or `ctrl+x q` leaves. Your real config and registry are only read;
everything it writes goes under `target/`. `just preview --stripes` turns a `[picker]` switch on
without changing your config.

The pick popup is put together from parts that each sit behind a trait, so each is tested alone with
a small fake, and any one can be replaced where the parts are chosen, in the CLI's `pick` and
`popup`:

- `Matcher` ranks items by how well their labels match a search. `Fuzzy` is the one in use.
- `TextField` edits the search. `ReadlineField` is the one in use.
- `Keymap` turns keys into the menu's commands. `ReadlineKeymap` is the one in use.
- `Menu` holds the popup's state, takes commands, and describes what to show as a `Frame`, without
  knowing keys or terminals.
- `pick`, in the application crate, runs the popup's loop against an `InputSource` and a `Screen`.
  The adapters crate gives it the popup's terminal as `TerminalInput` and `TerminalScreen`, the only
  code that knows ANSI.
- `open_picker`, in the application crate, opens the popup through a `PopupHost` at the size the
  domain's `popup_size` gives for the host's window and the columns
  `TerminalScreen::columns_to_fit` measures. `SocketPopupHost` is the host in use, over herdr's
  socket.

The colors are one `Palette` per theme in `palette.rs` beside the view. Another theme is a pull
request adding a palette there, its name to the domain's `Theme`, and its `[picker]` value.

## License

MIT
