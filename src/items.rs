use std::ops::Div;

use rocket::{response::status::Custom, Route, State};
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::{types::PgMoney, PgRow}, prelude::FromRow, query_as, PgPool, Row};

use crate::{common::to_internal_server_err, templates::CommonPageContext};

pub(crate) fn routes() -> Vec<Route> {
    routes![index, item_page]
}

#[derive(Serialize)]
struct PageContext<'a> {
    title: &'a str
}

#[get("/")]
async fn index(
    common_context: CommonPageContext<'_>,
    pool: &State<PgPool>
) -> Result<Template, Custom<String>> {
    let page_context = PageContext {
        title: "Home"
    };
    Ok(Template::render("index", context! {
        common: common_context,
        page: page_context,
        items: ItemSummary::load_all( pool).await.map_err(to_internal_server_err)?
    }))
}

#[get("/item/<item_id>")]
async fn item_page(
    item_id: i64,
    common_context: CommonPageContext<'_>,
    pool: &State<PgPool>
) -> Result<Template, Custom<String>> {
    let item_summary = ItemSummary::load_by_id(item_id, pool)
        .await
        .map_err(to_internal_server_err)?;
    let images = ItemImage::load_by_item_id(item_id, pool)
        .await
        .map_err(to_internal_server_err)?;
    let page_context = PageContext {
        title: &item_summary.title
    };
    Ok(Template::render("item", context! {
        common: common_context,
        page: page_context,
        item: &item_summary,
        images: images
    }))

}

#[derive(Debug, Deserialize, FromRow, Serialize)]
struct Category {
    id: i32,
    name: String    
}

impl Category {
    async fn load_all(pool: &PgPool) -> Result<Vec<Category>, sqlx::Error> {
        query_as("SELECT * FROM category ORDER BY weighting DESC, name ASC")
            .fetch_all(pool)
            .await
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ItemSummary {
    id: i64,
    category: Category,
    title: String,
    price: f64,
    description: String,
    lead_image_url: Option<String>
}

impl FromRow<'_, PgRow> for ItemSummary {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            category: Category {
                id: row.try_get("category")?, 
                name: row.try_get("category_name")?
            },
            title: row.try_get("title")?,
            price: (row.try_get::<PgMoney, &str>("price")?.0 as f64) / 100.0,
            description: row.try_get("description").unwrap_or_else(|_| "".to_string()),
            lead_image_url: row.try_get("lead_image_url").ok()
        })
    }
}

impl ItemSummary {
    const BASE_QUERY: &str = "SELECT i.id, i.category, i.title, i.description, i.price, c.\"name\" AS category_name,
        (SELECT url FROM image WHERE image.item = i.id ORDER BY ordinal ASC LIMIT 1) AS lead_image_url
        FROM item AS i
        JOIN category AS c ON i.category = c.id";
    async fn load_all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        query_as(Self::BASE_QUERY)
            .fetch_all(pool)
            .await
    }
    async fn load_by_id(item_id: i64, pool: &PgPool) -> Result<Self, sqlx::Error> {
        let query = Self::BASE_QUERY.to_string() + " WHERE i.id = $1";
        query_as(&query)
            .bind(item_id)
            .fetch_one(pool)
            .await
    }
}

#[derive(Debug, Deserialize, FromRow, Serialize)]
struct ItemImage {
    id: i64,
    path: Option<String>,
    url: String,
    width: Option<i32>,
    height: Option<i32>
}

impl ItemImage {
    async fn load_by_item_id(item_id: i64, pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        query_as("SELECT * FROM image WHERE item = $1 ORDER BY ordinal ASC")
            .bind(item_id)
            .fetch_all(pool)
            .await
    }
}