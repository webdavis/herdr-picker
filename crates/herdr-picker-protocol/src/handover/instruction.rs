use herdr_picker_domain::NewPane;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Insert,
    Cd,
    Run,
    // The text is a herdr call's JSON.
    Herdr,
}

// A line `instruction` printed, read back: `<verb>\t<text>`, led by `<new pane>\t` for Alt-H/J/K/L/N.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub pane: Option<NewPane>,
    pub verb: Verb,
    pub text: String,
}

pub fn parse_instruction(line: &str) -> Option<Instruction> {
    let (first, rest) = line.split_once('\t')?;
    let (pane, verb, text) = match NewPane::parse(first) {
        Some(pane) => {
            let (verb, text) = rest.split_once('\t')?;
            (Some(pane), verb, text)
        }
        None => (None, first, rest),
    };
    let verb = match verb {
        "insert" => Verb::Insert,
        "cd" => Verb::Cd,
        "run" => Verb::Run,
        "herdr" => Verb::Herdr,
        _ => return None,
    };
    Some(Instruction {
        pane,
        verb,
        text: text.to_string(),
    })
}
