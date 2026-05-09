//! SQLite schema management, catalog loading, and generated content cache helpers.

use crate::generation::now_millis;
use crate::models::{ArticleDto, CatalogResponse, DomainDto, SectionDto};
use crate::paths::{ensure_storage, RuntimePaths};
use crate::seeds::{ARTICLES, DOMAINS, SECTIONS};
use rusqlite::{params, Connection, OptionalExtension};

/// Opens the local SQLite database and idempotently applies schema + seed data.
pub(crate) fn open_database(paths: &RuntimePaths) -> Result<Connection, String> {
  ensure_storage(paths)?;
  let connection = Connection::open(&paths.database_path).map_err(|error| error.to_string())?;
  initialize_database(&connection)?;
  Ok(connection)
}

pub(crate) fn normalize_locale(locale: Option<&str>) -> &'static str {
  match locale.unwrap_or("fr").to_lowercase().as_str() {
    "en" | "en-us" | "en-gb" => "en",
    _ => "fr",
  }
}

pub(crate) fn load_catalog(connection: &Connection, locale: &str) -> Result<CatalogResponse, String> {
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

pub(crate) fn load_article(connection: &Connection, article_id: &str, locale: &str) -> Result<ArticleDto, String> {
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
    .ok_or_else(|| format!("Unknown article: {article_id}"))
}

pub(crate) fn read_cache(connection: &Connection, key: &str) -> Result<Option<String>, String> {
  connection
    .query_row("SELECT value FROM cache_entries WHERE key = ?1", params![key], |row| row.get(0))
    .optional()
    .map_err(|error| error.to_string())
}

pub(crate) fn write_cache(connection: &Connection, key: &str, value: &str) -> Result<(), String> {
  connection
    .execute(
      "INSERT OR REPLACE INTO cache_entries (key, value, created_at) VALUES (?1, ?2, ?3)",
      params![key, value, now_millis() as i64],
    )
    .map_err(|error| error.to_string())?;
  Ok(())
}

fn initialize_database(connection: &Connection) -> Result<(), String> {
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
      );",
    )
    .map_err(|error| error.to_string())?;

  seed_database(connection)
}

fn seed_database(connection: &Connection) -> Result<(), String> {
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