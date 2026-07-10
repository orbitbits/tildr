pub fn authors_format(authors: &str, indentation: &str) -> String {
  authors
    .replace("://", "\x00PROTO\x00") // protect the ://
    .split(':')
    .map(|a| {
      let restored = a.trim().replace("\x00PROTO\x00", "://");
      format!("{}{}", indentation, restored)
    })
    .collect::<Vec<_>>()
    .join("\n")
}
