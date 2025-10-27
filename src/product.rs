use abc_product::AbcProduct;
use ean13::Ean13;
use rust_decimal::Decimal;
use std::collections::HashMap;

pub type DuplicateProducts = Vec<AbcProduct>;

pub fn map_upcs(
    existing_map: &HashMap<String, AbcProduct>,
) -> HashMap<Ean13, (DuplicateProducts, AbcProduct)> {
    let mut upc_map = HashMap::new();
    for (_sku, product) in existing_map {
        for upc in product.upcs().iter() {
            if let Some((dup, prod)) = upc_map.insert(upc.clone(), (Vec::new(), product.to_owned()))
            {
                let mut dup = dup;
                if product.sku() != prod.sku() {
                    dup.push(product.to_owned());
                    dup.push(prod.clone());
                }
                upc_map.insert(upc.clone(), (dup, prod));
            }
        }
    }
    upc_map
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct ExportedProduct {
    pub sku: String,
    pub upc: Ean13,
    pub desc: String,
    pub weight: Option<f64>,
    pub cost: Decimal,
    pub retail: Option<Decimal>,
}
