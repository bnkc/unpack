pub fn print_error(msg: impl Into<String>) {
    eprintln!("[pip-udeps error]: {}", msg.into());
}
