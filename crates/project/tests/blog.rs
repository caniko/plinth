#![cfg(feature = "brick-blog")]

use std::fs;
use std::path::Path;

use plinth_project::{RenderOptions, Severity, load_project_site, render_static, validate_site};

const SITE: &str = r#"
[site]
title = "Example"
description = "Example site"
canonical_domain = "example.com"
favicon = "/favicon.svg"

[[pages]]
slug = "index"
title = "Example"
"#;

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("file has a parent")).unwrap();
    fs::write(path, contents).unwrap();
}

fn post(title: &str, date: &str, extra: &str, body: &str) -> String {
    format!("---\ntitle: \"{title}\"\n{extra}date: \"{date}\"\n---\n{body}\n")
}

fn blog_site(dir: &Path) -> plinth_project::ProjectSite {
    write(
        &dir.join("plinth-project.toml"),
        &format!("{SITE}\n[blog]\ntitle = \"Notes\"\n"),
    );
    write(
        &dir.join("posts/newer-post.md"),
        &post(
            "Newer post",
            "2026-06-01",
            "description: \"Newest entry\"\ntags: [\"rust\", \"release\"]\nauthor: \"Ada\"\n",
            "## Section\n\nBody with `code`.",
        ),
    );
    write(
        &dir.join("posts/older-post.md"),
        "---\ntitle: Older post & notes\ndescription: Earlier entry\ndate: 2026-01-15\ntags: [rust]\n---\nEarlier body.\n",
    );
    write(
        &dir.join("posts/draft-post.md"),
        &post(
            "Draft post",
            "2026-03-01",
            "published: false\n",
            "Unpublished body.",
        ),
    );
    load_project_site(dir.join("plinth-project.toml")).unwrap()
}

#[test]
fn blog_builds_ordered_index_posts_feed_and_sitemap() {
    let dir = tempfile::tempdir().unwrap();
    let site = blog_site(dir.path());
    let blog = site.blog.as_ref().expect("blog is configured");
    assert_eq!(blog.title, "Notes");
    assert_eq!(
        blog.posts
            .iter()
            .map(|post| post.slug.as_str())
            .collect::<Vec<_>>(),
        ["newer-post", "older-post"]
    );
    assert!(validate_site(&site).diagnostics.is_empty());

    let out = dir.path().join("out");
    render_static(&site, &RenderOptions::new(&out)).unwrap();

    let index = fs::read_to_string(out.join("blog/index.html")).unwrap();
    let newer = index.find("Newer post").expect("newest post in index");
    let older = index.find("Older post").expect("older post in index");
    assert!(newer < older, "index must list newest posts first");
    assert!(!index.contains("Draft post"));
    assert!(index.contains("Older post &amp; notes"));
    assert!(index.contains("<li>rust</li>"));
    assert!(index.contains("href=\"/blog/newer-post/\""));
    assert!(
        index.contains("<a href=\"/blog/\">Blog</a>"),
        "published posts add a Blog nav link"
    );

    let newer_page = fs::read_to_string(out.join("blog/newer-post/index.html")).unwrap();
    assert!(newer_page.contains("<h1>Newer post</h1>"));
    assert!(newer_page.contains("<h2>Section</h2>"));
    assert!(newer_page.contains("<code>code</code>"));
    assert!(
        newer_page.contains("class=\"top-nav\""),
        "site chrome is kept"
    );
    assert!(
        newer_page.contains("<link rel=\"icon\" href=\"/favicon.svg\""),
        "blog pages share the site document head"
    );
    assert!(
        newer_page
            .contains("<link rel=\"canonical\" href=\"https://example.com/blog/newer-post/\">")
    );
    assert!(!out.join("blog/draft-post").exists());

    let feed = fs::read_to_string(out.join("blog/feed.xml")).unwrap();
    assert!(feed.contains("<link>https://example.com/blog/newer-post/</link>"));
    assert!(feed.contains("Older post &amp; notes"));
    assert!(feed.contains("<pubDate>"));
    assert!(!feed.contains("Draft post"));

    let sitemap = fs::read_to_string(out.join("sitemap.xml")).unwrap();
    assert!(sitemap.contains("<loc>https://example.com/blog/</loc>"));
    assert!(sitemap.contains("<loc>https://example.com/blog/newer-post/</loc>"));
    assert!(!sitemap.contains("draft-post"));

    let llms = fs::read_to_string(out.join("llms.txt")).unwrap();
    assert!(llms.contains("## Notes"));
    assert!(llms.contains("https://example.com/blog/newer-post/"));
    assert!(!llms.contains("Draft post"));

    let css = fs::read_to_string(out.join("style.css")).unwrap();
    assert!(css.contains(".blog-index"));
}

#[test]
fn site_without_blog_renders_no_blog_output() {
    let dir = tempfile::tempdir().unwrap();
    write(&dir.path().join("plinth-project.toml"), SITE);
    let site = load_project_site(dir.path().join("plinth-project.toml")).unwrap();
    assert!(site.blog.is_none());

    let out = dir.path().join("out");
    render_static(&site, &RenderOptions::new(&out)).unwrap();

    assert!(!out.join("blog").exists());
    let index = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(!index.contains("href=\"/blog/\""));
    let sitemap = fs::read_to_string(out.join("sitemap.xml")).unwrap();
    assert!(!sitemap.contains("/blog/"));
    let llms = fs::read_to_string(out.join("llms.txt")).unwrap();
    assert!(!llms.contains("## Blog"));
    let css = fs::read_to_string(out.join("style.css")).unwrap();
    assert!(!css.contains(".blog-index"));
}

#[test]
fn blog_without_canonical_domain_skips_feed() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("plinth-project.toml"),
        "[site]\ntitle = \"Example\"\ndescription = \"Example site\"\n\n[[pages]]\nslug = \"index\"\ntitle = \"Example\"\n\n[blog]\n",
    );
    write(
        &dir.path().join("posts/only-post.md"),
        &post("Only post", "2026-02-02", "", "Body."),
    );
    let site = load_project_site(dir.path().join("plinth-project.toml")).unwrap();

    let out = dir.path().join("out");
    render_static(&site, &RenderOptions::new(&out)).unwrap();

    assert!(out.join("blog/index.html").exists());
    assert!(out.join("blog/only-post/index.html").exists());
    assert!(
        !out.join("blog/feed.xml").exists(),
        "a relative feed is not published"
    );
}

#[test]
fn post_without_title_fails_diagnostics_and_render() {
    let dir = tempfile::tempdir().unwrap();
    write(
        &dir.path().join("plinth-project.toml"),
        &format!("{SITE}\n[blog]\n"),
    );
    write(
        &dir.path().join("posts/missing-title.md"),
        "---\ndescription: \"No title\"\ndate: \"2026-05-01\"\n---\nBody\n",
    );
    let site = load_project_site(dir.path().join("plinth-project.toml")).unwrap();

    let report = validate_site(&site);
    assert!(report.has_errors());
    let error = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "blog.missing_field")
        .expect("missing title is reported");
    assert_eq!(error.severity, Severity::Error);
    assert!(
        error.message.contains("missing-title.md"),
        "{}",
        error.message
    );
    assert!(error.message.contains("title"), "{}", error.message);
    assert!(render_static(&site, &RenderOptions::new(dir.path().join("out"))).is_err());
}
