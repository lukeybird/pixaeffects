//! Offline guide data shared by each editor's Pixa help window.
use serde::Deserialize;
#[derive(Deserialize)]
pub struct Chapter {
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub tool_id: Option<String>,
}
#[derive(Deserialize)]
pub struct Guide {
    pub schema_version: u32,
    pub product: String,
    pub name: String,
    pub title: String,
    pub scope: String,
    pub chapters: Vec<Chapter>,
    pub tour: Vec<Chapter>,
    #[serde(default)]
    pub tools: Vec<Chapter>,
}
pub fn read(bytes: &str, expected_product: &str) -> Result<Guide, String> {
    if bytes.len() > 262_144 {
        return Err("Manual is too large.".into());
    }
    let guide: Guide =
        serde_json::from_str(bytes).map_err(|_| "Could not read the offline manual.")?;
    if guide.schema_version != 1
        || guide.product != expected_product
        || guide.chapters.is_empty()
        || guide.tour.is_empty()
    {
        return Err("Manual does not match this product.".into());
    }
    Ok(guide)
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    #[test]
    fn wrong_product_and_empty_tour_are_rejected() {
        let doc = r#"{"schema_version":1,"product":"pixagrid","name":"PixaGrid","title":"Manual","scope":"Starter","chapters":[{"title":"Save","body":"Save your work"}],"tour":[{"title":"Save","body":"Save your work"}]}"#;
        assert!(read(doc, "pixagrid").is_ok());
        assert!(read(doc, "pixaword").is_err());
        let mut value: serde_json::Value = serde_json::from_str(doc).unwrap();
        value["tour"] = serde_json::json!([]);
        assert!(read(&value.to_string(), "pixagrid").is_err());
    }
}
