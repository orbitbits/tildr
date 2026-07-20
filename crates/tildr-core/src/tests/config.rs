use crate::{Config, CryptoMode, Git};

#[test]
fn legacy_git_config_defaults_available_and_enable() {
  let config: Config = toml::from_str(
    r#"
        [core]
        repo = "~/.dotfiles"

        [git]
        auto_commit = true
      "#,
  )
  .expect("legacy config should parse");

  assert!(config.git.available);
  assert_eq!(config.git.enable, None);
  assert!(config.git.auto_commit);
}

#[test]
fn git_enable_false_overrides_available() {
  let git = Git {
    available: true,
    enable: Some(false),
    auto_commit: true,
  };

  assert!(!git.operations_enabled());
  assert!(!git.auto_commit_enabled());
}

#[test]
fn git_operations_require_availability_even_when_explicitly_enabled() {
  let git = Git {
    available: false,
    enable: Some(true),
    auto_commit: true,
  };

  assert!(!git.operations_enabled());
  assert!(!git.auto_commit_enabled());
}

#[test]
fn serializing_default_config_includes_available_without_enable_override() {
  let toml = toml::to_string_pretty(&Config::default()).expect("config should serialize");

  assert!(toml.contains("[git]"));
  assert!(toml.contains("available = true"));
  assert!(toml.contains("auto_commit = true"));
  assert!(!toml.contains("enable ="));
}

#[test]
fn search_threshold_defaults_to_15_when_not_set() {
  let config: Config = toml::from_str(
    r#"
        [core]
        repo = "~/.dotfiles"

        [git]
        auto_commit = true
      "#,
  )
  .expect("config should parse without search_threshold");

  assert_eq!(config.core.search_threshold, 15);
}

#[test]
fn search_threshold_can_be_set_in_config() {
  let config: Config = toml::from_str(
    r#"
        [core]
        repo = "~/.dotfiles"
        search_threshold = 5

        [git]
        auto_commit = true
      "#,
  )
  .expect("config should parse with search_threshold");

  assert_eq!(config.core.search_threshold, 5);
}

#[test]
fn crypto_defaults_to_symmetric_with_no_gpg_key() {
  let config: Config = toml::from_str(
    r#"
        [core]
        repo = "~/.dotfiles"
      "#,
  )
  .expect("should parse without crypto section");

  assert_eq!(config.crypto.mode, CryptoMode::Symmetric);
  assert!(config.crypto.gpg_key.is_empty());
}

#[test]
fn crypto_asymmetric_mode_parses_correctly() {
  let config: Config = toml::from_str(
    r#"
        [core]
        repo = "~/.dotfiles"

        [crypto]
        mode = "asymmetric"
        gpg_key = "william@email.com"
      "#,
  )
  .expect("should parse asymmetric crypto config");

  assert_eq!(config.crypto.mode, CryptoMode::Asymmetric);
  assert_eq!(config.crypto.gpg_key, "william@email.com");
}

#[test]
fn color_defaults_to_true() {
  let config: Config = toml::from_str(
    r#"
        [core]
        repo = "~/.dotfiles"
      "#,
  )
  .expect("should parse without color key");

  assert!(config.core.color);
}

#[test]
fn search_threshold_defaults_to_15() {
  let config: Config = toml::from_str(
    r#"
        [core]
        repo = "~/.dotfiles"
      "#,
  )
  .expect("should parse without search_threshold");

  assert_eq!(config.core.search_threshold, 15);
}

#[test]
fn existing_config_without_crypto_section_still_parses() {
  let config: Config = toml::from_str(
    r#"
        [core]
        repo = "~/.dotfiles"

        [git]
        available = true
        auto_commit = true
      "#,
  )
  .expect("old config without crypto section should parse");

  assert_eq!(config.crypto.mode, CryptoMode::Symmetric);
}
