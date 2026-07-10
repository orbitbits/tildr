// :@ Keep the exits clean, stupid, no frills.
// pub const CHECK: &str = "";
// pub const CROSS: &str = "";
// pub const WARN: &str = "";
// pub const INFO: &str = "!";
// pub const ARROW: &str = "";
// pub const UNKNOWN: &str = "";
// pub const BROKEN: &str = "";

pub struct Icons {
  pub check: &'static str,
  pub cross: &'static str,
  pub warn: &'static str,
  pub info: &'static str,
  pub arrow: &'static str,
  pub unknown: &'static str,
  pub broken: &'static str,
  pub none: &'static str,
}

pub static FANCY: Icons = Icons {
  check: "✔ ",
  cross: "✖ ",
  warn: "⚠ ",
  info: "ℹ ",
  arrow: "→ ",
  unknown: "? ",
  broken: "⚡ ",
  none: "",
};

pub static PLAIN: Icons = Icons {
  check: "* ",
  cross: "x ",
  warn: "! ",
  info: "i ",
  arrow: "-> ",
  unknown: "? ",
  broken: "- ",
  none: "",
};

pub fn icons() -> &'static Icons {
  if supports_unicode() { &FANCY } else { &PLAIN }
}

fn supports_unicode() -> bool {
  // // Respect explicit opt-out.
  // if std::env::var("NO_COLOR").is_ok() {
  //   return false;
  // }
  // Check locale encoding.
  for var in &["LC_ALL", "LC_CTYPE", "LANG"] {
    if let Ok(val) = std::env::var(var)
      && (val.to_uppercase().contains("UTF-8") || val.to_uppercase().contains("UTF8"))
    {
      return true;
    }
  }
  false
}
