use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use crate::error::AppError;
use crate::models::container::Container;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS containers (
                id TEXT PRIMARY KEY,
                task TEXT NOT NULL,
                status TEXT NOT NULL,
                docker_id TEXT,
                skill_repos TEXT NOT NULL DEFAULT '[]',
                cpu_limit TEXT NOT NULL DEFAULT '2',
                memory_limit TEXT NOT NULL DEFAULT '4Gi',
                idle_timeout INTEGER NOT NULL DEFAULT 300,
                max_lifetime INTEGER NOT NULL DEFAULT 3600,
                created_at TEXT NOT NULL,
                last_activity TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn create_container(&self, container: &Container) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO containers (id, task, status, docker_id, skill_repos, cpu_limit, memory_limit, idle_timeout, max_lifetime, created_at, last_activity)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&container.id)
        .bind(&container.task)
        .bind(&container.status)
        .bind(&container.docker_id)
        .bind(&container.skill_repos)
        .bind(&container.cpu_limit)
        .bind(&container.memory_limit)
        .bind(container.idle_timeout)
        .bind(container.max_lifetime)
        .bind(&container.created_at)
        .bind(&container.last_activity)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_container(&self, id: &str) -> Result<Container, AppError> {
        sqlx::query_as::<_, Container>(
            "SELECT id, task, status, docker_id, skill_repos, cpu_limit, memory_limit, idle_timeout, max_lifetime, created_at, last_activity FROM containers WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn update_status(&self, id: &str, status: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE containers SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn update_last_activity(&self, id: &str) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE containers SET last_activity = ? WHERE id = ?")
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn list_active_containers(&self) -> Result<Vec<Container>, AppError> {
        sqlx::query_as::<_, Container>(
            "SELECT id, task, status, docker_id, skill_repos, cpu_limit, memory_limit, idle_timeout, max_lifetime, created_at, last_activity FROM containers WHERE status IN ('Running', 'Idle')"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn delete_container(&self, id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM containers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn list_containers(
        &self,
        status: Option<&str>,
        search: Option<&str>,
        sort_by: &str,
        sort_order: &str,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<Container>, i64), AppError> {
        let allowed_sort = match sort_by {
            "created_at" | "last_activity" | "task" | "status" => sort_by,
            _ => "created_at",
        };
        let order = match sort_order {
            "asc" => "ASC",
            _ => "DESC",
        };

        let per_page = per_page.min(100).max(1);
        let page = page.max(1);
        let offset = (page - 1) * per_page;

        let mut conditions: Vec<String> = Vec::new();
        let mut binds: Vec<String> = Vec::new();

        if let Some(s) = status {
            if !s.is_empty() {
                conditions.push(format!("status = ?"));
                binds.push(s.to_string());
            }
        }

        if let Some(s) = search {
            if !s.is_empty() {
                let term = format!("%{}%", s);
                conditions.push(format!("(task LIKE ? OR id LIKE ?)"));
                binds.push(term.clone());
                binds.push(term);
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let order_clause = format!("ORDER BY {} {}", allowed_sort, order);
        let limit_clause = format!("LIMIT ? OFFSET ?");

        let data_query = format!(
            "SELECT id, task, status, docker_id, skill_repos, cpu_limit, memory_limit, \
             idle_timeout, max_lifetime, created_at, last_activity \
             FROM containers {} {} {}",
            where_clause, order_clause, limit_clause
        );

        let count_query = format!(
            "SELECT COUNT(*) FROM containers {}",
            where_clause
        );

        // Build data query with binds
        let mut data_query_builder = sqlx::query_as::<_, Container>(&data_query);
        let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);

        // Apply binds to both queries (same WHERE parameters, without LIMIT/OFFSET for count)
        for bind_val in &binds {
            data_query_builder = data_query_builder.bind(bind_val);
            count_query_builder = count_query_builder.bind(bind_val);
        }
        data_query_builder = data_query_builder.bind(per_page).bind(offset);

        let data: Vec<Container> = data_query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let total: i64 = count_query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok((data, total))
    }

    pub async fn get_stats(&self) -> Result<crate::models::container::StatsResponse, AppError> {
        let status_counts: Vec<crate::models::container::StatusCount> = sqlx::query_as(
            "SELECT status, COUNT(*) as count FROM containers GROUP BY status"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let by_status: std::collections::HashMap<String, i64> = status_counts
            .into_iter()
            .map(|sc| (sc.status, sc.count))
            .collect();

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM containers")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(crate::models::container::StatsResponse { total, by_status })
    }

    pub async fn get_config(&self, key: &str) -> Result<Option<String>, AppError> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT value FROM config WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| r.0))
    }

    pub async fn set_config(&self, key: &str, value: &str) -> Result<(), AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO config (key, value, updated_at) VALUES (?, ?, ?) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(value)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_db() -> Database {
        Database::new("sqlite::memory:").await.unwrap()
    }

    fn test_container(id: &str) -> Container {
        let now = chrono::Utc::now().to_rfc3339();
        Container {
            id: id.to_string(),
            task: "test task".to_string(),
            status: "Running".to_string(),
            docker_id: Some("test-docker-id".to_string()),
            skill_repos: "[]".to_string(),
            cpu_limit: "1".to_string(),
            memory_limit: "1Gi".to_string(),
            idle_timeout: 300,
            max_lifetime: 3600,
            created_at: now.clone(),
            last_activity: now,
        }
    }

    #[tokio::test]
    async fn test_create_and_get_container() {
        let db = test_db().await;
        let container = test_container("test-1");

        db.create_container(&container).await.unwrap();
        let fetched = db.get_container("test-1").await.unwrap();

        assert_eq!(fetched.id, "test-1");
        assert_eq!(fetched.task, "test task");
        assert_eq!(fetched.status, "Running");
    }

    #[tokio::test]
    async fn test_update_status() {
        let db = test_db().await;
        let container = test_container("test-2");

        db.create_container(&container).await.unwrap();
        db.update_status("test-2", "Stopped").await.unwrap();

        let fetched = db.get_container("test-2").await.unwrap();
        assert_eq!(fetched.status, "Stopped");
    }

    #[tokio::test]
    async fn test_list_active_containers() {
        let db = test_db().await;
        let now = chrono::Utc::now().to_rfc3339();

        let c1 = Container {
            id: "test-3".to_string(),
            task: "task".to_string(),
            status: "Running".to_string(),
            docker_id: None,
            skill_repos: "[]".to_string(),
            cpu_limit: "1".to_string(),
            memory_limit: "1Gi".to_string(),
            idle_timeout: 300,
            max_lifetime: 3600,
            created_at: now.clone(),
            last_activity: now.clone(),
        };
        db.create_container(&c1).await.unwrap();

        let c2 = Container {
            id: "test-4".to_string(),
            task: "task".to_string(),
            status: "Stopped".to_string(),
            docker_id: None,
            skill_repos: "[]".to_string(),
            cpu_limit: "1".to_string(),
            memory_limit: "1Gi".to_string(),
            idle_timeout: 300,
            max_lifetime: 3600,
            created_at: now.clone(),
            last_activity: now,
        };
        db.create_container(&c2).await.unwrap();

        let active = db.list_active_containers().await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "test-3");
    }

    #[tokio::test]
    async fn test_delete_container() {
        let db = test_db().await;
        let container = test_container("test-5");

        db.create_container(&container).await.unwrap();
        db.delete_container("test-5").await.unwrap();

        let result = db.get_container("test-5").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_containers_empty() {
        let db = test_db().await;
        let (data, total) = db
            .list_containers(None, None, "created_at", "desc", 1, 20)
            .await
            .unwrap();
        assert_eq!(total, 0);
        assert!(data.is_empty());
    }

    #[tokio::test]
    async fn test_list_containers_with_data() {
        let db = test_db().await;
        let c1 = test_container("test-l1");
        let c2 = test_container("test-l2");
        db.create_container(&c1).await.unwrap();
        db.create_container(&c2).await.unwrap();

        let (data, total) = db
            .list_containers(None, None, "created_at", "desc", 1, 20)
            .await
            .unwrap();
        assert_eq!(total, 2);
        assert_eq!(data.len(), 2);
    }

    #[tokio::test]
    async fn test_list_containers_status_filter() {
        let db = test_db().await;
        let now = chrono::Utc::now().to_rfc3339();
        let running = Container {
            id: "test-sf-running".to_string(),
            task: "task".to_string(),
            status: "Running".to_string(),
            docker_id: None,
            skill_repos: "[]".to_string(),
            cpu_limit: "1".to_string(),
            memory_limit: "1Gi".to_string(),
            idle_timeout: 300,
            max_lifetime: 3600,
            created_at: now.clone(),
            last_activity: now.clone(),
        };
        let stopped = Container {
            id: "test-sf-stopped".to_string(),
            status: "Stopped".to_string(),
            ..running.clone()
        };
        db.create_container(&running).await.unwrap();
        db.create_container(&stopped).await.unwrap();

        let (data, total) = db
            .list_containers(Some("Running"), None, "created_at", "desc", 1, 20)
            .await
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(data[0].id, "test-sf-running");
    }

    #[tokio::test]
    async fn test_list_containers_search() {
        let db = test_db().await;
        let now = chrono::Utc::now().to_rfc3339();
        let c = Container {
            id: "test-search-1".to_string(),
            task: "review PR #42".to_string(),
            status: "Running".to_string(),
            docker_id: None,
            skill_repos: "[]".to_string(),
            cpu_limit: "1".to_string(),
            memory_limit: "1Gi".to_string(),
            idle_timeout: 300,
            max_lifetime: 3600,
            created_at: now.clone(),
            last_activity: now,
        };
        db.create_container(&c).await.unwrap();

        let (_data, total) = db
            .list_containers(None, Some("PR #42"), "created_at", "desc", 1, 20)
            .await
            .unwrap();
        assert_eq!(total, 1);

        let (_data, total) = db
            .list_containers(None, Some("nonexistent"), "created_at", "desc", 1, 20)
            .await
            .unwrap();
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_list_containers_pagination() {
        let db = test_db().await;
        let now = chrono::Utc::now().to_rfc3339();
        for i in 0..5 {
            let c = Container {
                id: format!("test-pg-{}", i),
                task: "task".to_string(),
                status: "Running".to_string(),
                docker_id: None,
                skill_repos: "[]".to_string(),
                cpu_limit: "1".to_string(),
                memory_limit: "1Gi".to_string(),
                idle_timeout: 300,
                max_lifetime: 3600,
                created_at: now.clone(),
                last_activity: now.clone(),
            };
            db.create_container(&c).await.unwrap();
        }

        let (data, total) = db
            .list_containers(None, None, "created_at", "desc", 1, 2)
            .await
            .unwrap();
        assert_eq!(total, 5);
        assert_eq!(data.len(), 2);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let db = test_db().await;
        let now = chrono::Utc::now().to_rfc3339();

        let running = Container {
            id: "test-stats-1".to_string(),
            task: "task".to_string(),
            status: "Running".to_string(),
            docker_id: None,
            skill_repos: "[]".to_string(),
            cpu_limit: "1".to_string(),
            memory_limit: "1Gi".to_string(),
            idle_timeout: 300,
            max_lifetime: 3600,
            created_at: now.clone(),
            last_activity: now.clone(),
        };
        let stopped = Container {
            id: "test-stats-2".to_string(),
            status: "Stopped".to_string(),
            ..running.clone()
        };
        db.create_container(&running).await.unwrap();
        db.create_container(&stopped).await.unwrap();

        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.by_status.get("Running"), Some(&1));
        assert_eq!(stats.by_status.get("Stopped"), Some(&1));
    }
}
