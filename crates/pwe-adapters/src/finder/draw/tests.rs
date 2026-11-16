use pwe_domain::finder::{Finder, Input, step};
use pwe_domain::{Action, Fuzzy, Key, PickerRow, Theme, View, ViewState};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};

use super::draw;

fn row(label: &str) -> PickerRow {
    PickerRow {
        label: label.to_string(),
        preview: None,
        enter: Action::Insert(label.to_string()),
        keys: Vec::new(),
        group: None,
    }
}

fn finder_of(rows: Vec<PickerRow>) -> Finder<Fuzzy> {
    let state = ViewState {
        view: View::Streamed { group: None },
        titles: vec!["Files".to_string()],
        description: "Find a file".to_string(),
        source: None,
        sources: Vec::new(),
    };
    let finder = Finder::new(Fuzzy).with_view(state);
    let (finder, _) = step(finder, Input::RowsArrived(rows));
    step(finder, Input::RowsEnded).0
}

fn after(finder: Finder<Fuzzy>, inputs: Vec<Input>) -> Finder<Fuzzy> {
    inputs
        .into_iter()
        .fold(finder, |finder, input| step(finder, input).0)
}

fn key(name: &str) -> Input {
    Input::Key(name.to_string())
}

fn drawn(finder: &Finder<Fuzzy>, width: u16, height: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("a test terminal");
    terminal
        .draw(|frame| draw(frame, finder, Theme::default()))
        .expect("a drawn frame");
    terminal.backend().buffer().clone()
}

fn lines(buffer: &Buffer) -> Vec<String> {
    let area = buffer.area;
    (0..area.height)
        .map(|y| {
            let line: String = (0..area.width).map(|x| buffer[(x, y)].symbol()).collect();
            line.trim_end().to_string()
        })
        .collect()
}

#[test]
fn the_first_frame_shows_the_title_the_prompt_the_rows_and_the_hints() {
    let finder = finder_of(vec![row("alpha"), row("beta")]);

    assert_eq!(
        lines(&drawn(&finder, 60, 12)),
        [
            " /                          2 │",
            "Files                         │",
            "  alpha                       │",
            "  beta                        │",
            "                              │",
            "                              │",
            "                              │",
            "                              │",
            "                              │",
            "enter insert · alt-o send     │",
            "F1 help                       │",
            "Find a file                   │",
        ]
    );
}

#[test]
fn a_coloured_label_keeps_its_colour_and_the_match_is_drawn_over_it() {
    let finder = after(
        finder_of(vec![row("\x1b[31mred\x1b[0m")]),
        vec![key("r"), key("d")],
    );

    let buffer = drawn(&finder, 40, 6);

    let matched = &buffer[(2, 2)];
    assert_eq!(matched.fg, Color::Rgb(137, 180, 250));
    assert!(matched.modifier.contains(Modifier::BOLD));
    assert_eq!(buffer[(3, 2)].fg, Color::Red);
    assert_eq!(buffer[(4, 2)].fg, Color::Rgb(137, 180, 250));
}

#[test]
fn the_group_column_is_drawn_dim_before_the_label() {
    let grouped = |label: &str, group: &str| PickerRow {
        group: Some(group.to_string()),
        ..row(label)
    };
    let finder = finder_of(vec![grouped("one", "docs"), grouped("two", "src")]);

    let buffer = drawn(&finder, 40, 8);

    assert_eq!(lines(&buffer)[2], "  docs  one         │");
    assert_eq!(lines(&buffer)[3], "  src   two         │");
    assert!(buffer[(2, 2)].modifier.contains(Modifier::DIM));
    assert!(!buffer[(8, 2)].modifier.contains(Modifier::DIM));
}

#[test]
fn a_marked_row_shows_the_mark_and_the_current_row_is_lit() {
    let finder = after(finder_of(vec![row("one"), row("two")]), vec![key("tab")]);

    let buffer = drawn(&finder, 40, 8);

    assert_eq!(lines(&buffer)[2], " ┃one               │");
    assert_eq!(buffer[(5, 3)].bg, Color::Rgb(54, 54, 70));
    assert_eq!(buffer[(5, 2)].bg, Color::Rgb(30, 30, 46));
}

#[test]
fn the_match_count_sits_at_the_right_of_the_search_line() {
    let finder = after(finder_of(vec![row("one"), row("two")]), vec![key("w")]);

    assert_eq!(lines(&drawn(&finder, 40, 6))[0], " /w               1 │");
}

#[test]
fn the_preview_shows_in_colour_on_the_right_half_beside_a_left_border() {
    let preview = Input::PreviewArrived {
        row: 0,
        text: "\x1b[32mhello\x1b[0m\nworld".to_string(),
    };
    let finder = after(finder_of(vec![row("one")]), vec![preview]);

    let buffer = drawn(&finder, 40, 6);

    assert_eq!(lines(&buffer)[0], " /                1 │hello");
    assert_eq!(lines(&buffer)[1], "Files               │world");
    assert_eq!(buffer[(21, 0)].fg, Color::Green);
}

#[test]
fn f1_draws_the_help_in_the_preview_pane() {
    let finder = after(finder_of(vec![row("one")]), vec![key("f1")]);

    let lines = lines(&drawn(&finder, 80, 12));

    assert!(lines[0].ends_with("│Files › Help"), "{lines:?}");
    assert!(lines[2].ends_with("│enter   insert one"), "{lines:?}");
}

#[test]
fn the_chooser_takes_the_whole_width_without_a_preview() {
    let state = ViewState {
        view: View::Streamed { group: None },
        titles: vec!["Files".to_string()],
        description: String::new(),
        source: None,
        sources: vec!["home".to_string(), "disk".to_string()],
    };
    let finder = Finder::new(Fuzzy).with_view(state);
    let finder = after(
        finder,
        vec![Input::RowsArrived(vec![row("one")]), key("ctrl-space")],
    );

    let lines = lines(&drawn(&finder, 40, 6));

    assert_eq!(lines[1], "Files › Choose source");
    assert_eq!(lines[2], "  home");
    assert!(lines.iter().all(|line| !line.contains('│')));
}

#[test]
fn copied_replaces_the_footer_and_a_refusal_is_red() {
    let copying = PickerRow {
        keys: vec![(
            Key::parse("alt-y").expect("a key"),
            Action::CopyStay("x".into()),
        )],
        ..row("one")
    };
    let copied = after(finder_of(vec![copying]), vec![key("alt-y")]);
    let refused = after(finder_of(vec![row("one")]), vec![key("ctrl-q")]);

    assert_eq!(lines(&drawn(&copied, 40, 6))[5], "Copied              │");
    let buffer = drawn(&refused, 40, 6);
    assert_eq!(lines(&buffer)[4], "No file locations in│");
    assert_eq!(buffer[(0, 4)].fg, Color::Rgb(243, 139, 168));
}
