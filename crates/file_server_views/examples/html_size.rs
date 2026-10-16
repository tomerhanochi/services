use file_server_core::prelude::*;
use file_server_views::prelude::*;
use htmlb::IntoHtml;
use std::{fs, path::PathBuf, time::SystemTime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| "/tmp/file-server-html".into());
    fs::create_dir_all(&output)?;
    let location = Location::new(MountName::new("documents")?, RelPath::parse("1000")?);
    for (role, editable) in [("editor", true), ("viewer", false)] {
        let entries = (0..1000)
            .map(|i| {
                Ok(Entry {
                    name: EntryName::new(format!("entry-{i:04}.txt"))?,
                    metadata: Metadata {
                        kind: EntryKind::File,
                        size: 20,
                        modified: Some(SystemTime::UNIX_EPOCH),
                    },
                })
            })
            .collect::<Result<Vec<_>, InvalidName>>()?;
        let page = Page {
            stylesheet: "/assets/app.css",
            user: Some("alice"),
            logout: false,
        };
        let html = directory(
            page,
            &location,
            &entries,
            Capabilities {
                list: true,
                download: true,
                rename: editable,
                moves: editable,
            },
            None,
        )
        .to_html();
        let svg_bytes: usize = html
            .match_indices("<svg")
            .map(|(start, _)| html[start..].find("</svg>").unwrap() + "</svg>".len())
            .sum();
        println!(
            "{role}: {} HTML bytes, {} SVG elements, {svg_bytes} SVG bytes ({:.1}%)",
            html.len(),
            html.matches("<svg").count(),
            100.0 * svg_bytes as f64 / html.len() as f64
        );
        fs::write(output.join(format!("{role}.html")), html)?;
    }
    println!("HTML saved to {}", output.display());
    Ok(())
}
