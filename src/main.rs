use anyhow::{bail, ensure, Result};
use std::process::Command;

fn main() {
    let exit_code = match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err}");
            1
        }
    };

    std::process::exit(exit_code);
}

fn run() -> Result<i32> {
    let command = resolved_command_name()?;
    let forwarded_args = std::env::args()
        .skip(1)
        .map(|arg| convert_argument(&arg))
        .collect::<Result<Vec<_>>>()?;
    let status = Command::new("wsl.exe")
        .arg("-e")
        .arg(command)
        .args(forwarded_args)
        .status()?;
    Ok(status.code().unwrap_or(1))
}

fn convert_argument(arg: &str) -> Result<String> {
    if let Some((key, value)) = split_key_value(arg) {
        if is_windows_path(value) {
            let converted = wsl_paths(value, None, true)?;
            return Ok(format!("{key}={converted}"));
        }
        return Ok(arg.to_string());
    }
    if is_windows_path(arg) {
        return wsl_paths(arg, None, true);
    }
    Ok(arg.to_string())
}

fn split_key_value(arg: &str) -> Option<(&str, &str)> {
    arg.split_once('=')
        .filter(|(key, value)| key.starts_with('-') && !value.is_empty())
}

fn is_windows_path(value: &str) -> bool {
    matches!(value.as_bytes(), [drive, b':', sep, ..]
        if drive.is_ascii_alphabetic() && (*sep == b'\\' || *sep == b'/'))
}

fn resolved_command_name() -> Result<String> {
    let exe_path = std::env::current_exe()?;
    let file_stem = exe_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "unable to determine executable name",
            )
        })?;
    ensure!(
        !file_stem.eq_ignore_ascii_case("wslx"),
        "the executable is still named wslx; rename wslx.exe to the command you want to forward, e.g., git.exe"
    );
    Ok(file_stem.to_string())
}

fn wsl_paths(
    path: &str,
    distro: Option<&str>,
    to_linux_path: bool,
) -> Result<String> {
    let mut cmd = Command::new("wsl.exe");
    if let Some(distro_name) = distro {
        cmd.args(["-d", distro_name]);
    }
    let path_arg = if to_linux_path { "-a" } else { "-m" };
    let output = cmd
        .arg("-e")
        .arg("wslpath")
        .arg(path_arg)
        .arg(path.replace('\\', "\\\\"))
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{}", stderr.trim());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn wsl_to_windows(path: &str) -> Result<String> {
        wsl_paths(path, None, false)
    }

    pub fn windows_to_wsl(path: &str) -> Result<String> {
        wsl_paths(path, None, true)
    }

    #[test]
    fn test_wsl_to_windows() {
        assert_eq!(wsl_to_windows("/mnt/c").unwrap_or_default(), "C:/");
    }
    #[test]
    fn test_windows_to_wsl() {
        assert_eq!(windows_to_wsl("C:/").unwrap_or_default(), "/mnt/c/");
    }
}
