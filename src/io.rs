use std::io::Write;

pub fn get_input<T>(message: T) -> Result<String, String>
where
    T: std::fmt::Display,
{
    print!("{}", message);
    std::io::stdout()
        .flush()
        .map_err(|e| format!("failed to flush stdout: {}", e))?;

    let mut s = String::new();
    std::io::stdin()
        .read_line(&mut s)
        .map_err(|e| format!("failed to read line from stdin: {}", e))?;

    Ok(s.trim().to_string())
}
