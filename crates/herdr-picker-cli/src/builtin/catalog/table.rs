use herdr_picker_domain::Action;

// A key binding as the chord table lists it: key, group, kind, action, description.
pub(super) struct Binding<'a> {
    pub(super) key: String,
    pub(super) group: &'a str,
    pub(super) kind: &'a str,
    pub(super) action: &'a str,
    pub(super) description: &'a str,
    pub(super) enter: Action,
}

// A readline command or macro is reached only by its key, so Enter leaves it be.
pub(super) fn table_bindings(table: &str) -> impl Iterator<Item = Binding<'_>> {
    table
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut fields = line.split('\t');
            let mut next = || fields.next().unwrap_or_default();
            let (key, group, kind, action, description) = (next(), next(), next(), next(), next());
            let enter = match kind {
                "run" => Action::Run(action.to_string()),
                "insert" | "function" => Action::Insert(action.to_string()),
                _ => Action::Nothing,
            };
            Binding {
                key: key.to_string(),
                group,
                kind,
                action,
                description,
                enter,
            }
        })
}
