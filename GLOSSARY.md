# herdr-picker

Jumps between herdr workspaces by key, from a chord or a pick popup.

## Language

**herdr window**:
The area herdr lays its panes out in, which the pick popup takes its size from.
_Avoid_: terminal, screen

**Launcher**:
The popup a `launch_` action opens, which reads the key after a base key such as ctrl-t and starts
that picker.

**Pane underneath**:
The pane focused when a launcher opened, which the picker's result goes to.

**Recent place**:
A pane and its folder as herdr last focused them, logged by the `record` hook.

**Saved agent list**:
herdr's agents, workspaces and tabs as the status hook last saw them, which the agents picker reads
instead of asking herdr.

**Send queue**:
Rows alt+o could not send yet, because the agent waited on a question or its input line held typing;
the status hook sends each once its agent is free.

**Quick answer**:
A question about the marked rows, asked of `claude -p` on one clauth profile and shown as a page.

**Source**:
One of the lists a picker can show, such as `hidden` in the files picker. The first is the one it
opens on; ctrl-space chooses another, and the picker's command reads the chosen one in
`HERDR_PICKER_SOURCE`.
_Avoid_: mode, tab

**Group**:
A label a row carries, by which ctrl-space narrows the rows already listed, without running the
command again.
_Avoid_: category, filter

**Step**:
A page a row's `next` action opens, run on no key: a built-in one, or a script a picker declares
under `steps`. esc goes back to the step before it.
_Avoid_: subpicker, screen

**Live step**:
A picker or step whose command runs again each time the query changes, reading it in
`HERDR_PICKER_QUERY`, rather than listing once for fzf to filter.
_Avoid_: dynamic picker, reload

**Handover**:
Giving a picker's result to the target pane the way what runs there takes it: typed, run, `cd`,
or opened in a new pane, unless a draft or a waiting question refuses it.
_Avoid_: paste

**Target pane**:
The pane a picker's result goes to: the pane underneath for a launcher, or a new pane or tab for
alt+h, alt+j, alt+k, alt+l and alt+n.
_Avoid_: destination, output pane
