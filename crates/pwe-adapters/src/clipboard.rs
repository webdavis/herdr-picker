use std::io::{self, Write};
use std::process::{Command, Stdio};

pub fn copy_to_clipboard(text: &str) -> io::Result<()> {
    let mut pbcopy = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;
    pbcopy
        .stdin
        .take()
        .expect("pbcopy's stdin is piped")
        .write_all(text.as_bytes())?;
    let status = pbcopy.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("pbcopy exited with {status}")))
    }
}
