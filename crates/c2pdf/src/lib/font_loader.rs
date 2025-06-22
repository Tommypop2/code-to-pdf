//! Functions for loading fonts from the system fonts, a path, or using the bundled `Helvetica` font

#[cfg(feature = "font-loading")]
use font_kit::{family_name::FamilyName, properties::Properties, source::SystemSource};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

/// Returns an atomicically reference counted reference to the underlying font data of a system font
///
/// This function always returns an error if the `font-loading` feature is disabled
fn load_font_system(name: String) -> Result<Arc<Vec<u8>>, Box<dyn std::error::Error>> {
  #[cfg(not(feature = "font-loading"))]
  {
    Err("font-loading feature is disabled".into())
  }
  #[cfg(feature = "font-loading")]
  {
    let handle =
      SystemSource::new().select_best_match(&[FamilyName::Title(name)], &Properties::new())?;
    let font = handle.load()?;
    let data = if let Some(d) = font.copy_font_data() {
      d
    } else {
      return Err("Unable to load font data".into());
    };
    Ok(data)
  }
}
/// Load font bytes from a specific path
fn load_font_path(path: String) -> Result<Arc<Vec<u8>>, Box<dyn std::error::Error>> {
  let bytes = fs::read(path)?;
  let arc = Arc::new(bytes);
  Ok(arc)
}
/// Loads bytes from bundled font
fn bundled_font_bytes() -> Arc<Vec<u8>> {
  let bytes = include_bytes!("./fonts/Helvetica.ttf").to_vec();
  Arc::new(bytes)
}
fn is_path(s: &str) -> bool {
  PathBuf::from(s).extension().is_some() || s.len() > 31 || s.starts_with('.')
}
/// Loads a given font - falling back to the bundled font if loading from the system, or from the given path fails
pub fn load_font(name_or_path: Option<String>) -> (Arc<Vec<u8>>, bool) {
  let data = name_or_path.and_then(|name_or_path| {
    if is_path(&name_or_path) {
      load_font_path(name_or_path)
    } else {
      load_font_system(name_or_path)
    }
    .ok()
  });

  match data {
    Some(d) => (d, false),
    None => (bundled_font_bytes(), true),
  }
}
