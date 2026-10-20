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
