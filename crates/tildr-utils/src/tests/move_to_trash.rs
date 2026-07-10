#[cfg(test)]
mod tests {

  use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
  };

  #[cfg(target_os = "macos")]
  use std::collections::HashSet;

  use super::fs::move_to_trash;

  fn unique_name(prefix: &str) -> String {
    let nanos = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after UNIX_EPOCH")
      .as_nanos();
    format!("{prefix}-{}-{nanos}", std::process::id())
  }

  #[cfg(target_os = "macos")]
  fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
      .map(PathBuf::from)
      .expect("HOME should be set for trash tests")
  }

  fn test_base_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
      return std::env::temp_dir().join(".tildr-test-trash");
    }

    #[cfg(target_os = "macos")]
    {
      return home_dir().join(".tildr-test-trash");
    }

    #[allow(unreachable_code)]
    std::env::temp_dir().join(".tildr-test-trash")
  }

  fn create_source_file(prefix: &str) -> (PathBuf, String) {
    let name = unique_name(prefix);
    let root = test_base_dir().join(&name);
    fs::create_dir_all(&root).expect("test directory should be created");

    let path = root.join(format!("{name}.txt"));
    fs::write(&path, b"tildr trash test").expect("test file should be created");

    (path, name)
  }

  fn remove_empty_parents(path: &Path) {
    let test_root = test_base_dir();
    let mut current = path.parent().map(Path::to_path_buf);

    while let Some(dir) = current {
      if dir == test_root {
        let _ = fs::remove_dir(&dir);
        break;
      }

      if fs::remove_dir(&dir).is_err() {
        break;
      }

      current = dir.parent().map(Path::to_path_buf);
    }
  }

  fn wait_for<T, F>(mut op: F) -> T
  where
    F: FnMut() -> Option<T>,
  {
    for _ in 0..50 {
      if let Some(value) = op() {
        return value;
      }
      thread::sleep(Duration::from_millis(100));
    }

    panic!("timed out waiting for trash operation");
  }

  #[cfg(target_os = "linux")]
  fn linux_trash_file_path(item: &trash::TrashItem) -> PathBuf {
    let info_file = PathBuf::from(&item.id);
    let name_in_trash = info_file
      .file_stem()
      .expect("trash info file should have a file stem");

    info_file
      .parent()
      .and_then(Path::parent)
      .expect("trash info file should live inside an info directory")
      .join("files")
      .join(name_in_trash)
  }

  #[cfg(target_os = "linux")]
  #[test]
  fn move_to_trash_moves_file_into_linux_trash() {
    let (source_path, name_prefix) = create_source_file("move-to-trash-linux");

    move_to_trash(&source_path).expect("file should be moved to trash");
    assert!(
      !source_path.exists(),
      "source file should no longer exist after trashing"
    );

    let item = wait_for(|| {
      trash::os_limited::list().ok().and_then(|items| {
        items
          .into_iter()
          .find(|item| item.original_path() == source_path)
      })
    });

    let trashed_path = linux_trash_file_path(&item);
    assert!(
      trashed_path.exists(),
      "trashed file should exist inside the Linux trash backend"
    );
    assert!(
      item.name.to_string_lossy().starts_with(&name_prefix),
      "trash item should preserve the unique test prefix"
    );

    trash::os_limited::purge_all([item]).expect("test trash item should be purged");
    remove_empty_parents(&source_path);
  }

  #[cfg(target_os = "macos")]
  fn macos_trash_entries_with_prefix(prefix: &str) -> HashSet<PathBuf> {
    let trash_dir = home_dir().join(".Trash");

    fs::read_dir(trash_dir)
      .expect("macOS trash directory should be readable")
      .filter_map(Result::ok)
      .map(|entry| entry.path())
      .filter(|path| {
        path
          .file_name()
          .map(|name| name.to_string_lossy().starts_with(prefix))
          .unwrap_or(false)
      })
      .collect()
  }

  #[cfg(target_os = "macos")]
  #[test]
  fn move_to_trash_moves_file_into_macos_trash() {
    let (source_path, name_prefix) = create_source_file("move-to-trash-macos");
    let before = macos_trash_entries_with_prefix(&name_prefix);

    move_to_trash(&source_path).expect("file should be moved to trash");
    assert!(
      !source_path.exists(),
      "source file should no longer exist after trashing"
    );

    let trashed_path = wait_for(|| {
      let after = macos_trash_entries_with_prefix(&name_prefix);
      after.into_iter().find(|path| !before.contains(path))
    });

    assert!(
      trashed_path.exists(),
      "trashed file should exist inside ~/.Trash on macOS"
    );

    fs::remove_file(&trashed_path).expect("test trash item should be removed");
    remove_empty_parents(&source_path);
  }
}
