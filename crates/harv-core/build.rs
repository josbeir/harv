use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir_str = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir_str);
    let locales_dir = manifest_dir.join("locales");

    let mut langs: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(&locales_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && let Some(name) = entry.file_name().to_str()
                && entry.path().join("main.ftl").exists()
            {
                langs.push(name.to_string());
            }
        }
    }

    langs.sort();

    assert!(!langs.is_empty(), "no locale directories found in locales/");
    assert!(
        langs.contains(&"en".to_string()),
        "locales/en/main.ftl is required as the base locale"
    );

    let out_dir_str = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_str);

    let lang_list: Vec<String> = langs.iter().map(|l| format!("\"{l}\"")).collect();
    fs::write(
        out_dir.join("supported_langs.rs"),
        format!("&[{}]", lang_list.join(", ")),
    )
    .unwrap();

    let mut match_block = String::from("match lang_code {\n");
    for lang in &langs {
        if lang == "en" {
            continue;
        }
        let path = format!("/locales/{lang}/main.ftl");
        match_block.push_str(&format!(
            r#"    {lang:?} => {{
        let extra = parse_ftl(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), {path:?})));
        for (k, v) in extra {{
            messages.insert(k, v);
        }}
    }}
"#,
        ));
    }
    match_block.push_str("    _ => {}\n}\n");
    fs::write(out_dir.join("lang_load.rs"), &match_block).unwrap();

    println!("cargo:rerun-if-changed=locales/");
}
