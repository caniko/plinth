use chrono::{DateTime, Utc};
use pgvector::Vector;
use plinth_shared::{
    BlogListItem, BlogPost, ContentFormat, PortfolioItem, SeriesEntry, SeriesListItem, SiteContent,
    Tag, TodoItem, TodoListItem,
};
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

fn content_format(value: String) -> Result<ContentFormat, Error> {
    match value.as_str() {
        "markdown" => Ok(ContentFormat::Markdown),
        "typst" => Ok(ContentFormat::Typst),
        other => Err(decode_error(format!("unknown content format '{other}'"))),
    }
}

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

pub fn series_entry(row: PgRow) -> Result<SeriesEntry, Error> {
    Ok(SeriesEntry {
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        position: as_u32(row.try_get("position")?, "position")?,
    })
}

pub fn series_list_item(row: PgRow) -> Result<SeriesListItem, Error> {
    Ok(SeriesListItem {
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        post_count: as_u32(row.try_get("post_count")?, "post_count")?,
        total_reading_time: as_u32(row.try_get("total_reading_time")?, "total_reading_time")?,
        latest_published_at: row.try_get::<Option<DateTime<Utc>>, _>("latest_published_at")?,
    })
}

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
        image_url: row.try_get("image_url")?,
        date: row.try_get("date")?,
        featured: row.try_get("featured")?,
        order: row.try_get("order")?,
    })
}

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
