use abc_product::AbcProduct;
use bigdecimal::BigDecimal;
use gtin::Gtin;
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr};

pub type DuplicateProducts = Vec<AbcProduct>;

pub fn map_upcs(
    existing_map: &HashMap<String, AbcProduct>,
) -> HashMap<Gtin, (DuplicateProducts, AbcProduct)> {
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

fn parse_bigdecimal_price<'d, D>(s: String) -> Result<BigDecimal, D::Error>
where
    D: serde::Deserializer<'d>,
{
    Ok(BigDecimal::from_str(&s)
        .map_err(serde::de::Error::custom)?
        .with_scale_round(2, bigdecimal::RoundingMode::HalfEven))
}

fn deserialize_bigdecimal<'d, D>(deserializer: D) -> Result<BigDecimal, D::Error>
where
    D: serde::Deserializer<'d>,
{
    let s = String::deserialize(deserializer)?;
    parse_bigdecimal_price::<D>(s)
}

fn deserialize_optional_bigdecimal<'de, D>(deserializer: D) -> Result<Option<BigDecimal>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(d) => Ok(Some(parse_bigdecimal_price::<D>(d)?)),
        None => Ok(None),
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct ExportedProduct {
    pub sku: String,
    pub upc: Gtin,
    pub desc: String,
    pub weight: Option<f64>,
    #[serde(deserialize_with = "deserialize_bigdecimal")]
    pub cost: BigDecimal,
    #[serde(deserialize_with = "deserialize_optional_bigdecimal")]
    pub retail: Option<BigDecimal>,
}
