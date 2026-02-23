use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const MAX_HISTORY_LINES: usize = 200;

pub fn history_path() -> PathBuf {
    let base_dir = dirs::data_local_dir()
        .or_else(|| dirs::home_dir().map(|home| home.join(".local").join("share")))
        .unwrap_or_else(std::env::temp_dir);
    base_dir.join("yeet").join("history.txt")
}

pub fn record_launch(app_name: &str) {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let path = history_path();

    let result = (|| -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        ensure_not_symlink(&path)?;
        let mut file = open_history_for_append(&path)?;
        writeln!(file, "{}\t{}", timestamp, app_name)?;
        Ok(())
    })();

    if result.is_ok() {
        if let Ok(meta) = fs::metadata(&path) {
            if meta.len() > (MAX_HISTORY_LINES as u64) * 100 {
                trim_history(MAX_HISTORY_LINES);
            }
        }
    }
}

pub fn load_history() -> HashMap<String, u64> {
    let path = history_path();
    let file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => return HashMap::new(),
    };

    let reader = std::io::BufReader::new(file);
    let mut history = HashMap::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if let Some((ts_str, name)) = line.split_once('\t') {
            if let Ok(ts) = ts_str.parse::<u64>() {
                let entry = history.entry(name.to_string()).or_insert(0u64);
                if ts > *entry {
                    *entry = ts;
                }
            }
        }
    }

    history
}

pub fn trim_history(max_lines: usize) {
    let path = history_path();

    let _ = (|| -> std::io::Result<()> {
        ensure_not_symlink(&path)?;
        let content = fs::read_to_string(&path)?;
        let mut entries: Vec<(u64, &str)> = content
            .lines()
            .filter_map(|line| {
                let (ts_str, name) = line.split_once('\t')?;
                let ts = ts_str.parse::<u64>().ok()?;
                Some((ts, name))
            })
            .collect();

        if entries.len() <= max_lines {
            return Ok(());
        }

        entries.sort_by(|a, b| b.0.cmp(&a.0));
        entries.truncate(max_lines);
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let (temp_path, mut file) = create_temp_history_file(&path)?;
        for (ts, name) in entries {
            writeln!(file, "{}\t{}", ts, name)?;
        }
        drop(file);
        fs::rename(&temp_path, &path).map_err(|rename_err| {
            let _ = fs::remove_file(&temp_path);
            rename_err
        })?;
        Ok(())
    })();
}

fn ensure_not_symlink(path: &Path) -> std::io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };

    if metadata.file_type().is_symlink() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "history path cannot be a symlink",
        ));
    }

    Ok(())
}

fn open_history_for_append(path: &Path) -> std::io::Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    options.open(path)
}

fn create_temp_history_file(path: &Path) -> std::io::Result<(PathBuf, fs::File)> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "history path has no parent",
        )
    })?;
    let unique = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_path = parent.join(format!(".history.{}.{}.tmp", std::process::id(), unique));

    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }

    options.open(&temp_path).map(|f| (temp_path, f))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn parse_history_from_str(input: &str) -> HashMap<String, u64> {
        let mut history = HashMap::new();
        for line in input.lines() {
            if let Some((ts_str, name)) = line.split_once('\t') {
                if let Ok(ts) = ts_str.parse::<u64>() {
                    let entry = history.entry(name.to_string()).or_insert(0u64);
                    if ts > *entry {
                        *entry = ts;
                    }
                }
            }
        }
        history
    }

    #[test]
    fn load_parses_valid_lines() {
        let input = "1000\tfirefox\n2000\tterminal\n3000\tfirefox\n";
        let history = parse_history_from_str(input);

        assert_eq!(history.len(), 2);
        assert_eq!(history["firefox"], 3000);
        assert_eq!(history["terminal"], 2000);
    }

    #[test]
    fn load_skips_malformed_lines() {
        let input = "not_a_number\tfirefox\n\nbadline\n1500\tvalid_app\n";
        let history = parse_history_from_str(input);

        assert_eq!(history.len(), 1);
        assert_eq!(history["valid_app"], 1500);
    }

    #[test]
    fn trim_keeps_only_max_lines() {
        let dir = std::env::temp_dir().join("yeet_test_trim");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("history.txt");

        let mut file = fs::File::create(&path).unwrap();
        for i in 0..10u64 {
            writeln!(file, "{}\tapp_{}", i * 100, i).unwrap();
        }
        drop(file);

        let content = fs::read_to_string(&path).unwrap();
        let mut entries: Vec<(u64, &str)> = content
            .lines()
            .filter_map(|line| {
                let (ts_str, name) = line.split_once('\t')?;
                let ts = ts_str.parse::<u64>().ok()?;
                Some((ts, name))
            })
            .collect();

        assert_eq!(entries.len(), 10);

        entries.sort_by(|a, b| b.0.cmp(&a.0));
        entries.truncate(5);
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let mut file = fs::File::create(&path).unwrap();
        for (ts, name) in &entries {
            writeln!(file, "{}\t{}", ts, name).unwrap();
        }
        drop(file);

        let remaining = fs::read_to_string(&path).unwrap();
        let line_count = remaining.lines().count();
        assert_eq!(line_count, 5);

        assert!(remaining.contains("app_9"));
        assert!(remaining.contains("app_5"));
        assert!(!remaining.contains("app_0"));

        let _ = fs::remove_dir_all(&dir);
    }
}
