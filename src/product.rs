use abc_product::AbcProduct;
use bigdecimal::BigDecimal;
use gtin::Gtin;
use serde::{Deserialize, Serialize, Serializer};
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

fn parse_bigdecimal<'d, D>(s: String) -> Result<BigDecimal, D::Error>
where
    D: serde::Deserializer<'d>,
{
    BigDecimal::from_str(&s).map_err(serde::de::Error::custom)
}

fn deserialize_bigdecimal<'d, D>(deserializer: D) -> Result<BigDecimal, D::Error>
where
    D: serde::Deserializer<'d>,
{
    let s = String::deserialize(deserializer)?;
    parse_bigdecimal::<D>(s)
}

fn deserialize_optional_bigdecimal<'de, D>(deserializer: D) -> Result<Option<BigDecimal>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(d) => Ok(Some(parse_bigdecimal::<D>(d)?)),
        None => Ok(None),
    }
}

fn serialize_bigdecimal<S: Serializer>(dec: &BigDecimal, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&dec.to_plain_string())
}

fn serialize_optional_bigdecimal<S: Serializer>(
    dec: &Option<BigDecimal>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match dec {
        Some(d) => {
            let str = d.to_plain_string();
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
    pub list: Option<BigDecimal>,
    pub alt1: Option<String>,
    pub alt2: Option<String>,
    pub alt3: Option<String>,
}

/// Same information as [`ExportedProduct`], but adds other fields for alt skus, general ledger
/// number, discount group, and vendor code. Used exclusively for writing the new_products.csv file
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct NewProduct {
    pub sku: String,
    pub gtin: Gtin,
    pub desc: String,
    pub weight_lbs: Option<f64>,
    #[serde(
        deserialize_with = "deserialize_bigdecimal",
        serialize_with = "serialize_bigdecimal"
    )]
    pub cost: BigDecimal,
    #[serde(
        deserialize_with = "deserialize_optional_bigdecimal",
        serialize_with = "serialize_optional_bigdecimal"
    )]
    pub list: Option<BigDecimal>,
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
            weight_lbs: value.weight,
            cost: value.cost,
            list: value.list,
            alt1: value.alt1,
            alt2: value.alt2,
            alt3: value.alt3,
            gl: None,
            group: None,
            vendor: None,
        }
    }
}

/// Store the differences between an [`AbcProduct`] and an [`ExportedProduct`]
#[derive(Debug, Serialize)]
pub struct ProductDiff {
    old_sku: String,
    new_sku: String,
    old_gtin: Option<Gtin>,
    new_gtin: Option<Gtin>,
    old_weight: Option<f64>,
    new_weight: Option<f64>,
    old_cost: Option<BigDecimal>,
    new_cost: Option<BigDecimal>,
    old_list: Option<BigDecimal>,
    new_list: Option<BigDecimal>,
}

impl From<(ExportedProduct, AbcProduct)> for ProductDiff {
    fn from((exported_prod, abc_prod): (ExportedProduct, AbcProduct)) -> Self {
        ProductDiff::from((abc_prod, exported_prod))
    }
}

impl From<(AbcProduct, ExportedProduct)> for ProductDiff {
    fn from((abc_prod, exported_prod): (AbcProduct, ExportedProduct)) -> Self {
        let (old_sku, new_sku) = (abc_prod.sku(), exported_prod.sku);
        let (old_gtin, new_gtin) = if abc_prod.upcs().last() == Some(&exported_prod.gtin) {
            (None, None)
        } else {
            (abc_prod.upcs().last().cloned(), Some(exported_prod.gtin))
        };
        let (old_weight, new_weight) = if abc_prod.weight() == exported_prod.weight {
            (None, None)
        } else {
            (abc_prod.weight(), exported_prod.weight)
        };
        let (old_cost, new_cost) = if abc_prod.cost() == exported_prod.cost {
            (None, None)
        } else {
            (Some(abc_prod.cost()), Some(exported_prod.cost))
        };
        let (old_list, new_list) = if Some(abc_prod.list()) == exported_prod.list {
            (None, None)
        } else {
            (Some(abc_prod.list()), exported_prod.list)
        };
        Self {
            old_sku,
            new_sku,
            old_gtin,
            new_gtin,
            old_weight,
            new_weight,
            old_cost,
            new_cost,
            old_list,
            new_list,
        }
    }
}
