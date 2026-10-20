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
silently. The `last_workspace` action, the `record` event hooks and the `agent-changed` hook are always
rendered and need no configuration.

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

## Pickers in the shell

`herdr-picker init bash` prints a function and one `bind -x` line per picker, for each of the
`emacs-standard`, `vi-insert` and `vi-command` keymaps, so the keys work whichever editing mode the
shell is in or switches to. The function calls `herdr-picker run`, so the binary has to be on
`PATH`. Load it from `~/.bashrc`:

```bash
eval "$(herdr-picker init bash)"
```

A picker's result comes back to the prompt: text is typed at the cursor, a folder is changed into,
and a command is run and added to the history. The pickers need no config file; without one the
built-in pickers run on their own keys.

## The built-in pickers

| Key | Picker | Sources, the first opened on |
| --- | --- | --- |
| `ctrl-t f` | files | this folder, hidden, home, whole disk, recent |
| `ctrl-t d` | folders | subfolders, parents, git, bookmarks |
| `ctrl-t ctrl-b` | bookmark | none: bookmarks the current folder for the folders picker |
| `ctrl-g s` | status | all, conflicts, staged, unstaged, untracked, stashes, tracked, ignored |
| `ctrl-g l` | history | commits, reflog, changes (live) |
| `ctrl-g b` | branches | branches, repos |
| `ctrl-g /` | contents | none (live) |
| `ctrl-g v` | git-config | none |
| `ctrl-t j` | recipes | none |
| `ctrl-t t` | tests | tests, errors, quickfix |
| `ctrl-t p` | processes | processes, services, logs |
| `ctrl-t n` | network | devices, captures, web checks (live), commands |
| `ctrl-t i` | data | files, clipboard, temporal |
| `ctrl-t a` | agents | everything, needs you, done, agents, terminals, workspaces, recent, worktrees, instructions, failures, start, sessions |
| `ctrl-t ;` | pickers | pickers, bindings |
| `ctrl-t m` | apple | messages, contacts, shortcuts |
| `ctrl-t w` | tabs | none |

A live source runs again as you type, rather than filtering what it listed. Each picker's keys
besides enter show in its footer and, in full, on F1. The ones worth knowing before you open it:

- **files.** Enter types the path. ctrl+enter edits, ctrl+o opens, alt+c changes into its folder,
  alt+y copies and stays, alt+. switches between this folder and hidden.
- **folders.** Enter changes into the folder; alt+y copies it, ctrl+o opens it.
- **status.** Enter types the path. A changed file adds ctrl+z, which stashes it. A stash's
  ctrl+enter applies it after asking. In ignored, ctrl+enter edits the rule that ignores the file.
- **history.** Enter types the hash. ctrl+enter checks it out, asking first when the tree is dirty;
  ctrl+t lists its files; alt+i shows it; alt+s toggles oldest first. changes lists the commits
  whose changes match what is typed.
- **branches.** Enter types the name. ctrl+enter checks out the branch or changes into its
  worktree, ctrl+t opens it in its own worktree, ctrl+e renames it, ctrl+x removes its worktree,
  alt+i shows it, ctrl+f fetches, ctrl+l pulls and alt+d prunes. repos lists the repositories with
  something to prune.
- **contents.** Lines matching what is typed; enter edits the file at the line. alt+x toggles
  regex, alt+. hidden files, alt+s recently changed first, alt+u untracked files, and alt+b shows
  the line's history.
- **git-config.** Enter types the setting's name, ctrl+enter edits where it is set, ctrl+v changes
  it and ctrl+x removes it. Secrets show as `[redacted]`.
- **recipes.** Enter types the recipe's command, ctrl+enter runs it.
- **tests.** tests lists, in order: the tests each saved discovery found, one row per test that
  runs only that test (a discovery that failed shows its exit code and output first); the nearest
  `package.json`'s scripts with "test" in their name, run with `npm --prefix <folder> run`; one
  "Project:" row for each project folder below the current one, which changes into it; for the
  nearest Swift, Cargo, pytest and Go project, a "Discover … tests" row that saves its test listing
  (`swift test list`, `cargo test -- --list`, `pytest --collect-only`, `go test -list`) only when
  you run it, plus "Run all Swift tests" and "Run all Rust tests"; Bashunit tests and scripts in
  unit, integration and e2e folders; and justfile recipes with "test" in their name. Reopen the
  picker after a discovery to see its tests. errors lists the saved compiler and linter messages,
  enter editing the file at the line, and rows that save a fresh `swift build`, `cargo check` or
  lint recipe. quickfix is the list ctrl+q saved.
- **processes.** Enter types the pid, alt+d the command listing its sockets. services lists
  Homebrew, launchd and Docker services, alt+d typing the command for their logs and alt+e a shell
  in a container. logs opens the last hour of unified logs or the crash reports.
- **network.** devices merges ARP, Bonjour, UniFi and the last scan: enter types the address,
  ctrl+s `ssh` to it, ctrl+x scans the network and alt+x types the scan instead, ctrl+o opens it
  in the browser, ctrl+y wakes it, ctrl+g reloads. captures lists `.pcap`, `.pcapng` and `.cap`
  files under this folder: enter opens the analysis menu, alt+. toggles hidden captures, ctrl+t
  newest first, and alt+f asks for another folder. web checks times a URL you type and checks its
  certificate. commands is the network command catalog: enter types a command, ctrl+enter runs it,
  alt+y copies it, and a command with blanks asks you to type them. Some entries open a step of
  their own: saved hosts lists `~/.ssh/config` aliases, following `Include`, `/etc/hosts` names and
  Tailscale peers; DNS record inspection takes a record type, then a host and an optional server
  for `dig`; build a packet capture takes an interface, then `name=value` settings (`host`,
  `proto`, `port`, `count`, `seconds`, `tool`, `file`); throughput test takes an iperf3 mode, then
  the server and optional seconds, port and bitrate.
- **data.** Opens a JSON, JSON Lines, YAML, TOON or SQLite file, or the clipboard, by its paths,
  zoomed into one value, by keys, by levels, by shape, or by tables. Enter types the jq path or
  zooms in, ctrl+enter edits the file at the value, alt+y copies the value with secrets hidden and
  alt+u with them, and alt+; copies it as JSON, YAML or TOON.
- **agents, apple, tabs.** See the sections below.

## Keys in every picker

| Key | Does |
| --- | --- |
| tab | Marks a row; a key then acts on every marked row |
| ctrl+space | Chooses a source or group |
| alt+h, alt+j, alt+k, alt+l | Open a row that runs, edits or changes folder in a new pane left, below, above or right, inside herdr |
| alt+n | The same, in a new tab |
| alt+o | Sends the marked rows to an agent, copies them, or asks a quick question about them |
| ctrl+q | Saves the marked rows' files and lines as Neovim's quickfix list and opens it |
| esc | Back a step, or closes on the first |
| F1 | Help: every key of the highlighted row, then these |

The footer names the highlighted row's keys in a few words, then the new-pane keys it leaves free,
ctrl+q when it has a file and line, alt+o and F1.

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

A shell gets the same care for a folder or a command to run, since Enter would submit whatever is
already typed. herdr-picker reads the shell's current line, the last line on its screen with text,
and takes what follows the first prompt character on it (`❯` or `❮` from Starship, or `$`, `#`,
`%` or `>` followed by a space) as typing. When there is typing, or no prompt character, the `cd`
or command is refused with a notification. Text or paths are typed at the cursor, after any typing,
without reading the line.

On a row that runs, edits or changes folder, alt+h, alt+j, alt+k and alt+l open the result in a new
pane left of, below, above or right of the pane underneath, and alt+n in a new tab. They work from
the shell keys inside herdr too, and say so in the footer outside it.

The `record` hook also runs on every pane focus and keeps the last 200 places, each pane and folder
once, in `places` in the plugin's state folder, for a picker of recent places.

The plugin's state folder is the one herdr names in `HERDR_PLUGIN_STATE_DIR`. A shell key, which
runs outside herdr, works it out the way herdr does: `$XDG_STATE_HOME/herdr/plugins/herdr-picker`,
or `~/.local/state/herdr/plugins/herdr-picker` without `XDG_STATE_HOME`.

## Your own pickers

A `[pickers.<name>]` table adds a picker or changes a built-in one. The name is letters, digits,
`-` and `_`. Every field is optional except that a new picker needs a `key` and exactly one of
`command` or `builtin`; an unknown field is refused at its line.

| Field | Default | Meaning |
| --- | --- | --- |
| `key` | the built-in's key | The chord, such as `"ctrl-t m"`: ctrl or alt and a letter, then plain letters or punctuation |
| `description` | the built-in's | The line under the rows |
| `command` | | A command line, split like the shell, or an array of a program and its arguments. A relative program is a script in the `pickers` folder beside `config.toml`, so a program on `PATH` needs its full path |
| `builtin` | | Runs one of the built-in pickers under this name |
| `args` | `[]` | Arguments after the command's own, before a `next`'s |
| `enabled` | `true` | `false` turns the picker off, a built-in one included |
| `title` | the name, title-cased | The header's title |
| `sources` | `[]` | The sources a script reads, in lowercase; refused on a built-in picker |
| `live` | `false` | The script runs again as the query changes; refused on a built-in picker, which decides that itself |
| `steps.<step>` | | A script only a row's `next` opens, with its own `command`, `sources` and `live` |

A table named for a built-in picker that names no `command` changes that built-in picker: its
`key` moves it, `enabled = false` turns it off. A configured picker that takes a built-in picker's
key replaces it there; two configured pickers on one key are refused, naming both.

```toml
[pickers.notes]
key = "ctrl-t m"
title = "Vault"
description = "Notes by folder, tag, or lately opened"
command = "notes.py"
sources = ["folders", "tags", "recent"]

[pickers.notes.steps.note-actions]
command = "notes.py actions"

[pickers.notes.steps.grep]
command = "notes.py grep"
sources = ["titles", "bodies"]
live = true
```

The first source is the one a picker or step opens on, ctrl-space chooses another, the header says
`Source: <source>`, and the script reads the chosen one in `HERDR_PICKER_SOURCE`. A picker or step
with `live = true` is run again each time the query changes, with the query in
`HERDR_PICKER_QUERY`, and fzf shows its rows as they are rather than filtering them. Without it
the script runs once and fzf filters what it printed.

A step belongs to its picker: a row's `next` finds it from that picker or from any step opened
after it, so two pickers may each have a step of the same name, and a picker's own step wins over a
built-in step. A step runs from the same folder, with the row's arguments after its own.

### What a script prints

A script prints one JSON object per line. A line `{"header": "text"}` shows under the title; every
other line is a row, and a line that is neither is refused with its line number in the footer.

| Row field | Meaning |
| --- | --- |
| `label` | The text shown, on one line; ANSI colors are kept |
| `enter` | The action enter takes |
| `keys` | Other keys and their actions, as `{"ctrl-o": action}` |
| `labels` | A few words for what a claimed key does, as `{"alt-y": "copy URL"}`, which help and the footer show in place of the action's verb; every labelled key must be in `keys` |
| `value` | Text, or a list of words, for a bare `insert`, `copy` or `copy_stay`; a list inserts as shell words and copies as lines |
| `preview` | Text, `{"path": "<file or folder>"}`, or `{"command": ["<program>", ...]}`, drawn when the row is highlighted |
| `group` | A lowercase group, which ctrl-space narrows the rows to |

A row may claim ctrl-enter (sent as alt-enter), ctrl and a letter, alt and a letter or digit,
alt-. , alt-; and F2 to F12, except the engine's own: ctrl-c, ctrl-h, ctrl-i, ctrl-m, ctrl-n,
ctrl-p, ctrl-q and alt-o. A key no row claims keeps fzf's meaning, such as ctrl-a for the start of
the line.

| Action | Does |
| --- | --- |
| `"insert"`, `{"insert": text or words}` | Types the text at the prompt |
| `"copy"`, `{"copy": ...}` | Copies the text and closes |
| `"copy_stay"`, `{"copy_stay": ...}` | Copies the text and stays |
| `{"cd": "<folder>"}` | Changes into the folder |
| `{"run": "<command>"}` | Runs the command |
| `{"edit": {"path": "...", "line": 3}}`, or a list of them | Opens the files in the editor |
| `{"open": "<file or URL>"}` | Opens it with the system's opener |
| `{"next": {"picker": "<name>", "args": [], "state": <json>, "query": "..."}}` | Opens a step, handing it `args` and `state` and typing `query` |
| `{"confirm": {"question": "...", "action": <action>}}` | Asks yes or no first |
| `{"source": "<source>"}` | Shows the same step from another source |
| `{"state": <json>}` | Runs the same step again with this state, as a toggle does |
| `"reload"`, `"back"`, `"none"` | Runs the step again, goes back a step, or does nothing |
| `{"herdr": {"focus_pane": id}}`, `focus_tab`, `focus_workspace`, `close_tab`, `{"prompt": {"pane_id": id, "text": "..."}}` | Asks herdr once the picker has closed |
| `{"in_pane": {"pane": "left", "action": <action>}}` | Opens a run, edit or cd in a new pane `left`, `below`, `above`, `right` or `tab` |

A command gets the picker's `args`, then the `next`'s `args`, and three variables:
`HERDR_PICKER_SOURCE`, the chosen source (unset without sources); `HERDR_PICKER_STATE`, the JSON a
`next` or `state` action carried (unset without one); and `HERDR_PICKER_QUERY`, what is typed, for
a live picker or step only.

`ctrl-t ;` lists every picker by its key; Enter starts one. Its bindings source lists every shell
key herdr-picker binds, then the rows of `~/.config/chord/bindings-menu.tsv` if there is one: Enter
runs a `run` row, types an `insert` or `function` row, and leaves a readline command or macro to its
key.

## The agents picker

`ctrl-t a` lists herdr's agents, one row each: herdr's status glyph (`◉` needs you, `◐` working,
`●` done, `✓` idle), the session's title, then the agent, workspace and tab, folder, branch and how
long it has been in that state. Agents that need you or just finished come first, then the panes
visited most recently; the current tab's agents come last. The preview is the agent's latest
output. Enter focuses the agent's pane, alt+x closes its tab after asking, and ctrl+r asks herdr
again.

Its sources are everything, needs you, done, agents, terminals (tabs with no agent), workspaces,
recent (the places the `record` hook logged), worktrees (agents in a linked git worktree),
instructions (every file that tells an agent how to work here; Enter opens it in the editor),
failures (saved test and check runs that failed), start (Claude on the personal or Broccoli clauth
profile, Codex, or Hermes and its profiles, in a new pane here) and sessions (past Claude and Codex
sessions started in this repository, with their first prompt; Enter resumes one in a new pane).
Outside herdr the running sources say herdr isn't running; the others still work.

A third hook, on `pane.agent_status_changed`, saves herdr's agents in `agents.json` in the plugin's
state folder, so the picker opens without asking herdr; a list older than 30 seconds is asked for
again.

alt+o in every picker sends the marked rows, as text, to a running agent. An agent that is free or
working, with nothing typed in its input line, gets them at once. One that waits on a question, or
whose input line holds your typing, gets them later: the rows wait in `send-queue.json`, and the
status hook sends them once that agent finishes its turn with an empty input line. Your draft is
never touched. The same step offers "copy instead", and "quick answer", which takes a question as
you type it, asks `claude -p` through clauth on the personal or Broccoli profile, and shows the
answer as a page: Enter copies it and Esc goes back. A Broccoli picker's rows are only asked on the
Broccoli profile.

## Apple apps and Arc tabs

`ctrl-t m` reads the Mac's own apps. Its sources are messages, contacts and shortcuts. Contacts
copies a contact's card on enter, alt+y copies and stays, alt+0 to alt+6 choose which fields
enter copies (all, name, phone, email, address, work, link), alt+7 lists the fields to pick
from, ctrl+v calls, and alt+m opens the contact's conversations. Reading the address book takes
seconds, so a copy is kept in `contacts.json` in the plugin's state folder until the address book
changes. Shortcuts prepares `shortcuts run` on enter, and ctrl+o prepares opening the editor.

Messages come from a [BlueBubbles](https://bluebubbles.app) server. Its URL goes in the config
and its server password in the Keychain; the password reaches `curl` on stdin, never on its
command line:

```toml
[pickers.apple.bluebubbles]
url = "https://messages.example.net:1234"
keychain-service = "BlueBubbles server password"
```

Store the password with `security add-generic-password -s "BlueBubbles server password" -a
"$USER" -w`. Without the table, messages says BlueBubbles is not configured. Attachments open
from the Messages database, which needs Full Disk Access for the terminal.

`ctrl-t w` lists Arc's open tabs by Space: enter switches to the tab, alt+y copies its URL, and
alt+b switches to Chrome's tabs and back. Both pickers say so in one row on other systems, and
when Arc is not running or not installed.

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
