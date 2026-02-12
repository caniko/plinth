use kameo::actor::ActorRef;
use kameo::{Actor, Reply};
use kameo::message::{Context, Message};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use shared::BlogPost;

/// Vector search actor that handles semantic search queries
#[derive(Actor)]
pub struct VectorSearch {
    db: Surreal<Db>,
    embedding_model: TextEmbedding,
}

impl VectorSearch {
    /// Create a new VectorSearch actor with a SurrealDB connection
    pub fn new(db: Surreal<Db>) -> Result<Self, fastembed::Error> {
        // Initialize the embedding model
        let embedding_model = TextEmbedding::try_new(InitOptions {
            model_name: EmbeddingModel::AllMiniLML6V2, // 384 dimensions
            show_download_progress: true,
            ..Default::default()
        })?;

        Ok(Self { db, embedding_model })
    }

    /// Generate an embedding for a text query
    fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
        // Truncate text if too long
        let truncated = if text.len() > 5000 {
            &text[..5000]
        } else {
            text
        };

        // Generate embedding
        let embeddings = self
            .embedding_model
            .embed(vec![truncated.to_string()], None)
            .map_err(|e| format!("Failed to generate embedding: {}", e))?;

        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| "No embedding generated".to_string())
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
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
        _ctx: Context<'_, Self, Self::Reply>,
    ) -> Self::Reply {
        // Generate embedding for search query
        let query_embedding = self.generate_embedding(&msg.query)?;

        // Query all published blog posts with embeddings
        let result: Result<Vec<BlogPost>, _> = self
            .db
            .query("SELECT * FROM blog_posts WHERE published = true AND embedding IS NOT NULL")
            .await
            .and_then(|mut response| response.take(0));

        let posts = result.map_err(|e| format!("Database error: {}", e))?;

        // Calculate similarity scores
        let mut results: Vec<(BlogPost, f32)> = posts
            .into_iter()
            .filter_map(|post| {
                post.embedding.as_ref().map(|emb| {
                    let similarity = Self::cosine_similarity(&query_embedding, emb);
                    (post, similarity)
                })
            })
            .collect();

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N results
        results.truncate(msg.limit);

        Ok(results)
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
        _ctx: Context<'_, Self, Self::Reply>,
    ) -> Self::Reply {
        // Get the source article
        let source_result: Result<Option<BlogPost>, _> = self
            .db
            .query("SELECT * FROM blog_posts WHERE slug = $slug AND published = true")
            .bind(("slug", &msg.slug))
            .await
            .and_then(|mut response| response.take(0));

        let source_post = source_result
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Source article not found".to_string())?;

        let source_embedding = source_post
            .embedding
            .as_ref()
            .ok_or_else(|| "Source article has no embedding".to_string())?;

        // Query all other published blog posts with embeddings
        let result: Result<Vec<BlogPost>, _> = self
            .db
            .query("SELECT * FROM blog_posts WHERE published = true AND slug != $slug AND embedding IS NOT NULL")
            .bind(("slug", &msg.slug))
            .await
            .and_then(|mut response| response.take(0));

        let posts = result.map_err(|e| format!("Database error: {}", e))?;

        // Calculate similarity scores
        let mut results: Vec<(BlogPost, f32)> = posts
            .into_iter()
            .filter_map(|post| {
                post.embedding.as_ref().map(|emb| {
                    let similarity = Self::cosine_similarity(source_embedding, emb);
                    (post, similarity)
                })
            })
            .collect();

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N results
        results.truncate(msg.limit);

        Ok(results)
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
        _ctx: Context<'_, Self, Self::Reply>,
    ) -> Self::Reply {
        // Generate embedding for topic
        let topic_embedding = self.generate_embedding(&msg.topic)?;

        // Query all published blog posts with embeddings, ordered by date
        let result: Result<Vec<BlogPost>, _> = self
            .db
            .query("SELECT * FROM blog_posts WHERE published = true AND embedding IS NOT NULL ORDER BY published_at ASC")
            .await
            .and_then(|mut response| response.take(0));

        let posts = result.map_err(|e| format!("Database error: {}", e))?;

        // Calculate similarity scores and filter by minimum similarity
        let results: Vec<(BlogPost, f32)> = posts
            .into_iter()
            .filter_map(|post| {
                post.embedding.as_ref().and_then(|emb| {
                    let similarity = Self::cosine_similarity(&topic_embedding, emb);
                    if similarity >= msg.min_similarity {
                        Some((post, similarity))
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((VectorSearch::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!((VectorSearch::cosine_similarity(&c, &d) - 0.0).abs() < 0.001);

        let e = vec![1.0, 1.0, 0.0];
        let f = vec![1.0, 1.0, 0.0];
        assert!((VectorSearch::cosine_similarity(&e, &f) - 1.0).abs() < 0.001);
    }
}
