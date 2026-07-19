use std::collections::HashMap;

use anyhow::Result;
use console::style;
use tildr_core::context::Context;
use tildr_utils::fs::format_size;

pub fn run(ctx: &Context) -> Result<()> {
  let entries = tildr_repo::scatildr_repo(&ctx.repo_path)?;

  let mut total_size: u64 = 0;
  let mut largest_size: u64 = 0;
  let mut largest_name = String::new();
  let mut extensions: HashMap<String, usize> = HashMap::new();

  for entry in &entries {
    if let Ok(meta) = std::fs::metadata(&entry.repo_path) {
      let size = meta.len();
      total_size += size;
      if size > largest_size {
        largest_size = size;
        largest_name = entry.relative.display().to_string();
      }
    }
    if let Some(ext) = entry.relative.extension().and_then(|e| e.to_str()) {
      *extensions.entry(ext.to_string()).or_insert(0) += 1;
    }
  }

  println!("{} {}", style("Managed files:").bold(), entries.len());
  println!(
    "{} {}",
    style("Total size:").bold(),
    format_size(total_size)
  );
  if !largest_name.is_empty() {
    println!(
      "{} {} ({})",
      style("Largest:").bold(),
      largest_name,
      format_size(largest_size)
    );
  }

  if !extensions.is_empty() {
    let mut exts: Vec<_> = extensions.into_iter().collect();
    exts.sort_by(|a, b| b.1.cmp(&a.1));
    let display: Vec<_> = exts
      .iter()
      .take(6)
      .map(|(ext, count)| format!(".{} ({})", ext, count))
      .collect();
    println!("{} {}", style("By extension:").bold(), display.join(", "));
  }

  Ok(())
}
