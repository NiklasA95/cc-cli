use anyhow::{Context, Error, Result};
use serde::Deserialize;
use shopify_api::utils::ReadJsonTreeSteps;
use shopify_api::*;
use std::{collections::HashMap, path::PathBuf};

type GetReviewsResult = Result<(String, Vec<(String, String)>), Error>;

#[derive(Deserialize)]
struct QueryData {
    edges: Vec<OrderNode>,
}

#[derive(Deserialize)]
struct OrderNode {
    node: Order,
}

#[derive(Deserialize)]
struct Order {
    #[serde(rename(deserialize = "lineItems"))]
    line_items: LineItemEdges,
}

#[derive(Deserialize)]
struct LineItemEdges {
    edges: Vec<LineItemNode>,
}

#[derive(Deserialize, Clone)]
struct LineItemNode {
    node: LineItem,
}
#[derive(Deserialize, Clone)]
struct LineItem {
    sku: String,
    product: Product,
}

#[derive(Deserialize, Clone)]
struct Product {
    #[serde(rename(deserialize = "legacyResourceId"))]
    id: String,
}

pub async fn group_by_variant(file_path: &PathBuf) -> Result<(), Error> {
    let (product_id, reviews_with_order_number) = get_reviews_with_order_number(file_path)?;

    let shop_name = std::env::var("SHOP_NAME").with_context(|| ": SHOP_NAME")?;
    let api_key = std::env::var("API_KEY").with_context(|| ": API_KEY")?;

    let shopify_client = Shopify::new(&shop_name, &api_key, ShopifyAPIVersion::V2023_01, None);

    let mut variant_reviews: HashMap<String, Vec<String>> = HashMap::new();

    // TODO: Implement with futures and join
    for item in reviews_with_order_number {
        let line_items: Vec<LineItemNode> =
            get_line_items_for_order_number(&shopify_client, &item.0, &item.1)
                .await
                .into_iter()
                .filter(|item| item.node.product.id == product_id)
                .collect();

        let sku = &line_items[0].node.sku;

        if let Some(reviews) = variant_reviews.get_mut(sku) {
            reviews.push(item.0);
            continue;
        } else {
            let reviews: Vec<String> = vec![item.0];
            variant_reviews.insert(sku.to_string(), reviews);
        }
    }

    println!("\nReviews grouped by variant:\n\n{:?}", variant_reviews);

    Ok(())
}

/// Returns reviews that include an order number in a tuple of review id and order number
fn get_reviews_with_order_number(file_path: &PathBuf) -> GetReviewsResult {
    let mut reviews_with_order_number: Vec<(String, String)> = vec![];
    let mut reader = csv::Reader::from_path(file_path)
        .with_context(|| format!("could not read file `{}`", file_path.display()))?;
    let mut product_id = String::from("");

    for result in reader.records() {
        let record = result?;
        if product_id.is_empty() {
            product_id = String::from(&record[30]);
        }
        let review_id = String::from(&record[1]);
        let order_number = String::from(&record[22]);

        if !order_number.is_empty() {
            reviews_with_order_number.push((review_id, order_number))
        }
    }

    Ok((product_id, reviews_with_order_number))
}

/// Returns line items of the order which resulted in the provided review
async fn get_line_items_for_order_number(
    shopify_client: &Shopify,
    review_id: &String,
    order_number: &String,
) -> Vec<LineItemNode> {
    let order_name_query = format!("name:{}-QDO", order_number);
    let variables = serde_json::json!({ "order_name_query": order_name_query });
    let graphql_query = r#"
        query ($order_name_query: String) {
            orders(first: 1, query: $order_name_query) {
                edges {
                    node {
                        lineItems(first: 100) {
                            edges {
                                node {
                                    sku
                                    product {
                                        legacyResourceId
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    "#;

    let json_finder = vec![
        ReadJsonTreeSteps::Key("data"),
        ReadJsonTreeSteps::Key("orders"),
    ];

    let response: Result<QueryData, ShopifyAPIError> = shopify_client
        .graphql_query(graphql_query, &variables, &json_finder)
        .await;

    if let Err(err) = &response {
        eprintln!(
            "Fetching the line items for order {} which resulted in review {} failed ({:?})",
            order_number, review_id, err
        );
    }
    let data = response.unwrap();

    data.edges[0].node.line_items.edges.clone()
}
