use abc_product::AbcProduct;
use bigdecimal::BigDecimal;
use gtin::Gtin;
use serde::{Deserialize, Serializer};
use std::{collections::HashMap, str::FromStr};

pub type DuplicateProducts = Vec<AbcProduct>;

pub fn map_upcs(
    existing_map: &HashMap<String, AbcProduct>,
) -> HashMap<Gtin, (DuplicateProducts, AbcProduct)> {
    let mut upc_map = HashMap::new();
    for product in existing_map.values() {
        for upc in product.upcs().iter() {
            if let Some((dup, prod)) = upc_map.insert(*upc, (Vec::new(), product.to_owned())) {
                let mut dup = dup;
                if product.sku() != prod.sku() {
                    dup.push(product.to_owned());
                    dup.push(prod.clone());
                }
                upc_map.insert(*upc, (dup, prod));
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

fn serialize_bigdecimal<S: Serializer>(dec: &BigDecimal, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(
        &dec.with_scale_round(2, bigdecimal::RoundingMode::HalfEven)
            .to_plain_string(),
    )
}

fn serialize_optional_bigdecimal<S: Serializer>(
    dec: &Option<BigDecimal>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match dec {
        Some(d) => {
            let str = d
                .with_scale_round(2, bigdecimal::RoundingMode::HalfEven)
                .to_plain_string();
            s.serialize_some(&str)
        }
        None => s.serialize_none(),
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct ExportedProduct {
    pub sku: String,
    pub gtin: Gtin,
    pub desc: String,
    pub weight: Option<f64>,
    #[serde(
        deserialize_with = "deserialize_bigdecimal",
        serialize_with = "serialize_bigdecimal"
    )]
    pub cost: BigDecimal,
    #[serde(
        deserialize_with = "deserialize_optional_bigdecimal",
        serialize_with = "serialize_optional_bigdecimal"
    )]
    pub retail: Option<BigDecimal>,
}

/// Same information as [`ExportedProduct`], but adds other fields for alt skus, general ledger
/// number, discount group, and vendor code. Used exclusively for writing the new_products.csv file
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct NewProduct {
    pub sku: String,
    pub gtin: Gtin,
    pub desc: String,
    pub weight: Option<f64>,
    #[serde(
        deserialize_with = "deserialize_bigdecimal",
        serialize_with = "serialize_bigdecimal"
    )]
    pub cost: BigDecimal,
    #[serde(
        deserialize_with = "deserialize_optional_bigdecimal",
        serialize_with = "serialize_optional_bigdecimal"
    )]
    pub retail: Option<BigDecimal>,
    pub alt1: Option<String>,
    pub alt2: Option<String>,
    pub alt3: Option<String>,
    pub gl: Option<String>,
    pub group: Option<String>,
    pub vendor: Option<String>,
}

impl From<ExportedProduct> for NewProduct {
    fn from(value: ExportedProduct) -> Self {
        Self {
            sku: value.sku,
            gtin: value.gtin,
            desc: value.desc,
            weight: value.weight,
            cost: value.cost,
            retail: value.retail,
            alt1: None,
            alt2: None,
            alt3: None,
            gl: None,
            group: None,
            vendor: None,
        }
    }
}
