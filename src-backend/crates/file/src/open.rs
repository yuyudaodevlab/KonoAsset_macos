use std::path::Path;

pub fn open_in_file_manager<P>(path: P) -> Result<(), String>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.exists() {
        return Err("File or directory does not exist".into());
    }

    #[cfg(target_os = "macos")]
    {
        // showfile 0.1.1 autoreleases an already-autoreleased NSArray on macOS.
        // Use the system utility so revealing a file cannot corrupt our AppKit pool.
        let path = path.canonicalize().map_err(|e| e.to_string())?;
        let output = finder_command(&path, path.is_dir())
            .output()
            .map_err(|e| format!("Unable to open Finder: {e}"))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Unable to open Finder ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if path.is_dir() {
            opener::open(path).map_err(|e| e.to_string())
        } else {
            showfile::show_path_in_file_manager(path);
            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
fn finder_command(path: &Path, is_directory: bool) -> std::process::Command {
    let mut command = std::process::Command::new("/usr/bin/open");
    if is_directory {
        command.args(["-a", "Finder"]);
    } else {
        command.arg("-R");
    }
    // Pass the path directly, without shell interpolation or lossy UTF-8 conversion.
    command.arg("--").arg(path);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_path_returns_error() {
        let path = std::env::temp_dir()
            .join(format!("konoasset-missing-{}", std::process::id()))
            .join("missing.unitypackage");
        assert_eq!(
            open_in_file_manager(path),
            Err("File or directory does not exist".into())
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn finder_preserves_paths_and_reveals_files_without_launching_them() {
        use std::ffi::OsStr;

        let path = Path::new("/tmp/日本語 assets/'test' $(touch unwanted).unitypackage");
        let command = finder_command(path, false);
        assert_eq!(command.get_program(), OsStr::new("/usr/bin/open"));
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [OsStr::new("-R"), OsStr::new("--"), path.as_os_str()]
        );

        let command = finder_command(path.parent().unwrap(), true);
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [
                OsStr::new("-a"),
                OsStr::new("Finder"),
                OsStr::new("--"),
                path.parent().unwrap().as_os_str()
            ]
        );
    }
}
