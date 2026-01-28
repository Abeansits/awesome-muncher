use color_eyre::Result;
use regex::Regex;

use crate::models::Project;

/// Parsed project from markdown (before database insertion)
#[derive(Debug, Clone)]
pub struct ParsedProject {
    pub name: String,
    pub url: String,
    pub description: String,
    pub category: String,
}

/// Parse an awesome-list markdown file into projects
pub fn parse_awesome_list(markdown: &str, source_id: i64) -> Result<Vec<Project>> {
    let parsed = parse_markdown(markdown)?;

    let projects = parsed
        .into_iter()
        .map(|p| Project::new(source_id, p.name, p.url, p.description, p.category))
        .collect();

    Ok(projects)
}

/// Parse markdown content into ParsedProject structs
fn parse_markdown(markdown: &str) -> Result<Vec<ParsedProject>> {
    let mut projects = Vec::new();
    let mut current_category = String::new();

    // Regex for markdown links: [name](url) - description
    // or [name](url): description
    // or just [name](url)
    let link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)(?:\s*[-:]\s*(.+))?").unwrap();

    // Regex for headers: ## Category Name or ### Category Name
    let header_re = Regex::new(r"^#{2,4}\s+(.+)$").unwrap();

    for line in markdown.lines() {
        let line = line.trim();

        // Check for category headers
        if let Some(caps) = header_re.captures(line) {
            let header = caps.get(1).unwrap().as_str().trim();
            // Skip certain headers that aren't categories
            if !is_meta_header(header) {
                current_category = clean_category_name(header);
            }
            continue;
        }

        // Skip lines that don't look like list items
        if !line.starts_with('-') && !line.starts_with('*') {
            continue;
        }

        // Try to parse as a project link
        if let Some(caps) = link_re.captures(line) {
            let name = caps.get(1).unwrap().as_str().trim().to_string();
            let url = caps.get(2).unwrap().as_str().trim().to_string();
            let description = caps
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            // Skip non-project links (e.g., category anchors, badges)
            if is_valid_project_url(&url) && !name.is_empty() {
                projects.push(ParsedProject {
                    name,
                    url,
                    description,
                    category: current_category.clone(),
                });
            }
        }
    }

    Ok(projects)
}

/// Check if a header is a meta/navigation header (not a real category)
fn is_meta_header(header: &str) -> bool {
    let lower = header.to_lowercase();
    let meta_patterns = [
        "contents",
        "table of contents",
        "toc",
        "contributing",
        "license",
        "resources",
        "related",
        "see also",
        "credits",
        "acknowledgments",
        "links",
        "summary",
        "about",
        "awesome",
    ];

    meta_patterns.iter().any(|p| lower.contains(p))
}

/// Clean up category name (remove anchors, emojis markers, etc.)
fn clean_category_name(name: &str) -> String {
    let mut clean = name.to_string();

    // Remove HTML anchor tags like <a name="..."></a> or <a id="...">
    let html_anchor_re = Regex::new(r#"<a[^>]*>|</a>"#).unwrap();
    clean = html_anchor_re.replace_all(&clean, "").to_string();

    // Remove anchor links like [](#section)
    let anchor_re = Regex::new(r"\[?\]?\(#[^)]+\)").unwrap();
    clean = anchor_re.replace_all(&clean, "").to_string();

    // Remove badge images ![](...)
    let badge_re = Regex::new(r"!\[[^\]]*\]\([^)]+\)").unwrap();
    clean = badge_re.replace_all(&clean, "").to_string();

    // Remove leftover brackets
    clean = clean.replace('[', "").replace(']', "");

    // Trim and clean up multiple spaces
    let multi_space_re = Regex::new(r"\s+").unwrap();
    clean = multi_space_re.replace_all(&clean, " ").to_string();

    clean.trim().to_string()
}

/// Check if a URL looks like a valid project URL
fn is_valid_project_url(url: &str) -> bool {
    // Must be http/https
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return false;
    }

    // Skip anchor links
    if url.starts_with('#') {
        return false;
    }

    // Skip badge/shield URLs
    if url.contains("shields.io") || url.contains("badge") || url.contains("img.shields") {
        return false;
    }

    // Skip images
    if url.ends_with(".png")
        || url.ends_with(".jpg")
        || url.ends_with(".svg")
        || url.ends_with(".gif")
    {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_list() {
        let markdown = r#"
## TUI Frameworks

- [ratatui](https://github.com/ratatui-org/ratatui) - A Rust library for building terminal user interfaces.
- [cursive](https://github.com/gyscos/cursive) - A TUI library for Rust using ncurses.

## Git Tools

- [gitui](https://github.com/extrawurst/gitui) - Blazing fast terminal-ui for git.
        "#;

        let projects = parse_markdown(markdown).unwrap();

        assert_eq!(projects.len(), 3);

        assert_eq!(projects[0].name, "ratatui");
        assert_eq!(projects[0].category, "TUI Frameworks");
        assert!(projects[0].description.contains("terminal user interfaces"));

        assert_eq!(projects[2].name, "gitui");
        assert_eq!(projects[2].category, "Git Tools");
    }

    #[test]
    fn test_parse_various_formats() {
        let markdown = r#"
## Category

- [project1](https://github.com/a/b) - Description with dash
- [project2](https://github.com/c/d): Description with colon
* [project3](https://github.com/e/f) - Description with star
- [project4](https://github.com/g/h)
        "#;

        let projects = parse_markdown(markdown).unwrap();

        assert_eq!(projects.len(), 4);
        assert_eq!(projects[0].description, "Description with dash");
        assert_eq!(projects[1].description, "Description with colon");
        assert_eq!(projects[2].description, "Description with star");
        assert_eq!(projects[3].description, "");
    }

    #[test]
    fn test_skip_meta_headers() {
        let markdown = r#"
## Contents

- [link](#anchor)

## Real Category

- [project](https://github.com/a/b) - A real project

## Contributing

- [guidelines](https://github.com/contrib/guide) - Contributing guidelines
        "#;

        let projects = parse_markdown(markdown).unwrap();

        // Contents section is skipped, Contributing is skipped
        // Only Real Category project should have that category
        assert!(projects.iter().any(|p| p.category == "Real Category"));
        assert!(projects.iter().any(|p| p.name == "project"));
    }

    #[test]
    fn test_skip_invalid_urls() {
        let markdown = r#"
## Category

- [valid](https://github.com/a/b) - Valid project
- [badge](https://shields.io/badge) - Should skip
- [image](https://example.com/pic.png) - Should skip
- [anchor](#section) - Should skip
        "#;

        let projects = parse_markdown(markdown).unwrap();

        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "valid");
    }

    #[test]
    fn test_github_url_parsing() {
        let projects =
            parse_awesome_list("## Test\n- [test](https://github.com/owner/repo) - desc", 1)
                .unwrap();

        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].github_owner, Some("owner".to_string()));
        assert_eq!(projects[0].github_repo, Some("repo".to_string()));
    }

    #[test]
    fn test_clean_html_anchors_from_category() {
        let markdown = r#"
## <a name="games"></a>Games

- [game1](https://github.com/a/b) - A game

## AI / ChatGPT <a name="ai"></a>

- [ai1](https://github.com/c/d) - AI tool
        "#;

        let projects = parse_markdown(markdown).unwrap();

        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].category, "Games");
        assert_eq!(projects[1].category, "AI / ChatGPT");
    }
}
