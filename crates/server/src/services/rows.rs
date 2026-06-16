#[cfg(feature = "brick-blog")]
use chrono::{DateTime, Utc};
#[cfg(feature = "brick-blog")]
use pgvector::Vector;
#[cfg(feature = "brick-portfolio")]
use plinth_shared::PortfolioItem;
#[cfg(feature = "brick-activity")]
use plinth_shared::{ActivityItem, ActivityListItem};
#[cfg(feature = "brick-blog")]
use plinth_shared::{BlogListItem, BlogPost, ContentFormat, SeriesEntry, SeriesListItem};
use plinth_shared::{SiteContent, Tag};
#[cfg(feature = "brick-todo")]
use plinth_shared::{TodoItem, TodoListItem};
use sqlx::{Error, Row, postgres::PgRow};

fn id(table: &str, value: i64) -> Option<String> {
    Some(format!("{table}:{value}"))
}

fn decode_error(message: impl Into<String>) -> Error {
    Error::Decode(Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message.into(),
    )))
}

fn as_u32(value: i32, column: &str) -> Result<u32, Error> {
    value
        .try_into()
        .map_err(|_| decode_error(format!("{column} contained negative value {value}")))
}

#[cfg(feature = "brick-blog")]
fn content_format(value: String) -> Result<ContentFormat, Error> {
    match value.as_str() {
        "markdown" => Ok(ContentFormat::Markdown),
        "typst" => Ok(ContentFormat::Typst),
        other => Err(decode_error(format!("unknown content format '{other}'"))),
    }
}

#[cfg(feature = "brick-activity")]
fn parse_activity_enum<T>(value: String) -> Result<T, Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value.parse::<T>().map_err(|e| decode_error(e.to_string()))
}

/// Decode a single blog post row from the database.
#[cfg(feature = "brick-blog")]
pub fn blog_post(row: PgRow) -> Result<BlogPost, Error> {
    let embedding: Option<Vector> = row.try_get("embedding")?;
    Ok(BlogPost {
        id: id("blog_posts", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        content: row.try_get("content")?,
        html_content: row.try_get("html_content")?,
        published_at: row.try_get("published_at")?,
        updated_at: row.try_get("updated_at")?,
        author: row.try_get("author")?,
        tags: row.try_get("tags")?,
        featured: row.try_get("featured")?,
        published: row.try_get("published")?,
        reading_time_minutes: as_u32(row.try_get("reading_time_minutes")?, "reading_time_minutes")?,
        embedding: embedding.map(|v| v.to_vec()),
        content_format: content_format(row.try_get("content_format")?)?,
        source: row.try_get("source")?,
        content_hash: row.try_get("content_hash")?,
        series_slug: row.try_get("series_slug")?,
        series_title: row.try_get("series_title")?,
        series_position: row
            .try_get::<Option<i32>, _>("series_position")?
            .map(|v| as_u32(v, "series_position"))
            .transpose()?,
    })
}

/// Decode a blog post list-item row (lighter than full `blog_post`).
#[cfg(feature = "brick-blog")]
pub fn blog_list_item(row: PgRow) -> Result<BlogListItem, Error> {
    Ok(BlogListItem {
        id: id("blog_posts", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        published_at: row.try_get("published_at")?,
        author: row.try_get("author")?,
        tags: row.try_get("tags")?,
        featured: row.try_get("featured")?,
        reading_time_minutes: as_u32(row.try_get("reading_time_minutes")?, "reading_time_minutes")?,
        series_slug: row.try_get("series_slug")?,
        series_title: row.try_get("series_title")?,
        series_position: row
            .try_get::<Option<i32>, _>("series_position")?
            .map(|v| as_u32(v, "series_position"))
            .transpose()?,
    })
}

/// Decode a series entry row (slug, title, position) from the database.
#[cfg(feature = "brick-blog")]
pub fn series_entry(row: PgRow) -> Result<SeriesEntry, Error> {
    Ok(SeriesEntry {
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        position: as_u32(row.try_get("position")?, "position")?,
    })
}

/// Decode a series list-item row with aggregate counts.
#[cfg(feature = "brick-blog")]
pub fn series_list_item(row: PgRow) -> Result<SeriesListItem, Error> {
    Ok(SeriesListItem {
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        post_count: as_u32(row.try_get("post_count")?, "post_count")?,
        total_reading_time: as_u32(row.try_get("total_reading_time")?, "total_reading_time")?,
        latest_published_at: row.try_get::<Option<DateTime<Utc>>, _>("latest_published_at")?,
    })
}

/// Decode a single todo item row from the database.
#[cfg(feature = "brick-todo")]
pub fn todo_item(row: PgRow) -> Result<TodoItem, Error> {
    Ok(TodoItem {
        id: id("todos", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        content: row.try_get("content")?,
        html_content: row.try_get("html_content")?,
        tags: row.try_get("tags")?,
        completed: row.try_get("completed")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        order: row.try_get("order")?,
    })
}

/// Decode a todo list-item row (lightweight, no content body).
#[cfg(feature = "brick-todo")]
pub fn todo_list_item(row: PgRow) -> Result<TodoListItem, Error> {
    Ok(TodoListItem {
        id: id("todos", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        tags: row.try_get("tags")?,
        completed: row.try_get("completed")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        order: row.try_get("order")?,
    })
}

/// Decode a portfolio item row from the database.
#[cfg(feature = "brick-portfolio")]
pub fn portfolio_item(row: PgRow) -> Result<PortfolioItem, Error> {
    Ok(PortfolioItem {
        id: id("portfolio_items", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        content: row.try_get("content")?,
        html_content: row.try_get("html_content")?,
        tech_stack: row.try_get("tech_stack")?,
        link: row.try_get("link")?,
        demo: row.try_get("demo")?,
        project_url: row.try_get("project_url")?,
        links: row
            .try_get::<sqlx::types::Json<Vec<plinth_shared::ExternalLink>>, _>("links")?
            .0,
        image_url: row.try_get("image_url")?,
        date: row.try_get("date")?,
        featured: row.try_get("featured")?,
        order: row.try_get("order")?,
    })
}

/// Decode a single activity item row from the database.
#[cfg(feature = "brick-activity")]
pub fn activity_item(row: PgRow) -> Result<ActivityItem, Error> {
    Ok(ActivityItem {
        id: row.try_get::<i64, _>("id")?,
        forge: parse_activity_enum(row.try_get("forge")?)?,
        repo_owner: row.try_get("repo_owner")?,
        repo_name: row.try_get("repo_name")?,
        kind: parse_activity_enum(row.try_get("kind")?)?,
        number: row.try_get("number")?,
        url: row.try_get("url")?,
        title: row.try_get("title")?,
        body: row.try_get("body")?,
        state: parse_activity_enum(row.try_get("state")?)?,
        created_at: row.try_get("created_at")?,
        closed_at: row.try_get("closed_at")?,
        merged_at: row.try_get("merged_at")?,
        impact: row.try_get("impact")?,
        additions: row.try_get("additions")?,
        deletions: row.try_get("deletions")?,
        comments_count: row.try_get("comments_count")?,
        labels: row.try_get("labels")?,
        repo_stars: row.try_get("repo_stars")?,
        fetched_at: row.try_get("fetched_at")?,
        featured: row.try_get("featured")?,
        published: row.try_get("published")?,
        content_hash: row.try_get("content_hash")?,
    })
}

/// Decode an activity list-item row (lighter than full `activity_item`).
#[cfg(feature = "brick-activity")]
pub fn activity_list_item(row: PgRow) -> Result<ActivityListItem, Error> {
    Ok(ActivityListItem {
        id: row.try_get::<i64, _>("id")?,
        forge: parse_activity_enum(row.try_get("forge")?)?,
        repo_owner: row.try_get("repo_owner")?,
        repo_name: row.try_get("repo_name")?,
        kind: parse_activity_enum(row.try_get("kind")?)?,
        number: row.try_get("number")?,
        url: row.try_get("url")?,
        title: row.try_get("title")?,
        state: parse_activity_enum(row.try_get("state")?)?,
        created_at: row.try_get("created_at")?,
        closed_at: row.try_get("closed_at")?,
        merged_at: row.try_get("merged_at")?,
        impact: row.try_get("impact")?,
        labels: row.try_get("labels")?,
        featured: row.try_get("featured")?,
        score: row.try_get::<f64, _>("score")?,
    })
}

/// Decode a site content row from the database.
pub fn site_content(row: PgRow) -> Result<SiteContent, Error> {
    Ok(SiteContent {
        id: id("site_content", row.try_get("id")?),
        key: row.try_get("key")?,
        title: row.try_get("title")?,
        content: row.try_get("content")?,
        html_content: row.try_get("html_content")?,
        updated_at: row.try_get("updated_at")?,
    })
}

/// Decode a tag row from the database.
pub fn tag(row: PgRow) -> Result<Tag, Error> {
    Ok(Tag {
        id: id("tags", row.try_get("id")?),
        name: row.try_get("name")?,
        slug: row.try_get("slug")?,
        #[cfg(feature = "brick-blog")]
        post_count: as_u32(row.try_get("post_count")?, "post_count")?,
        #[cfg(feature = "brick-todo")]
        todo_count: as_u32(row.try_get("todo_count")?, "todo_count")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_formats_table_and_value() {
        assert_eq!(id("blog_posts", 3), Some("blog_posts:3".to_string()));
    }

    #[test]
    fn as_u32_accepts_nonnegative() {
        assert_eq!(as_u32(0, "c").unwrap(), 0);
        assert_eq!(as_u32(5, "c").unwrap(), 5);
    }

    #[test]
    fn as_u32_rejects_negative_and_names_the_column() {
        let err = as_u32(-1, "reading_time_minutes").unwrap_err();
        assert!(matches!(err, Error::Decode(_)));
        assert!(err.to_string().contains("reading_time_minutes"));
    }

    #[cfg(feature = "brick-blog")]
    #[test]
    fn content_format_maps_known_and_rejects_unknown() {
        assert!(matches!(
            content_format("markdown".to_string()).unwrap(),
            ContentFormat::Markdown
        ));
        assert!(matches!(
            content_format("typst".to_string()).unwrap(),
            ContentFormat::Typst
        ));
        assert!(content_format("bogus".to_string()).is_err());
    }
}
