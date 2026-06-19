use crate::OutputFormat;
use tabled::builder::Builder;
use tabled::settings::Style;

/// Renders output as a table or JSON string.
pub(crate) fn render<H, R>(headers: &[H], rows: &[R], format: &OutputFormat) -> String
where
    H: AsRef<str>,
    R: AsRef<[String]>,
{
    match format {
        OutputFormat::Table => {
            let mut builder = Builder::default();
            builder.push_record(headers.iter().map(|h| h.as_ref()).collect::<Vec<_>>());
            for row in rows {
                builder.push_record(row.as_ref().iter().map(|s| s.as_str()).collect::<Vec<_>>());
            }
            builder.build().with(Style::rounded()).to_string()
        }
        OutputFormat::Json => {
            if rows.is_empty() {
                return "[]".into();
            }
            let mut result = String::from("[\n");
            for (i, row) in rows.iter().enumerate() {
                result.push_str("  {");
                let fields: Vec<String> = headers
                    .iter()
                    .zip(row.as_ref().iter())
                    .map(|(h, v)| format!(r#""{}": "{}""#, h.as_ref(), v))
                    .collect();
                result.push_str(&fields.join(", "));
                result.push('}');
                if i < rows.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push(']');
            result
        }
    }
}

/// Render a single key-value record as a vertical table or JSON object.
#[allow(dead_code)]
pub(crate) fn render_kv(entries: &[(&str, &str)], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Table => {
            let mut builder = Builder::default();
            for (k, v) in entries {
                builder.push_record([*k, *v]);
            }
            builder.build().with(Style::rounded()).to_string()
        }
        OutputFormat::Json => {
            let mut result = String::from("{");
            let fields: Vec<String> = entries
                .iter()
                .map(|(k, v)| format!(r#""{}": "{}""#, k, v))
                .collect();
            result.push_str(&fields.join(", "));
            result.push('}');
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_table() {
        let headers = ["ID", "Name"];
        let rows: Vec<[String; 2]> = vec![["1".into(), "Test".into()]];
        let output = render(&headers, &rows, &OutputFormat::Table);
        assert!(output.contains("ID"));
        assert!(output.contains("Name"));
        assert!(output.contains("1"));
        assert!(output.contains("Test"));
    }

    #[test]
    fn test_render_json() {
        let headers = ["ID", "Name"];
        let rows: Vec<[String; 2]> = vec![["1".into(), "Test".into()]];
        let output = render(&headers, &rows, &OutputFormat::Json);
        assert!(output.contains("\"ID\""));
        assert!(output.contains("\"Name\""));
    }

    #[test]
    fn test_render_empty_json() {
        let headers: [&str; 0] = [];
        let rows: Vec<[String; 0]> = vec![];
        let output = render(&headers, &rows, &OutputFormat::Json);
        assert_eq!(output, "[]");
    }

    #[test]
    fn test_render_kv_json() {
        let entries = [("key", "value")];
        let output = render_kv(&entries, &OutputFormat::Json);
        assert!(output.contains("\"key\""));
        assert!(output.contains("\"value\""));
    }

    #[test]
    fn test_render_kv_table() {
        let entries = [("Setting", "Value"), ("Locale", "en")];
        let output = render_kv(&entries, &OutputFormat::Table);
        assert!(output.contains("Setting"));
        assert!(output.contains("Value"));
        assert!(output.contains("Locale"));
    }
}
