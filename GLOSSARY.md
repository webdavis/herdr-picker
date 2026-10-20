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
