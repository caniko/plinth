use std::sync::{Arc, Mutex};

use crate::PlinthDb;
use fastembed::{EmbeddingModel, TextEmbedding};
use kameo::Actor;
use kameo::message::{Context, Message};
use pgvector::Vector;

use plinth_shared::{BlogPost, ContentFormat};
use sqlx::Row;

pub const EMBEDDING_DIM: usize = 384;
const OPINION_EVOLUTION_CANDIDATE_LIMIT: i64 = 1_000;

/// Vector search actor that handles semantic search queries.
///
/// The fastembed model is held behind an `Arc<Mutex<…>>` so embedding inference
/// (synchronous, CPU-heavy) can be moved off the async runtime via
/// `spawn_blocking` — otherwise it would block the single-threaded server
/// runtime and stall every other in-flight request.
#[derive(Actor)]
pub struct VectorSearch {
    db: PlinthDb,
    embedding_model: Arc<Mutex<TextEmbedding>>,
    vector_truncation: usize,
}

impl VectorSearch {
    /// Create a new VectorSearch actor with a database connection.
    pub fn new(db: PlinthDb, vector_truncation: usize) -> Result<Self, fastembed::Error> {
        // Initialize the embedding model
        let mut init_options = fastembed::TextInitOptions::default();
        init_options.model_name = EmbeddingModel::AllMiniLML6V2;
        init_options.show_download_progress = true;
        let embedding_model = TextEmbedding::try_new(init_options)?;

        Ok(Self {
            db,
            embedding_model: Arc::new(Mutex::new(embedding_model)),
            vector_truncation,
        })
    }

    /// Generate an embedding for a text query.
    ///
    /// The blocking fastembed call runs on the blocking thread pool so it never
    /// occupies the async executor.
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
        // Truncate on a char boundary so a multi-byte codepoint straddling
        // `vector_truncation` cannot panic.
        let truncated =
            crate::services::truncate_on_char_boundary(text, self.vector_truncation).to_string();
        let model = Arc::clone(&self.embedding_model);

        let embeddings = tokio::task::spawn_blocking(move || -> Result<Vec<Vec<f32>>, String> {
            let mut model = model
                .lock()
                .map_err(|_| "embedding model mutex poisoned".to_string())?;
            model
                .embed(vec![truncated], None)
                .map_err(|e| format!("Failed to generate embedding: {e}"))
        })
        .await
        .map_err(|e| format!("Embedding task failed: {e}"))??;

        let embedding = embeddings
            .into_iter()
            .next()
            .ok_or_else(|| "No embedding generated".to_string())?;
        if embedding.len() != EMBEDDING_DIM {
            return Err(format!(
                "embedding model dimension {} does not match database schema {}",
                embedding.len(),
                EMBEDDING_DIM
            ));
        }
        Ok(embedding)
    }

    fn row_to_blog_post(row: &sqlx::postgres::PgRow) -> Result<BlogPost, sqlx::Error> {
        let content_format = match row.try_get::<String, _>("content_format")?.as_str() {
            "typst" => ContentFormat::Typst,
            _ => ContentFormat::Markdown,
        };

        Ok(BlogPost {
            id: Some(row.try_get::<i64, _>("id")?.to_string()),
            slug: row.try_get("slug")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            content: row.try_get("content")?,
            html_content: row.try_get("html_content")?,
            published_at: row.try_get("published_at")?,
            updated_at: Some(row.try_get("updated_at")?),
            author: row.try_get("author")?,
            tags: row.try_get("tags")?,
            featured: row.try_get("featured")?,
            published: row.try_get("published")?,
            reading_time_minutes: row.try_get::<i32, _>("reading_time_minutes")? as u32,
            embedding: None,
            content_format,
            source: row.try_get("source")?,
            content_hash: row.try_get("content_hash")?,
            series_slug: row.try_get("series_slug")?,
            series_title: row.try_get("series_title")?,
            series_position: row
                .try_get::<Option<i32>, _>("series_position")?
                .map(|pos| pos as u32),
        })
    }

    async fn search_by_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<(BlogPost, f32)>, String> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                slug,
                title,
                description,
                -- Search results only need a short excerpt (BlogListItem uses at
                -- most 200 chars of content and never html_content), so avoid
                -- transferring/decoding the full body columns.
                LEFT(content, 200) AS content,
                ''::text AS html_content,
                published_at,
                updated_at,
                author,
                tags,
                featured,
                published,
                reading_time_minutes,
                content_format,
                source,
                content_hash,
                series_slug,
                series_title,
                series_position,
                1 - (embedding <=> $1) AS similarity
            FROM blog_posts
            WHERE embedding IS NOT NULL AND published = true
            ORDER BY embedding <=> $1
            LIMIT $2
            "#,
        )
        .bind(Vector::from(embedding))
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Vector search query failed: {e}"))?;

        rows.into_iter()
            .map(|row| {
                let post = Self::row_to_blog_post(&row)
                    .map_err(|e| format!("Failed to decode vector search result: {e}"))?;
                let similarity = row
                    .try_get::<f64, _>("similarity")
                    .map_err(|e| format!("Failed to decode similarity score: {e}"))?
                    as f32;
                Ok((post, similarity))
            })
            .collect()
    }
}

// Messages for vector search

/// Search for similar articles based on a text query
pub struct SearchSimilarArticles {
    pub query: String,
    pub limit: usize,
}

impl Message<SearchSimilarArticles> for VectorSearch {
    type Reply = Result<Vec<(BlogPost, f32)>, String>; // Vec<(post, similarity_score)>

    async fn handle(
        &mut self,
        msg: SearchSimilarArticles,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let embedding = self.generate_embedding(&msg.query).await?;
        self.search_by_embedding(embedding, msg.limit).await
    }
}

/// Find articles related to a specific article
pub struct FindRelatedArticles {
    pub slug: String,
    pub limit: usize,
}

impl Message<FindRelatedArticles> for VectorSearch {
    type Reply = Result<Vec<(BlogPost, f32)>, String>;

    async fn handle(
        &mut self,
        msg: FindRelatedArticles,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let rows = sqlx::query(
            r#"
            WITH source_post AS (
                SELECT embedding
                FROM blog_posts
                WHERE slug = $1 AND embedding IS NOT NULL AND published = true
            )
            SELECT
                blog_posts.id,
                blog_posts.slug,
                blog_posts.title,
                blog_posts.description,
                -- Only a short excerpt is needed downstream (see search_by_embedding).
                LEFT(blog_posts.content, 200) AS content,
                ''::text AS html_content,
                blog_posts.published_at,
                blog_posts.updated_at,
                blog_posts.author,
                blog_posts.tags,
                blog_posts.featured,
                blog_posts.published,
                blog_posts.reading_time_minutes,
                blog_posts.content_format,
                blog_posts.source,
                blog_posts.content_hash,
                blog_posts.series_slug,
                blog_posts.series_title,
                blog_posts.series_position,
                1 - (blog_posts.embedding <=> source_post.embedding) AS similarity
            FROM blog_posts, source_post
            WHERE blog_posts.embedding IS NOT NULL
                AND blog_posts.published = true
                AND blog_posts.slug <> $1
            ORDER BY blog_posts.embedding <=> source_post.embedding
            LIMIT $2
            "#,
        )
        .bind(msg.slug)
        .bind(msg.limit as i64)
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Related articles query failed: {e}"))?;

        rows.into_iter()
            .map(|row| {
                let post = Self::row_to_blog_post(&row)
                    .map_err(|e| format!("Failed to decode related article result: {e}"))?;
                let similarity = row
                    .try_get::<f64, _>("similarity")
                    .map_err(|e| format!("Failed to decode similarity score: {e}"))?
                    as f32;
                Ok((post, similarity))
            })
            .collect()
    }
}

/// Generate an embedding for a given text (used for backfilling declarative articles)
pub struct GenerateEmbedding {
    pub text: String,
}

impl Message<GenerateEmbedding> for VectorSearch {
    type Reply = Result<Vec<f32>, String>;

    async fn handle(
        &mut self,
        msg: GenerateEmbedding,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.generate_embedding(&msg.text).await
    }
}

/// Track opinion evolution on a topic over time
pub struct TrackOpinionEvolution {
    pub topic: String,
    pub min_similarity: f32, // Minimum similarity score to include (e.g., 0.5)
}

impl Message<TrackOpinionEvolution> for VectorSearch {
    type Reply = Result<Vec<(BlogPost, f32)>, String>; // Sorted by date (oldest first)

    async fn handle(
        &mut self,
        msg: TrackOpinionEvolution,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let embedding = self.generate_embedding(&msg.topic).await?;
        let rows = sqlx::query(
            r#"
            WITH ranked_posts AS (
                SELECT
                    id,
                    slug,
                    title,
                    description,
                    -- Only a short excerpt is needed downstream (see search_by_embedding).
                    LEFT(content, 200) AS content,
                    ''::text AS html_content,
                    published_at,
                    updated_at,
                    author,
                    tags,
                    featured,
                    published,
                    reading_time_minutes,
                    content_format,
                    source,
                    content_hash,
                    series_slug,
                    series_title,
                    series_position,
                    1 - (embedding <=> $1) AS similarity
                FROM blog_posts
                WHERE embedding IS NOT NULL AND published = true
                ORDER BY embedding <=> $1
                LIMIT $3
            )
            SELECT *
            FROM ranked_posts
            WHERE similarity >= $2
            ORDER BY published_at ASC
            "#,
        )
        .bind(Vector::from(embedding))
        .bind(msg.min_similarity as f64)
        .bind(OPINION_EVOLUTION_CANDIDATE_LIMIT)
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Opinion evolution query failed: {e}"))?;

        rows.into_iter()
            .map(|row| {
                let post = Self::row_to_blog_post(&row)
                    .map_err(|e| format!("Failed to decode opinion evolution result: {e}"))?;
                let similarity = row
                    .try_get::<f64, _>("similarity")
                    .map_err(|e| format!("Failed to decode similarity score: {e}"))?
                    as f32;
                Ok((post, similarity))
            })
            .collect()
    }
}
