use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;

use crate::error::AppError;
use crate::models::container::PaginatedResponse;
use crate::models::skill::*;
use crate::AppState;

pub async fn create_skill(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<SkillResponse>), AppError> {
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "name" => {
                name = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("Read name error: {}", e)))?,
                );
            }
            "description" => {
                description = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("Read description error: {}", e)))?,
                );
            }
            "file" => {
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("Read file error: {}", e)))?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    let file_bytes = file_bytes.ok_or_else(|| AppError::BadRequest("file is required".to_string()))?;

    let skill_id = uuid::Uuid::new_v4().to_string();
    let skill_dir = format!("{}/{}", state.config.read().await.skills_dir, skill_id);

    // Create skill directory and extract zip
    tokio::fs::create_dir_all(&skill_dir)
        .await
        .map_err(|e| AppError::DockerError(format!("Failed to create skill dir: {}", e)))?;

    extract_zip(&file_bytes, &skill_dir)?;

    // Try to read metadata from skill.md (case-insensitive) if name not provided via multipart
    if name.is_none() {
        if let Some((manifest_name, manifest_desc)) = read_skill_manifest(&skill_dir) {
            name = Some(manifest_name);
            if description.is_none() {
                description = manifest_desc;
            }
        }
    }

    let name = name.ok_or_else(|| {
        AppError::BadRequest("name is required (provide in form or skill.md)".to_string())
    })?;
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::BadRequest(
            "name must be alphanumeric with hyphens or underscores".to_string(),
        ));
    }

    // Check name uniqueness
    if state.db.get_skill_by_name(&name).await.is_ok() {
        return Err(AppError::BadRequest(
            "A skill with this name already exists".to_string(),
        ));
    }

    let now = Utc::now().to_rfc3339();
    let skill = Skill {
        id: skill_id.clone(),
        name: name.clone(),
        description: description.unwrap_or_default(),
        created_at: now.clone(),
        updated_at: now,
    };

    state.db.create_skill(&skill).await?;

    Ok((StatusCode::CREATED, Json(skill.into())))
}

pub async fn get_skill(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SkillResponse>, AppError> {
    let skill = state.db.get_skill(&id).await?;
    Ok(Json(skill.into()))
}

pub async fn list_skills(
    State(state): State<AppState>,
    Query(params): Query<ListSkillsQuery>,
) -> Result<Json<PaginatedResponse<SkillResponse>>, AppError> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    let search = params.search.as_deref();

    let (data, total) = state.db.list_skills(search, page, per_page).await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i64;

    Ok(Json(PaginatedResponse {
        data: data.into_iter().map(|s| s.into()).collect(),
        total,
        page,
        per_page,
        total_pages,
    }))
}

pub async fn update_skill(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSkillRequest>,
) -> Result<Json<SkillResponse>, AppError> {
    // Verify skill exists
    state.db.get_skill(&id).await?;

    // Check name uniqueness if name is being changed
    if let Some(ref new_name) = payload.name {
        if new_name.is_empty()
            || !new_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(AppError::BadRequest(
                "name must be alphanumeric with hyphens or underscores".to_string(),
            ));
        }
        if let Ok(existing) = state.db.get_skill_by_name(new_name).await {
            if existing.id != id {
                return Err(AppError::BadRequest(
                    "A skill with this name already exists".to_string(),
                ));
            }
        }
    }

    state
        .db
        .update_skill(&id, payload.name.as_deref(), payload.description.as_deref())
        .await?;

    let skill = state.db.get_skill(&id).await?;
    Ok(Json(skill.into()))
}

pub async fn delete_skill(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let skill_dir = format!("{}/{}", state.config.read().await.skills_dir, id);

    state.db.delete_skill(&id).await?;

    // Clean up skill directory, ignore errors
    if let Err(e) = tokio::fs::remove_dir_all(&skill_dir).await {
        tracing::warn!("Failed to remove skill dir {}: {}", skill_dir, e);
    }

    Ok(StatusCode::NO_CONTENT)
}

fn extract_zip(data: &[u8], dest_dir: &str) -> Result<(), AppError> {
    let reader = std::io::Cursor::new(data);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| AppError::BadRequest(format!("Invalid zip: {}", e)))?;

    // Detect single top-level directory to strip (e.g. "my-skill/..." → extract contents directly)
    let strip_prefix = {
        let mut top_levels: Vec<String> = Vec::new();
        for i in 0..archive.len() {
            let file = archive
                .by_index(i)
                .map_err(|e| AppError::BadRequest(format!("Zip read error: {}", e)))?;
            if let Some(name) = file.enclosed_name() {
                if let Some(first) = name.components().next() {
                    let top = first.as_os_str().to_string_lossy().to_string();
                    if !top_levels.contains(&top) {
                        top_levels.push(top);
                    }
                }
            }
        }
        // Strip if there's exactly one top-level dir (excluding __MACOSX)
        let real_tops: Vec<_> = top_levels.iter().filter(|t| t.as_str() != "__MACOSX").collect();
        if real_tops.len() == 1 {
            Some(real_tops[0].clone())
        } else {
            None
        }
    };

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::BadRequest(format!("Zip read error: {}", e)))?;

        let enclosed = match file.enclosed_name() {
            Some(path) => path.to_path_buf(),
            None => continue,
        };

        // Skip __MACOSX directory
        if enclosed.starts_with("__MACOSX") {
            continue;
        }

        // Strip single top-level directory if detected
        let relative = if let Some(ref prefix) = strip_prefix {
            if enclosed.starts_with(prefix) {
                enclosed.strip_prefix(prefix).unwrap().to_path_buf()
            } else {
                continue; // skip entries outside the prefix (e.g. __MACOSX)
            }
        } else {
            enclosed
        };

        // Skip empty paths (the top-level dir entry itself)
        if relative.as_os_str().is_empty() {
            continue;
        }

        let outpath = std::path::Path::new(dest_dir).join(&relative);

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| AppError::DockerError(format!("Create dir error: {}", e)))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)
                        .map_err(|e| AppError::DockerError(format!("Create dir error: {}", e)))?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| AppError::DockerError(format!("Create file error: {}", e)))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| AppError::DockerError(format!("Write file error: {}", e)))?;
        }
    }

    Ok(())
}

/// Find and parse skill.md (case-insensitive) in the extracted directory.
/// Returns (name, description) from YAML frontmatter if present.
fn read_skill_manifest(skill_dir: &str) -> Option<(String, Option<String>)> {
    let dir = std::path::Path::new(skill_dir);
    let entries = std::fs::read_dir(dir).ok()?;

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if name_str.eq_ignore_ascii_case("skill.md") {
            let content = std::fs::read_to_string(entry.path()).ok()?;
            return parse_frontmatter(&content);
        }
    }
    None
}

fn parse_frontmatter(content: &str) -> Option<(String, Option<String>)> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let yaml_str = &rest[..end];

    #[derive(serde::Deserialize)]
    struct Manifest {
        name: Option<String>,
        description: Option<String>,
    }

    let manifest: Manifest = serde_yaml::from_str(yaml_str).ok()?;
    let name = manifest.name?;
    Some((name, manifest.description))
}
