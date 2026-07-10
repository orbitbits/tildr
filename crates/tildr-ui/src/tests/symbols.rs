#[cfg(test)]
mod tests {
  use super::*;

  // cargo test -p tildr-ui --test symbols tests::visible_symbols -- --nocapture
  #[test]
  fn visible_symbols() {
    let ic = icons();
    println!("check:   {}", ic.check);
    println!("cross:   {}", ic.cross);
    println!("warn:    {}", ic.warn);
    println!("info:    {}", ic.info);
    println!("arrow:   {}", ic.arrow);
    println!("unknown: {}", ic.unknown);
    println!("broken:  {}", ic.broken);
  }
}
