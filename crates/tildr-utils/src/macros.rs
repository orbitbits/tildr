// // (c) 2026 OrbitBits. All rights reserved.
// //! Path construction macros.
// //!
// //! Provides the [`userprofile!`] macro that resolves the current user's home
// //! directory from `$HOME` and appends one or more path components to it.
//
// /// Builds a [`std::path::PathBuf`] rooted at the current user's home directory.
// ///
// /// Accepts one or more string expressions that are joined with the
// /// platform-native separator (`/` on Unix) and appended to the
// /// home directory path.
// ///
// /// # Panics
// ///
// /// Panics if the `HOME` environment variable is not set.
// ///
// /// # Examples
// ///
// /// ```rust
// /// use tildr_utils::userprofile;
// ///
// /// // Resolves to e.g. "/home/william/.config.json"
// /// let path = userprofile!(".config.json");
// ///
// /// // Resolves to e.g. "/home/william/app1/config/settings.json"
// /// let nested = userprofile!("app1", "config", "settings.json");
// /// ```
// #[macro_export]
// macro_rules! userprofile {
//   ($($part:expr),* $(,)?) => {{
//     let mut path = dirs::home_dir()
//       .expect("Could not determine home directory");
//
//     $(
//       path.push($part);
//     )*
//
//     path
//   }};
// }
