# Graph Report - /data/nvme0/can/canix/projects/repos/owned/codeberg.org/caniko/plinth  (2026-07-13)

## Corpus Check
- 317 files · ~431,937 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 3172 nodes · 6570 edges · 210 communities (200 shown, 10 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 264 edges (avg confidence: 0.77)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- HTMX Runtime
- Forge Clients
- Shared Types
- CLI Image Scanner
- Todo Cache
- Pcomfy Comfyui
- Migrations Service
- Service Area
- Server Runtime
- CLI Commands
- Server Forge Activity
- Activity Cache
- Vector Search Actor
- Declarative Content Service
- Project Area
- Shared Serde Helpers
- Server Rendering Modes
- Project Dev
- Server Activity Brick
- Client Component
- Project Generator
- Client Application
- Docs Plinth Toml
- Server Search API
- Project Generator Group 2
- Blog Cache
- Rows Service
- Server Activity Refresh
- Client Blog API
- Client Application Group 2
- Core Cache Actor
- Server Images API
- Shared Domain Group 2
- Project Capability Matrix
- Shared Types Group 2
- Project Install
- Project Audit
- Server Observability
- Client Common API
- Pcomfy Format
- Project Diagnostics
- Portfolio Cache
- Docs Admin
- CLI Cli
- CLI Portfolio
- Server Admin API
- Server Error
- Shared Config
- Client Activity API
- Person Models
- Project Types
- Server Feeds API
- Server Runtime Group 2
- Script Home Streaming
- CLI Prompts
- CLI Typst Processor
- Shared Config Group 2
- CLI Activity
- CLI Ui
- Project Model
- Server Router
- Project Inspect
- Project Serde
- Activity Refresh
- Server Portfolio Publish
- Agent Skills
- Shared Domain Group 3
- Shared Types Group 3
- CLI Blog
- Shared Site Content
- Client Portfolio API
- Project Hero
- Blog Api
- Shared Content Format
- Shared Portfolio Item
- Project Generator Group 3
- Pcomfy Immich
- Project Person Mention
- Project Custom
- Project Feature Grid
- Blog Admin
- Portfolio Admin
- Docs Publishing
- Docs Readme
- CLI Activity Group 2
- Client Drawing Component
- Project Comparison
- Activity Api
- Server Error Group 2
- Shared Post
- Shared Parse
- Project Content
- Project Diagnostics Group 2
- Activity Admin
- Server Activity Feed Search
- Shared Config Group 3
- Agent Skills Group 2
- Woodpecker Area
- Docs Agents
- CLI Client
- CLI Portfolio Group 2
- Client Header Component
- Dioxus Migration Shell
- Server Health API
- Server Public API
- Portfolio Api
- Server Blog Post Conversion
- Shared Series
- Docs Overview
- CLI Todo
- Shared Tag
- Project Audience Grid
- Project Trust Panel
- Project Workflow Steps
- Project Publish
- Project Html
- Project Generator Group 4
- Docs Activity
- Package Area
- Project Screenshot Grid
- Project Dev Group 2
- Docs Activity Group 2
- Script Dev Db
- Forgejo Workflows
- Client Support
- Project Install Group 2
- Project Init
- Docs Search
- Docs Actor System
- Docs Rendering
- Docs Csr
- CLI Status
- CLI Tags
- Client Support Cta Component
- Project Audience Grid Group 2
- Project Comparison Group 2
- Project Feature Grid Group 2
- Project Hero Group 2
- Project Trust Panel Group 2
- Project Workflow Steps Group 2
- Docs Reverse Proxy
- Docs Contributing
- Docs Setup
- Docs Testing
- Repository Support
- CLI Content
- Client Theme Toggle Component
- Client Activity Detail
- Project Screenshot Grid Group 2
- Project Preset
- Seed Service
- Docs Installation
- Brand Assets
- CLI Error
- CLI Check Config
- CLI Static Site
- CLI Init
- Client About
- Brand Assets Group 2
- Brand Assets Group 3
- Brand Assets Group 4
- CLI Completions
- Client Application Group 3
- Client Error Message Component
- Client Not Found
- Client Portfolio Detail
- Client Series Detail
- Client Todo Detail
- Server Migration Integration
- Shared Types Group 4
- Brand Assets Group 5
- Brand Assets Group 6
- Brand Assets Group 7
- Brand Assets Group 8
- Brand Assets Group 9
- Brand Assets Group 10
- Brand Assets Group 11
- Brand Assets Group 12
- Brand Assets Group 13
- Brand Assets Group 14
- Brand Assets Group 15
- Brand Assets Group 16
- Brand Assets Group 17
- Brand Assets Group 18
- Brand Assets Group 19
- Nix Direnv Reload
- Docs Environment Vars

## God Nodes (most connected - your core abstractions)
1. `AppState` - 79 edges
2. `PlinthError` - 47 edges
3. `ActivityCache` - 38 edges
4. `ProjectSite` - 34 edges
5. `De()` - 30 edges
6. `BlogCache` - 29 edges
7. `FetchedActivity` - 29 edges
8. `ne()` - 29 edges
9. `ce()` - 29 edges
10. `ProjectBrick` - 28 edges

## Surprising Connections (you probably didn't know these)
- `Plinth Blog Writer` --semantically_similar_to--> `Plinth Blog Writer`  [INFERRED] [semantically similar]
  .skills/blog-writer/SKILL.md → .agents/skills/blog-writer/SKILL.md
- `Plinth Skill Index` --semantically_similar_to--> `Plinth Skill Index`  [INFERRED] [semantically similar]
  .skills/INDEX.md → .agents/skills/INDEX.md
- `Deploy Documentation to Codeberg Pages` --semantically_similar_to--> `Pages Publish Job`  [INFERRED] [semantically similar]
  .woodpecker.yml → .forgejo/workflows/pages.yaml
- `Typst Authoring Support` --conceptually_related_to--> `Typst Blog Authoring`  [INFERRED]
  README.md → .agents/skills/blog-writer/SKILL.md
- `Woodpecker CI` --references--> `Woodpecker CI Workflow`  [EXTRACTED]
  README.md → .woodpecker.yml

## Import Cycles
- 2-file cycle: `crates/shared/src/toml_config/defaults.rs -> crates/shared/src/toml_config/types.rs -> crates/shared/src/toml_config/defaults.rs`

## Hyperedges (group relationships)
- **Forgejo CI Quality, Build, and Release Flow** — _forgejo_workflows_ci_flake_check, _forgejo_workflows_ci_build_packages, _forgejo_workflows_ci_release_cache [EXTRACTED 1.00]
- **Plinth Typst Image Authoring Flow** — _agents_skills_blog_writer_skill_typst_authoring, _agents_skills_blog_writer_skill_blog_image, _agents_skills_blog_writer_skill_immich_image_upload, _agents_skills_blog_writer_skill_plinth_cli_publish [EXTRACTED 1.00]
- **Semantic Search Runtime Flow** — docs_src_architecture_actor_system_vector_search, docs_src_api_search_query_embedding, docs_src_api_search_pgvector_hnsw_cosine, docs_src_api_search_semantic_search_endpoint [INFERRED 0.95]
- **Plinth Four-Crate Architecture** — docs_src_architecture_overview_plinth_four_crate_workspace, docs_src_architecture_overview_plinth_shared, docs_src_architecture_overview_plinth_client, docs_src_architecture_overview_plinth_server, docs_src_architecture_overview_plinth_cli [EXTRACTED 1.00]
- **Plinth Rendering Strategy** — docs_src_architecture_rendering_static_generation, docs_src_architecture_rendering_streaming_ssr, docs_src_architecture_rendering_dynamic_ssr, docs_src_architecture_rendering_leptos_islands_boundary [EXTRACTED 1.00]
- **Typst Image Publishing Flow** — docs_src_guides_publishing_typst_workflow, docs_src_guides_publishing_blog_image_functions, docs_src_guides_image_handling_typst_image_upload, docs_src_guides_image_handling_immich_integration, docs_src_guides_publishing_admin_articles_api [EXTRACTED 1.00]

## Communities (210 total, 10 thin omitted)

### Community 0 - "HTMX Runtime"
Cohesion: 0.08
Nodes (103): _(), a(), Ae(), an(), at(), B(), be(), bn() (+95 more)

### Community 1 - "Forge Clients"
Cohesion: 0.05
Nodes (74): CodebergClient, FjIssue, FjLabel, FjPrMeta, FjPull, FjRepo, map_status(), merge_timestamp() (+66 more)

### Community 2 - "Shared Types"
Cohesion: 0.06
Nodes (64): default_about_title(), default_animated_background(), default_author_name(), default_blog_title(), default_cache_max_age(), default_codeberg_base_url(), default_database_url(), default_description() (+56 more)

### Community 3 - "CLI Image Scanner"
Cohesion: 0.05
Nodes (67): build_typst_frontmatter(), generate_embedding(), interactive_publish(), publish_article(), publish_markdown(), publish_typst(), ApiClient, HashMap (+59 more)

### Community 4 - "Todo Cache"
Cohesion: 0.06
Nodes (57): ApiClient, Result, Vec, get_todo_by_slug(), get_todos(), get_todos_by_tag(), query_todo_by_slug(), query_todos() (+49 more)

### Community 5 - "Pcomfy Comfyui"
Cohesion: 0.07
Nodes (55): ComfyUIInfo, download_image(), GeneratedImage, HistoryEntry, HistoryOutput, HistoryStatus, http_client(), list_workflows() (+47 more)

### Community 6 - "Migrations Service"
Cohesion: 0.06
Nodes (41): activity_migrations(), Vec, ActivityBrick, Vec, blog_migrations(), Vec, BlogBrick, Vec (+33 more)

### Community 7 - "Service Area"
Cohesion: 0.06
Nodes (50): html_escape(), markdown_to_html(), parse_image_dimensions(), render_image_tag(), Option, String, generate_slug(), parse_markdown() (+42 more)

### Community 8 - "Server Runtime"
Cohesion: 0.06
Nodes (36): cache_app(), get_cache_control(), main(), Option, Router, String, security_headers_app(), test_app() (+28 more)

### Community 9 - "CLI Commands"
Cohesion: 0.09
Nodes (44): check_sites(), check_target(), failed_probe(), load_config(), print_human_report(), ProbeKind, ProbeReport, resolve_config_path() (+36 more)

### Community 10 - "Server Forge Activity"
Cohesion: 0.11
Nodes (40): ForgeClient, Send, Sync, NoopForge, ForgeResult, activity_feed_returns_valid_rss(), activity_request(), admin_upsert_creates_and_upserts_by_natural_key() (+32 more)

### Community 11 - "Activity Cache"
Cohesion: 0.09
Nodes (30): ActivityCache, ActivityInvalidateCache, ActivityRefreshHandle, apply_limit(), CachedRefreshTarget, db_error(), GetRankedActivity, parse_activity_enum() (+22 more)

### Community 12 - "Vector Search Actor"
Cohesion: 0.10
Nodes (37): FindRelatedArticles, GenerateEmbedding, Arc, Context, Error, Message, PgRow, PlinthDb (+29 more)

### Community 13 - "Declarative Content Service"
Cohesion: 0.12
Nodes (41): add_tag_to_todo(), create_todo(), delete_todo(), invalidate_caches(), remove_tag_from_todo(), Json, Path, Result (+33 more)

### Community 14 - "Project Area"
Cohesion: 0.06
Nodes (19): AudienceGridBrick, CapabilityMatrixBrick, ComparisonBrick, ContentBrick, CustomBrick, FeatureGridBrick, HeroBrick, InstallBrick (+11 more)

### Community 15 - "Shared Serde Helpers"
Cohesion: 0.11
Nodes (16): A, AnyToStringVisitor, deserialize_flexible_id(), FlexibleIdVisitor, parse(), Error, Formatter, Option (+8 more)

### Community 16 - "Server Rendering Modes"
Cohesion: 0.11
Nodes (37): LeptosOptions, Self, IntoView, String, shell(), admin_json(), app_state(), assert_html_contains() (+29 more)

### Community 17 - "Project Dev"
Cohesion: 0.14
Nodes (33): AtomicBool, AtomicU64, bind_listener(), content_type(), DevServerError, ReloadState, render_dev_site(), request() (+25 more)

### Community 18 - "Server Activity Brick"
Cohesion: 0.11
Nodes (37): age_days_sql(), query_ranked_list(), Error, Option, PlinthDb, Result, String, Vec (+29 more)

### Community 19 - "Client Component"
Cohesion: 0.08
Nodes (28): CanvasRenderingContext2d, Closure, AnimatedBackground(), CanvasRuntime, normalize_preset(), prefers_reduced_motion(), IntoView, Self (+20 more)

### Community 20 - "Project Generator"
Cohesion: 0.14
Nodes (34): preview_site(), Result, AuditCommands, AuditInstallArgs, AuditSiteArgs, BuildArgs, CheckArgs, Cli (+26 more)

### Community 21 - "Client Application"
Cohesion: 0.10
Nodes (32): BoxStream, event_matches_scope(), invalidate_blog_static_routes(), regenerate_on(), Fn, Option, Send, SsrMode (+24 more)

### Community 22 - "Docs Plinth Toml"
Cohesion: 0.06
Nodes (35): Configuration Precedence, Database URL Overrides, Environment Variables, Immich Environment Credentials, OTLP Observability Overrides, Plausible Analytics Overrides, Database Configuration, Donation Links (+27 more)

### Community 23 - "Server Search API"
Cohesion: 0.11
Nodes (17): OpinionQuery, related_articles(), RelatedQuery, Json, Path, Query, Result, State (+9 more)

### Community 24 - "Project Generator Group 2"
Cohesion: 0.18
Nodes (26): ProjectSite, create_clean_dir(), dev_reload_script(), escape_css_value(), escape_json(), interaction_script(), person_by_id(), primary_person() (+18 more)

### Community 25 - "Blog Cache"
Cohesion: 0.15
Nodes (20): BlogCache, GetAllBlogPosts, GetAllSeries, GetBlogPost, GetPostsByTag, GetSeriesNav, GetSeriesPosts, InvalidateCache (+12 more)

### Community 26 - "Rows Service"
Cohesion: 0.21
Nodes (24): activity_item(), activity_list_item(), as_u32(), as_u32_rejects_negative_and_names_the_column(), blog_list_item(), blog_post(), content_format(), decode_error() (+16 more)

### Community 27 - "Server Activity Refresh"
Cohesion: 0.15
Nodes (21): app_state_with(), forge_config(), fresh_data_fires_no_refresh(), MockForge, MockMode, refresh_error_keeps_prior_data_and_200s(), Arc, AtomicUsize (+13 more)

### Community 28 - "Client Blog API"
Cohesion: 0.27
Nodes (25): content_format(), get_all_series(), get_blog_post_by_slug(), get_blog_posts(), get_blog_posts_by_tag(), get_series_nav(), get_series_posts(), query_all_series() (+17 more)

### Community 29 - "Client Application Group 2"
Cohesion: 0.08
Nodes (18): App(), IntoView, SiteConfig, use_site_config(), ActivityPage(), IntoView, BlogListPage(), IntoView (+10 more)

### Community 31 - "Core Cache Actor"
Cohesion: 0.16
Nodes (16): CoreCache, GetAllTags, GetSiteContent, InvalidateCache, Context, HashMap, Instant, Message (+8 more)

### Community 32 - "Server Images API"
Cohesion: 0.13
Nodes (20): default_size(), generate_etag(), ImageQuery, immich_asset_url(), Path, Query, Response, Result (+12 more)

### Community 33 - "Shared Domain Group 2"
Cohesion: 0.09
Nodes (4): default_source(), test_blog_list_item_from_blog_post_propagates_series(), test_blog_post_serialization_roundtrip(), test_blog_post_with_series_fields()

### Community 34 - "Project Capability Matrix"
Cohesion: 0.17
Nodes (20): BTreeMap, labelize(), load_capability_matrix(), Matrix, matrix_items(), MatrixItem, ConfigError, Path (+12 more)

### Community 35 - "Shared Types Group 2"
Cohesion: 0.21
Nodes (15): forge_label(), state_label(), ActivityItem, ActivityKind, ActivityListItem, ActivityState, FetchedActivity, Forge (+7 more)

### Community 36 - "Project Install"
Cohesion: 0.16
Nodes (17): build_install_ux_report(), InstallRouteUxFinding, InstallUxReport, String, Vec, validate_install_section(), InstallRoute, InstallSection (+9 more)

### Community 37 - "Project Audit"
Cohesion: 0.20
Nodes (18): audit_install(), audit_site(), capture_screenshot(), create_clean_dir(), explicit_routes_are_normalized_for_static_server_urls(), normalize_routes(), rendered_routes(), route_from_slug() (+10 more)

### Community 38 - "Server Observability"
Cohesion: 0.15
Nodes (19): init_observability(), init_otlp_tracer_provider(), ObservabilityConfig, parse_otlp_headers(), Box, Default, Error, HashMap (+11 more)

### Community 39 - "Client Common API"
Cohesion: 0.21
Nodes (21): api_url(), as_u32(), decode_error(), encode_segment(), fetch_json(), fetch_json_inner(), get_site_config(), get_site_content() (+13 more)

### Community 40 - "Pcomfy Format"
Cohesion: 0.18
Nodes (16): add_hero_image(), Article, detect_cluster(), parse_frontmatter(), prompt_for_article(), Path, Result, String (+8 more)

### Community 41 - "Project Diagnostics"
Cohesion: 0.15
Nodes (17): build_site(), Result, check_site(), Result, assert_valid(), DiagnosticReport, install_ux_report(), rejects_choice_overload_and_missing_recommendation() (+9 more)

### Community 42 - "Portfolio Cache"
Cohesion: 0.18
Nodes (14): GetAllPortfolioItems, GetPortfolioItem, InvalidateCache, PortfolioCache, Context, HashMap, Instant, Message (+6 more)

### Community 43 - "Docs Admin"
Cohesion: 0.11
Nodes (20): Immich Local Image Upload, plinth-cli Publish Workflow, Immich Image Proxy, Typst Publishing Flow, Admin API, Admin Bearer Authentication, Publish Article Endpoint, Site Content Upsert (+12 more)

### Community 44 - "CLI Cli"
Cohesion: 0.22
Nodes (17): ActivityCommands, Cli, Commands, ContentCommands, ForgeArg, plinth_shared::Forge, PortfolioCommands, Commands (+9 more)

### Community 45 - "CLI Portfolio"
Cohesion: 0.21
Nodes (19): base_request(), explicit_slug_is_trimmed_and_preserved(), publish(), rejects_empty_description(), rejects_empty_tech_stack(), rejects_empty_title(), rejects_non_markdown_format(), rejects_whitespace_tech_entry() (+11 more)

### Community 46 - "Server Admin API"
Cohesion: 0.17
Nodes (18): auth_middleware(), constant_time_eq(), get_admin_site_content(), list_tags(), Body, Json, Next, Option (+10 more)

### Community 47 - "Server Error"
Cohesion: 0.17
Nodes (13): ErrorBody, PlinthError, Box, Display, Error, From, Into, Option (+5 more)

### Community 48 - "Shared Config"
Cohesion: 0.21
Nodes (19): AboutPageConfig, AnalyticsConfig, AuthorConfig, BlogPageConfig, default_project_name(), default_project_url(), DonationConfig, DonationLink (+11 more)

### Community 49 - "Client Activity API"
Cohesion: 0.23
Nodes (17): activity_age_days_sql(), activity_score_param(), activity_score_sql(), get_activity_item_by_id(), get_activity_list(), parse_activity_token(), poke_activity_refresh(), query_activity_item() (+9 more)

### Community 50 - "Person Models"
Cohesion: 0.19
Nodes (14): ExternalLink, LinkKind, normalized_links(), normalized_links_drops_incomplete_entries(), project_reference_orders_canonical_links_first(), ProjectReference, Into, Option (+6 more)

### Community 51 - "Project Types"
Cohesion: 0.28
Nodes (18): AssetConfig, ConfigError, default_base_url(), LinkConfig, PageConfig, PersonConfig, PersonLinkConfig, ProjectConfig (+10 more)

### Community 52 - "Server Feeds API"
Cohesion: 0.25
Nodes (13): activity_feed(), blog_feed(), projects_feed(), resolve_base_url(), Path, Response, Result, State (+5 more)

### Community 53 - "Server Runtime Group 2"
Cohesion: 0.27
Nodes (18): attach_tag_to_todo(), blog_tag_names(), column_text_array(), ensure_tag(), insert_activity(), insert_blog_post(), insert_todo(), Client (+10 more)

### Community 54 - "Script Home Streaming"
Cohesion: 0.11
Nodes (17): CARGO_BUILD_JOBS, DATABASE_URL, LEPTOS_OUTPUT_NAME, LEPTOS_SITE_ADDR, LEPTOS_SITE_PKG_DIR, LEPTOS_SITE_ROOT, need_cmd(), PGDATA (+9 more)

### Community 55 - "CLI Prompts"
Cohesion: 0.19
Nodes (16): ContentSource, handle_inquire_err(), prompt_bool(), prompt_content(), prompt_optional_text(), prompt_tags(), prompt_text(), Display (+8 more)

### Community 56 - "CLI Typst Processor"
Cohesion: 0.21
Nodes (17): compile_typst_to_html(), extract_text_for_embedding(), extract_typst_frontmatter(), Option, Result, String, Vec, strip_typst_frontmatter() (+9 more)

### Community 57 - "Shared Config Group 2"
Cohesion: 0.19
Nodes (13): default_animated_background(), default_description(), default_lang(), default_nav(), default_site_name(), default_tagline(), default_theme(), test_donation_config_default() (+5 more)

### Community 58 - "CLI Activity"
Cohesion: 0.25
Nodes (15): add(), build_request(), build_request_maps_fetched_and_flags(), fetch(), generate_embedding(), list(), remove(), ApiClient (+7 more)

### Community 59 - "CLI Ui"
Cohesion: 0.16
Nodes (11): bold_style(), dim_style(), error_style(), info_style(), print_error(), Error, spinner(), success_style() (+3 more)

### Community 60 - "Project Model"
Cohesion: 0.24
Nodes (9): asset_json(), Asset, NavLink, Page, Into, PathBuf, Self, String (+1 more)

### Community 61 - "Server Router"
Cohesion: 0.18
Nodes (15): AppState, ImmichConfig, ActorRef, Client, Option, PlinthConfig, PlinthDb, SiteConfig (+7 more)

### Community 62 - "Project Inspect"
Cohesion: 0.19
Nodes (15): inspect_json(), inspect_site(), print_inspection(), Path, PathBuf, Result, Vec, section_name() (+7 more)

### Community 63 - "Project Serde"
Cohesion: 0.37
Nodes (14): build_page(), build_section(), build_site(), build_theme(), find_person(), load_project_config(), project_watch_paths(), push_watch_path() (+6 more)

### Community 64 - "Activity Refresh"
Cohesion: 0.22
Nodes (15): RefreshOutcome, RefreshTarget, reread_ranked(), Arc, DateTime, Error, PlinthDb, Result (+7 more)

### Community 65 - "Server Portfolio Publish"
Cohesion: 0.27
Nodes (13): app_state(), manifest(), NoopForge, post_manifest(), posting_same_slug_upserts_without_duplicate(), posting_valid_manifest_creates_row_and_refreshes_cached_list(), posting_without_bearer_token_returns_401(), ForgeResult (+5 more)

### Community 66 - "Agent Skills"
Cohesion: 0.14
Nodes (15): Blog Writer Skill Entry, Plinth Skill Index, blog-image Function, Extension-Based Content Format Detection, gallery Function, hero-image Function, Three-Part Image Suggestion Workflow, Immich Local Image Upload (+7 more)

### Community 67 - "Shared Domain Group 3"
Cohesion: 0.18
Nodes (6): test_validate_rejects_empty_repo_name(), test_validate_rejects_empty_repo_owner(), test_validate_rejects_impact_above_range(), test_validate_rejects_impact_below_range(), test_validate_rejects_non_positive_number(), valid_request()

### Community 68 - "Shared Types Group 3"
Cohesion: 0.23
Nodes (9): ActivityValidationError, ParseEnumError, Display, Error, Formatter, Result, Self, validate_activity_fields() (+1 more)

### Community 69 - "CLI Blog"
Cohesion: 0.21
Nodes (7): ApiClient, PublishArticleResponse, Option, Result, String, Value, Vec

### Community 70 - "Shared Site Content"
Cohesion: 0.19
Nodes (9): ApiClient, Option, Result, DateTime, Option, String, Utc, SiteContent (+1 more)

### Community 71 - "Client Portfolio API"
Cohesion: 0.29
Nodes (13): get_portfolio_item_by_slug(), get_portfolio_items(), query_portfolio_item_by_slug(), query_portfolio_items(), row_portfolio_item(), Error, Option, PgPool (+5 more)

### Community 72 - "Project Hero"
Cohesion: 0.22
Nodes (10): Cta, Hero, Into, Option, Self, String, Vec, render_hero() (+2 more)

### Community 73 - "Blog Api"
Cohesion: 0.42
Nodes (13): get_blog_post(), get_series_nav(), list_blog_posts(), list_blog_posts_by_tag(), list_series(), list_series_posts(), Json, Option (+5 more)

### Community 74 - "Shared Content Format"
Cohesion: 0.18
Nodes (8): PublishArticleRequest, Option, String, Vec, ContentFormat, Display, Formatter, Result

### Community 75 - "Shared Portfolio Item"
Cohesion: 0.22
Nodes (7): PortfolioItem, PublishPortfolioRequest, DateTime, Option, String, Utc, Vec

### Community 76 - "Project Generator Group 3"
Cohesion: 0.26
Nodes (12): AsRef, load_project_site(), capability_matrix_loads_legacy_games_source(), capability_matrix_loads_neutral_items_source(), capability_matrix_source_contributes_watch_path(), person_config_renders_author_metadata_and_links(), preset_catppuccin_latte_fills_all_theme_colors(), preset_with_individual_override() (+4 more)

### Community 77 - "Pcomfy Immich"
Cohesion: 0.36
Nodes (12): ExifInfo, get_asset_info(), http_client(), probe(), proxy_url(), Client, Option, Result (+4 more)

### Community 78 - "Project Person Mention"
Cohesion: 0.22
Nodes (10): PersonReference, build_person_mention(), Option, String, PersonMention, Option, String, render_person_mention() (+2 more)

### Community 79 - "Project Custom"
Cohesion: 0.24
Nodes (9): CustomSection, Arc, Fn, Into, Option, Self, Send, String (+1 more)

### Community 80 - "Project Feature Grid"
Cohesion: 0.22
Nodes (9): Feature, FeatureGrid, Into, Option, Self, String, Vec, render_feature_grid() (+1 more)

### Community 81 - "Blog Admin"
Cohesion: 0.37
Nodes (12): add_tag_to_post(), delete_article(), publish_article(), PublishArticleResponse, remove_tag_from_post(), Json, Option, Path (+4 more)

### Community 82 - "Portfolio Admin"
Cohesion: 0.35
Nodes (12): publish_portfolio_item(), PublishPortfolioResponse, required_text(), Json, Option, Result, State, String (+4 more)

### Community 83 - "Docs Publishing"
Cohesion: 0.18
Nodes (13): Image Handling, Immich Integration, Immutable Image Caching, Private Immich Image Proxy, Typst Image Upload and URL Rewriting, UUID Asset Validation, Admin Articles API, 384-Dimensional Article Embedding (+5 more)

### Community 84 - "Docs Readme"
Cohesion: 0.17
Nodes (13): Four-Crate Rust Workspace, Immich Integration, NixOS Deployment Module, OTLP Observability, Plausible Analytics Integration, Plinth, plinth-cli Crate, plinth-client Crate (+5 more)

### Community 85 - "CLI Activity Group 2"
Cohesion: 0.26
Nodes (7): ApiClient, PublishActivityResponse, RawPublishActivityResponse, Option, Result, String, Vec

### Community 86 - "Client Drawing Component"
Cohesion: 0.21
Nodes (4): super::CanvasRuntime, hash_noise(), palette(), smooth_noise()

### Community 87 - "Project Comparison"
Cohesion: 0.23
Nodes (8): ComparisonRow, ComparisonSection, Option, String, Vec, render_comparison(), String, ProjectSection

### Community 88 - "Activity Api"
Cohesion: 0.24
Nodes (11): ActivityListQuery, get_activity_item(), list_activity_items(), Json, Option, Path, Query, Result (+3 more)

### Community 90 - "Shared Post"
Cohesion: 0.27
Nodes (9): BlogListItem, BlogPost, DateTime, From, Option, Self, String, Utc (+1 more)

### Community 91 - "Shared Parse"
Cohesion: 0.23
Nodes (7): ConfigError, PlinthConfig, Error, Result, Self, SiteConfig, String

### Community 92 - "Project Content"
Cohesion: 0.22
Nodes (8): build_content(), Option, String, ContentSection, Option, String, render_content(), String

### Community 93 - "Project Diagnostics Group 2"
Cohesion: 0.27
Nodes (9): Diagnostic, Into, Self, String, Severity, AssetJson, BuildJson, CheckJson (+1 more)

### Community 94 - "Activity Admin"
Cohesion: 0.36
Nodes (10): delete_activity_handler(), patch_activity_handler(), PatchActivityBody, publish_activity_item(), Json, Option, Path, Result (+2 more)

### Community 95 - "Server Activity Feed Search"
Cohesion: 0.38
Nodes (9): activity_feed_returns_valid_xml_with_entries(), app_state(), PgPool, Router, Vec, search_returns_seeded_activity_above_min_similarity(), seed_activity(), test_app() (+1 more)

### Community 96 - "Shared Config Group 3"
Cohesion: 0.18
Nodes (6): default_about_title(), default_author_name(), default_blog_title(), default_portfolio_title(), default_todos_title(), Self

### Community 97 - "Agent Skills Group 2"
Cohesion: 0.22
Nodes (10): blog-image Function, Extension-Based Content Format Detection, gallery Function, hero-image Function, Three-Part Image Suggestion Workflow, Markdown Blog Authoring, Plinth Blog Writer, SEO and Content Best Practices (+2 more)

### Community 98 - "Woodpecker Area"
Cohesion: 0.27
Nodes (10): Atlas Nix Trusted Runner, deploy-pages Nix App, Codeberg Pages Workflow, Pages Publish Job, Build Documentation Step, Build Release Step, Check and Test Step, Deploy Documentation to Codeberg Pages (+2 more)

### Community 99 - "Docs Agents"
Cohesion: 0.20
Nodes (10): Blog Brick, Always-Present Core Services, Leptos SSR Feature Boundary, Modular Brick Architecture, Nix Sandbox Constraints, Portfolio Brick, Portfolio Manifest Publishing, Postgres and sqlx Persistence (+2 more)

### Community 100 - "CLI Client"
Cohesion: 0.31
Nodes (5): ApiClient, Client, Result, Self, String

### Community 101 - "CLI Portfolio Group 2"
Cohesion: 0.29
Nodes (7): ApiClient, PublishPortfolioResponse, Option, Result, String, Vec, SyncPortfolioResponse

### Community 102 - "Client Header Component"
Cohesion: 0.24
Nodes (7): Footer(), IntoView, Header(), MobileMenu(), IntoView, Vec, NavItem

### Community 103 - "Dioxus Migration Shell"
Cohesion: 0.33
Nodes (9): About(), App(), Home(), NotFound(), Projects(), Route, String, Vec (+1 more)

### Community 104 - "Server Health API"
Cohesion: 0.22
Nodes (6): health_check(), HealthResponse, Json, Option, State, StatusCode

### Community 105 - "Server Public API"
Cohesion: 0.24
Nodes (9): get_site_config(), get_site_content(), Json, Option, Path, Result, SiteConfig, State (+1 more)

### Community 106 - "Portfolio Api"
Cohesion: 0.27
Nodes (9): get_portfolio_item(), list_portfolio_items(), Json, Option, Path, Result, State, String (+1 more)

### Community 107 - "Server Blog Post Conversion"
Cohesion: 0.38
Nodes (9): sample_post(), test_empty_description_uses_content_preview(), test_empty_tags_preserved(), test_from_owned_basic(), test_from_ref_basic(), test_none_id_preserved(), test_nonempty_description_preserved(), test_owned_empty_description_uses_content() (+1 more)

### Community 108 - "Shared Series"
Cohesion: 0.31
Nodes (9): humanize_slug(), DateTime, Option, String, Utc, Vec, SeriesEntry, SeriesListItem (+1 more)

### Community 109 - "Docs Overview"
Cohesion: 0.27
Nodes (10): Architecture Overview, Article Publishing Pipeline, pgvector Similarity Search, plinth-cli, plinth-client, Plinth Four-Crate Workspace, plinth-server, plinth-shared (+2 more)

### Community 110 - "CLI Todo"
Cohesion: 0.53
Nodes (8): create_todo(), delete_todo(), interactive_create_todo(), list_todos(), ApiClient, Option, Result, update_todo()

### Community 111 - "Shared Tag"
Cohesion: 0.25
Nodes (6): BlogPostPage(), IntoView, AddTagRequest, Option, String, Tag

### Community 112 - "Project Audience Grid"
Cohesion: 0.31
Nodes (7): Audience, AudienceGrid, Option, String, Vec, render_audience_grid(), String

### Community 113 - "Project Trust Panel"
Cohesion: 0.31
Nodes (7): Option, String, Vec, TrustItem, TrustPanel, render_trust_panel(), String

### Community 114 - "Project Workflow Steps"
Cohesion: 0.31
Nodes (7): Option, String, Vec, WorkflowStep, WorkflowSteps, render_workflow_steps(), String

### Community 115 - "Project Publish"
Cohesion: 0.50
Nodes (8): collect_files(), collect_files_inner(), publish_manifest(), publish_site(), Path, Result, String, Vec

### Community 116 - "Project Html"
Cohesion: 0.33
Nodes (7): escape_attr(), escape_text(), id_attr(), link_kind_class(), render_external_link(), Option, String

### Community 117 - "Project Generator Group 4"
Cohesion: 0.39
Nodes (8): render_static(), dev_reload_script_is_opt_in(), identity_images_do_not_become_lightbox_triggers(), renders_custom_section(), renders_primary_person_links_and_metadata(), renders_product_brick_markers(), renders_theme_css_variables_when_configured(), screenshot_grid_images_are_lightbox_triggers()

### Community 118 - "Docs Activity"
Cohesion: 0.25
Nodes (9): Activity API, Activity Natural Key Upsert, Activity RSS Feed, Activity Semantic Search Union, Activity Admin Bearer Authentication, Impact and Recency Ranking, List Activity Endpoint, Publish Activity Endpoint (+1 more)

### Community 119 - "Package Area"
Cohesion: 0.22
Nodes (8): devDependencies, @playwright/test, name, private, scripts, test, test:headed, @playwright/test

### Community 120 - "Project Screenshot Grid"
Cohesion: 0.36
Nodes (6): String, Vec, Screenshot, ScreenshotGrid, render_screenshots(), String

### Community 121 - "Project Dev Group 2"
Cohesion: 0.46
Nodes (7): dev_site(), PathBuf, Result, String, serve_rendered_site(), serve_site(), ServeRenderedArgs

### Community 123 - "Docs Activity Group 2"
Cohesion: 0.25
Nodes (8): Activity Ranking Strategies, Forge Refresh Policy, Activity API Authentication and Rate Limits, AllMiniLML6V2 Activity Embedding, Activity Ingestion Flow, Curating External Activity, External Activity Feature, Forge Metadata Background Refresh

### Community 124 - "Script Dev Db"
Cohesion: 0.61
Nodes (7): ensure_tools(), is_running(), need_cmd(), reset_db(), dev-db.sh script, start_db(), stop_db()

### Community 125 - "Forgejo Workflows"
Cohesion: 0.43
Nodes (7): Atlas Runner, Build Packages Job, Canix Attic Cache, Forgejo CI Workflow, Flake Check Job, Release Cache Job, Forgejo Atlas CI

### Community 126 - "Client Support"
Cohesion: 0.38
Nodes (5): platform_name(), PlatformCardIcon(), IntoView, String, SupportPage()

### Community 127 - "Project Install Group 2"
Cohesion: 0.43
Nodes (6): build_install_route(), build_install_section(), InstallRouteConfig, Option, String, Vec

### Community 128 - "Project Init"
Cohesion: 0.43
Nodes (6): init_site(), initial_config_template(), Result, String, toml_escape(), InitArgs

### Community 129 - "Docs Search"
Cohesion: 0.52
Nodes (7): Opinion Evolution Endpoint, pgvector HNSW Cosine Search, 384-Dimensional Query Embedding, Related Articles Endpoint, Search API, Semantic Search Endpoint, VectorSearch Actor

### Community 130 - "Docs Actor System"
Cohesion: 0.33
Nodes (7): Actor Lifecycle in AppState, Kameo Actor System, Content Cache Actors, InvalidateCache Message, Lazy Cache Population, Actor System Documentation Entry, Architecture Documentation

### Community 131 - "Docs Rendering"
Cohesion: 0.29
Nodes (7): app_routes() Route Source of Truth, Request-Time Dynamic SSR, Leptos Islands Boundary, Rendering, Publish-Cadence Static Generation, StaticRoute Regeneration, Multi-Source Streaming SSR

### Community 132 - "Docs Csr"
Cohesion: 0.29
Nodes (7): plinth-csr Target, WASM Safety Boundary, Cross-Origin CORS Requirement, CSR Static Build, Default SSR and Hydrate Package, plinth-csr Package, Public REST Content API

### Community 133 - "CLI Status"
Cohesion: 0.33
Nodes (5): check_status(), HealthResponse, Option, Result, String

### Community 134 - "CLI Tags"
Cohesion: 0.73
Nodes (5): add_tag(), list_tags(), remove_tag(), ApiClient, Result

### Community 135 - "Client Support Cta Component"
Cohesion: 0.47
Nodes (5): platform_label(), PlatformIcon(), IntoView, String, SupportCta()

### Community 136 - "Project Audience Grid Group 2"
Cohesion: 0.47
Nodes (5): AudienceConfig, build_audience_grid(), Option, String, Vec

### Community 137 - "Project Comparison Group 2"
Cohesion: 0.47
Nodes (5): build_comparison(), ComparisonRowConfig, Option, String, Vec

### Community 138 - "Project Feature Grid Group 2"
Cohesion: 0.47
Nodes (5): build_feature_grid(), FeatureConfig, Option, String, Vec

### Community 139 - "Project Hero Group 2"
Cohesion: 0.47
Nodes (5): build_hero(), CtaConfig, Option, String, Vec

### Community 140 - "Project Trust Panel Group 2"
Cohesion: 0.47
Nodes (5): build_trust_panel(), Option, String, Vec, TrustItemConfig

### Community 141 - "Project Workflow Steps Group 2"
Cohesion: 0.47
Nodes (5): build_workflow_steps(), Option, String, Vec, WorkflowStepConfig

### Community 143 - "Docs Reverse Proxy"
Cohesion: 0.33
Nodes (6): Caddy Production Deployment, Caddy TLS Reverse Proxy, Image Proxy Cacheability, Nginx ACME Reverse Proxy, Plinth Static Asset Serving, Reverse Proxy

### Community 144 - "Docs Contributing"
Cohesion: 0.33
Nodes (6): Contributing, nix flake check Gate, Parameterized SQLx Queries, Transactional Tag Writes, WASM Feature Gating, Woodpecker CI

### Community 145 - "Docs Setup"
Cohesion: 0.33
Nodes (6): cargo leptos watch, Dev Environment, Local PostgreSQL and pgvector Cluster, mdBook Documentation Preview, Nix Development Shell, Nix Sandbox Constraints

### Community 146 - "Docs Testing"
Cohesion: 0.33
Nodes (6): Isolated PostgreSQL Test Databases, Network-Free Tests, pgvector Test Prerequisite, SQLx Integration Tests, Testing, Rust Unit Tests

### Community 147 - "Repository Support"
Cohesion: 0.40
Nodes (5): Favicon Generation Workflow, CSR Favicon Set, Plinth CSR HTML Shell, Plinth WASM Bootstrap, SSR with WASM Hydration

### Community 148 - "CLI Content"
Cohesion: 0.70
Nodes (4): get_content(), ApiClient, Result, set_content()

### Community 149 - "Client Theme Toggle Component"
Cohesion: 0.60
Nodes (4): apply_theme(), get_initial_theme(), IntoView, ThemeToggle()

### Community 150 - "Client Activity Detail"
Cohesion: 0.40
Nodes (4): ActivityDetailPage(), forge_label(), IntoView, state_label()

### Community 151 - "Project Screenshot Grid Group 2"
Cohesion: 0.60
Nodes (4): build_screenshot_grid(), String, Vec, ScreenshotConfig

### Community 152 - "Project Preset"
Cohesion: 0.50
Nodes (4): ProjectTheme, Option, resolve_preset(), Option

### Community 153 - "Seed Service"
Cohesion: 0.50
Nodes (4): Error, PlinthDb, Result, seed_sample_data()

### Community 156 - "Docs Installation"
Cohesion: 0.40
Nodes (5): Plinth Build Variants, Installation, Nix Flake Toolchain, Plinth Production Package, Production Build Artifacts

### Community 157 - "Brand Assets"
Cohesion: 0.40
Nodes (5): Navy Brick Plinth, Plinth Logo, Plinth Vector Logo, Split-Tone Brick Plinth, Teal Human Figure

### Community 158 - "CLI Error"
Cohesion: 0.50
Nodes (3): ErrorResponse, Option, String

### Community 159 - "CLI Check Config"
Cohesion: 0.50
Nodes (3): Option, Result, validate()

### Community 160 - "CLI Static Site"
Cohesion: 0.50
Nodes (3): routes(), String, Vec

### Community 161 - "CLI Init"
Cohesion: 0.67
Nodes (3): create_from_template(), Option, Result

### Community 162 - "Client About"
Cohesion: 0.67
Nodes (3): AboutPage(), DefaultAboutContent(), IntoView

### Community 164 - "Brand Assets Group 2"
Cohesion: 0.50
Nodes (4): Illuminated Human Figure, Industrial Brick Plinth, Plinth Brand Banner, Plinth Wordmark

### Community 165 - "Brand Assets Group 3"
Cohesion: 0.50
Nodes (4): Illuminated Human Figure, Industrial Brick Plinth, Plinth Brand Banner, Plinth Wordmark

### Community 166 - "Brand Assets Group 4"
Cohesion: 0.50
Nodes (3): API Route Crawl Exclusion, Public Crawl Policy, Dynamic Sitemap

### Community 168 - "Client Application Group 3"
Cohesion: 0.67
Nodes (3): Hello World Placeholder, main.scss Trunk Asset, Trunk App HTML Shell

### Community 176 - "Shared Types Group 4"
Cohesion: 0.67
Nodes (3): ActivityRefreshHook, Send, Sync

### Community 177 - "Brand Assets Group 5"
Cohesion: 0.67
Nodes (3): Navy Brick Plinth, Plinth Apple Touch Icon, Teal Human Figure

### Community 178 - "Brand Assets Group 6"
Cohesion: 0.67
Nodes (3): Navy Brick Plinth, Plinth 16 Pixel Favicon, Teal Human Figure

### Community 179 - "Brand Assets Group 7"
Cohesion: 0.67
Nodes (3): Navy Brick Plinth, Plinth 32 Pixel Favicon, Teal Human Figure

### Community 180 - "Brand Assets Group 8"
Cohesion: 0.67
Nodes (3): Dark Green Brick Plinth, Green Plinth Logo Variant, Lime Human Figure

### Community 181 - "Brand Assets Group 9"
Cohesion: 0.67
Nodes (3): Charcoal Brick Plinth, Coral Human Figure, Grey Plinth Logo Variant

### Community 182 - "Brand Assets Group 10"
Cohesion: 0.67
Nodes (3): Gold Human Figure, Purple Brick Plinth, Purple Plinth Logo Variant

### Community 183 - "Brand Assets Group 11"
Cohesion: 0.67
Nodes (3): Navy Brick Plinth, Plinth Vector Logo, Teal Human Figure

### Community 184 - "Brand Assets Group 12"
Cohesion: 0.67
Nodes (3): Navy Brick Plinth, Plinth 16 Pixel Favicon, Teal Human Figure

### Community 185 - "Brand Assets Group 13"
Cohesion: 0.67
Nodes (3): Navy Brick Plinth, Plinth 180 Pixel Favicon, Teal Human Figure

### Community 186 - "Brand Assets Group 14"
Cohesion: 0.67
Nodes (3): Navy Brick Plinth, Plinth 192 Pixel Favicon, Teal Human Figure

### Community 187 - "Brand Assets Group 15"
Cohesion: 0.67
Nodes (3): Navy Brick Plinth, Plinth 32 Pixel Favicon, Teal Human Figure

### Community 188 - "Brand Assets Group 16"
Cohesion: 0.67
Nodes (3): Navy Brick Plinth, Plinth 48 Pixel Favicon, Teal Human Figure

### Community 189 - "Brand Assets Group 17"
Cohesion: 0.67
Nodes (3): Navy Brick Plinth, Plinth 512 Pixel Favicon, Teal Human Figure

### Community 190 - "Brand Assets Group 18"
Cohesion: 0.67
Nodes (3): Navy Brick Plinth, Square Plinth Favicon, Teal Human Figure

### Community 191 - "Brand Assets Group 19"
Cohesion: 0.67
Nodes (3): Navy Brick Plinth, Plinth Vector Logo, Teal Human Figure

## Knowledge Gaps
- **162 isolated node(s):** `name`, `private`, `test`, `test:headed`, `@playwright/test` (+157 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **10 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `Server Router` to `Todo Cache`, `Server Forge Activity`, `Activity Cache`, `Vector Search Actor`, `Declarative Content Service`, `Server Rendering Modes`, `Server Activity Brick`, `Server Search API`, `Blog Cache`, `Server Activity Refresh`, `Core Cache Actor`, `Server Images API`, `Portfolio Cache`, `Server Admin API`, `Server Feeds API`, `Server Portfolio Publish`, `Blog Api`, `Blog Admin`, `Portfolio Admin`, `Activity Api`, `Activity Admin`, `Server Activity Feed Search`, `Server Health API`, `Server Public API`, `Portfolio Api`?**
  _High betweenness centrality (0.118) - this node is a cross-community bridge._
- **Why does `build_section()` connect `Project Serde` to `Project Types`, `Project Content`, `Project Person Mention`, `Project Comparison`?**
  _High betweenness centrality (0.093) - this node is a cross-community bridge._
- **Why does `ProjectSection` connect `Project Comparison` to `Project Capability Matrix`, `Project Install`, `Project Hero`, `Project Person Mention`, `Project Content`, `Project Audience Grid`, `Project Custom`, `Project Feature Grid`, `Project Trust Panel`, `Project Workflow Steps`, `Project Screenshot Grid`, `Project Generator Group 2`, `Project Model`, `Project Inspect`, `Project Serde`?**
  _High betweenness centrality (0.081) - this node is a cross-community bridge._
- **What connects `name`, `private`, `test` to the rest of the system?**
  _175 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `HTMX Runtime` be split into smaller, more focused modules?**
  _Cohesion score 0.08040293040293041 - nodes in this community are weakly interconnected._
- **Should `Forge Clients` be split into smaller, more focused modules?**
  _Cohesion score 0.05326460481099656 - nodes in this community are weakly interconnected._
- **Should `Shared Types` be split into smaller, more focused modules?**
  _Cohesion score 0.06202950918398073 - nodes in this community are weakly interconnected._