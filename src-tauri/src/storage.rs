//! SQLite schema management, catalog loading, and generated content cache helpers.

use crate::generation::now_millis;
use crate::i18n::{backend_messages, translate};
use crate::models::{ArticleDto, CatalogResponse, DomainDto, SectionDto, VisitedArticleDto};
use crate::paths::{ensure_storage, RuntimePaths};
use crate::seeds::{ARTICLES, DOMAINS, SECTIONS};
use rusqlite::{params, Connection, OptionalExtension};

/// Opens the local SQLite database and idempotently applies schema + seed data.
pub(crate) fn open_database(paths: &RuntimePaths) -> Result<Connection, String> {
  ensure_storage(paths)?;
  log::info!("Opening SQLite database at '{}'", paths.database_path.display());
  let connection = Connection::open(&paths.database_path).map_err(|error| error.to_string())?;
  initialize_database(&connection)?;
  Ok(connection)
}

/// Normalizes frontend locales to the two supported catalog languages.
pub(crate) fn normalize_locale(locale: Option<&str>) -> &'static str {
  match locale.unwrap_or("fr").to_lowercase().as_str() {
    "en" | "en-us" | "en-gb" => "en",
    _ => "fr",
  }
}

pub(crate) fn load_catalog(connection: &Connection, locale: &str) -> Result<CatalogResponse, String> {
  log::info!("Loading catalog from SQLite for locale='{locale}'");
  let mut statement = connection
    .prepare("SELECT id, icon, color, name_fr, name_en, welcome_fr, welcome_en FROM domains ORDER BY rowid")
    .map_err(|error| error.to_string())?;
  let rows = statement
    .query_map([], |row| {
      let id: String = row.get(0)?;
      Ok((
        id,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        row.get::<_, String>(3)?,
        row.get::<_, String>(4)?,
        row.get::<_, String>(5)?,
        row.get::<_, String>(6)?,
      ))
    })
    .map_err(|error| error.to_string())?;

  let mut domains = Vec::new();
  for row in rows {
    let (id, icon, color, name_fr, name_en, welcome_fr, welcome_en) = row.map_err(|error| error.to_string())?;
    domains.push(DomainDto {
      sections: load_sections(connection, &id, locale)?,
      id,
      icon,
      color,
      name: localized(name_fr, name_en, locale),
      welcome: localized(welcome_fr, welcome_en, locale),
    });
  }

  Ok(CatalogResponse { domains })
}

pub(crate) fn load_domain_sections(connection: &Connection, domain_id: &str, locale: &str) -> Result<Vec<SectionDto>, String> {
  let sections = load_sections(connection, domain_id, locale)?;
  if sections.is_empty() {
    Err(translate(
      &backend_messages(locale).errors.unknown_domain,
      &[("domain_id", domain_id)],
    ))
  } else {
    Ok(sections)
  }
}

pub(crate) fn load_article(connection: &Connection, article_id: &str, locale: &str) -> Result<ArticleDto, String> {
  log::info!("Loading article from SQLite: article_id='{article_id}', locale='{locale}'");
  connection
    .query_row(
      "SELECT id, title_fr, title_en, summary_fr, summary_en, questions_fr, questions_en FROM articles WHERE id = ?1",
      params![article_id],
      |row| {
        let questions_fr: String = row.get(5)?;
        let questions_en: String = row.get(6)?;
        Ok(ArticleDto {
          id: row.get(0)?,
          title: localized(row.get(1)?, row.get(2)?, locale),
          summary: localized(row.get(3)?, row.get(4)?, locale),
          questions: localized(questions_fr, questions_en, locale)
            .lines()
            .map(|line| line.to_string())
            .collect(),
        })
      },
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| {
      translate(
        &backend_messages(locale).errors.unknown_article,
        &[("article_id", article_id)],
      )
    })
}

pub(crate) fn read_cache(connection: &Connection, key: &str) -> Result<Option<String>, String> {
  let value = connection
    .query_row("SELECT value FROM cache_entries WHERE key = ?1", params![key], |row| row.get(0))
    .optional()
    .map_err(|error| error.to_string())?;
  log::info!("Cache {} for key='{key}'", if value.is_some() { "hit" } else { "miss" });
  Ok(value)
}

pub(crate) fn write_cache(connection: &Connection, key: &str, value: &str) -> Result<(), String> {
  connection
    .execute(
      "INSERT OR REPLACE INTO cache_entries (key, value, created_at) VALUES (?1, ?2, ?3)",
      params![key, value, now_millis() as i64],
    )
    .map_err(|error| error.to_string())?;
  log::info!("Cache entry written for key='{key}'");
  Ok(())
}

pub(crate) fn clear_cache(connection: &Connection) -> Result<usize, String> {
  let deleted = connection
    .execute("DELETE FROM cache_entries", [])
    .map_err(|error| error.to_string())?;
  log::info!("Cleared {deleted} cache entries");
  Ok(deleted)
}

pub(crate) fn record_article_visit(connection: &Connection, article_id: &str) -> Result<(), String> {
  let latest_visit = connection
    .query_row("SELECT MAX(visited_at) FROM article_visits", [], |row| row.get::<_, Option<i64>>(0))
    .map_err(|error| error.to_string())?
    .unwrap_or_default();
  let visited_at = (now_millis() as i64).max(latest_visit + 1);
  connection
    .execute(
      "INSERT INTO article_visits (article_id, visited_at)
      VALUES (?1, ?2)
      ON CONFLICT(article_id) DO UPDATE SET visited_at = excluded.visited_at",
      params![article_id, visited_at],
    )
    .map_err(|error| error.to_string())?;
  log::info!("Recorded article visit: article_id='{article_id}'");
  Ok(())
}

pub(crate) fn load_visited_articles(connection: &Connection, locale: &str, limit: usize) -> Result<Vec<VisitedArticleDto>, String> {
  let mut statement = connection
    .prepare(
      "SELECT articles.id, articles.title_fr, articles.title_en, articles.summary_fr, articles.summary_en,
        articles.questions_fr, articles.questions_en, article_visits.visited_at
      FROM article_visits
      INNER JOIN articles ON articles.id = article_visits.article_id
      ORDER BY article_visits.visited_at DESC
      LIMIT ?1",
    )
    .map_err(|error| error.to_string())?;
  let rows = statement
    .query_map(params![limit as i64], |row| {
      let questions_fr: String = row.get(5)?;
      let questions_en: String = row.get(6)?;
      Ok(VisitedArticleDto {
        article: ArticleDto {
          id: row.get(0)?,
          title: localized(row.get(1)?, row.get(2)?, locale),
          summary: localized(row.get(3)?, row.get(4)?, locale),
          questions: localized(questions_fr, questions_en, locale)
            .lines()
            .map(|line| line.to_string())
            .collect(),
        },
        visited_at: row.get(7)?,
      })
    })
    .map_err(|error| error.to_string())?;

  let mut visits = Vec::new();
  for row in rows {
    visits.push(row.map_err(|error| error.to_string())?);
  }
  Ok(visits)
}

fn initialize_database(connection: &Connection) -> Result<(), String> {
  log::info!("Ensuring SQLite schema and seed data are available");
  connection
    .execute_batch(
      "CREATE TABLE IF NOT EXISTS domains (
        id TEXT PRIMARY KEY,
        icon TEXT NOT NULL,
        color TEXT NOT NULL,
        name_fr TEXT NOT NULL,
        name_en TEXT NOT NULL,
        welcome_fr TEXT NOT NULL,
        welcome_en TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS sections (
        id TEXT NOT NULL,
        domain_id TEXT NOT NULL,
        article_id TEXT NOT NULL,
        icon TEXT NOT NULL,
        name_fr TEXT NOT NULL,
        name_en TEXT NOT NULL,
        position INTEGER NOT NULL,
        PRIMARY KEY (id, domain_id)
      );
      CREATE TABLE IF NOT EXISTS articles (
        id TEXT PRIMARY KEY,
        title_fr TEXT NOT NULL,
        title_en TEXT NOT NULL,
        summary_fr TEXT NOT NULL,
        summary_en TEXT NOT NULL,
        questions_fr TEXT NOT NULL,
        questions_en TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS cache_entries (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        created_at INTEGER NOT NULL
      );
      CREATE TABLE IF NOT EXISTS article_visits (
        article_id TEXT PRIMARY KEY,
        visited_at INTEGER NOT NULL,
        FOREIGN KEY(article_id) REFERENCES articles(id)
      );",
    )
    .map_err(|error| error.to_string())?;

  seed_database(connection)
}

fn seed_database(connection: &Connection) -> Result<(), String> {
  log::info!("Seeding deterministic catalog data");
  for domain in DOMAINS {
    connection
      .execute(
        "INSERT OR REPLACE INTO domains (id, icon, color, name_fr, name_en, welcome_fr, welcome_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![domain.id, domain.icon, domain.color, domain.name_fr, domain.name_en, domain.welcome_fr, domain.welcome_en],
      )
      .map_err(|error| error.to_string())?;
  }

  for (position, section) in SECTIONS.iter().enumerate() {
    connection
      .execute(
        "INSERT OR REPLACE INTO sections (id, domain_id, article_id, icon, name_fr, name_en, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![section.id, section.domain_id, section.article_id, section.icon, section.name_fr, section.name_en, position as i64],
      )
      .map_err(|error| error.to_string())?;
  }

  for article in ARTICLES {
    connection
      .execute(
        "INSERT OR REPLACE INTO articles (id, title_fr, title_en, summary_fr, summary_en, questions_fr, questions_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
          article.id,
          article.title_fr,
          article.title_en,
          article.summary_fr,
          article.summary_en,
          article.questions_fr.join("\n"),
          article.questions_en.join("\n"),
        ],
      )
      .map_err(|error| error.to_string())?;
  }

  Ok(())
}

fn load_sections(connection: &Connection, domain_id: &str, locale: &str) -> Result<Vec<SectionDto>, String> {
  let mut statement = connection
    .prepare("SELECT id, article_id, icon, name_fr, name_en FROM sections WHERE domain_id = ?1 ORDER BY position")
    .map_err(|error| error.to_string())?;
  let rows = statement
    .query_map(params![domain_id], |row| {
      Ok((
        row.get::<_, String>(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        row.get::<_, String>(3)?,
        row.get::<_, String>(4)?,
      ))
    })
    .map_err(|error| error.to_string())?;

  let mut sections = Vec::new();
  for row in rows {
    let (id, article_id, icon, name_fr, name_en) = row.map_err(|error| error.to_string())?;
    sections.push(SectionDto {
      id,
      article_id,
      icon,
      name: localized(name_fr, name_en, locale),
    });
  }

  Ok(sections)
}

fn localized(fr: String, en: String, locale: &str) -> String {
  if locale == "en" { en } else { fr }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn clear_cache_removes_generated_entries() {
    let connection = Connection::open_in_memory().expect("open in-memory database");
    initialize_database(&connection).expect("initialize database");
    write_cache(&connection, "text:fr:arctic-fox", "cached text").expect("write text cache");
    write_cache(&connection, "image:fr:arctic-fox", "/tmp/image.svg").expect("write image cache");

    let deleted = clear_cache(&connection).expect("clear cache");

    assert_eq!(deleted, 2);
    assert!(read_cache(&connection, "text:fr:arctic-fox").expect("read text cache").is_none());
    assert!(read_cache(&connection, "image:fr:arctic-fox").expect("read image cache").is_none());
  }

  #[test]
  fn visited_articles_are_recent_unique_and_not_cache_entries() {
    let connection = Connection::open_in_memory().expect("open in-memory database");
    initialize_database(&connection).expect("initialize database");
    write_cache(&connection, "text:fr:arctic-fox", "cached text").expect("write text cache");

    record_article_visit(&connection, "arctic-fox").expect("record first visit");
    record_article_visit(&connection, "coral-reef").expect("record second visit");
    record_article_visit(&connection, "arctic-fox").expect("record updated visit");
    let deleted = clear_cache(&connection).expect("clear cache");
    let visits = load_visited_articles(&connection, "fr", 10).expect("load visits");

    assert_eq!(deleted, 1);
    assert_eq!(visits.len(), 2);
    assert_eq!(visits[0].article.id, "arctic-fox");
    assert_eq!(visits[1].article.id, "coral-reef");
  }
}