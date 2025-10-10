use ean13::Ean13;
use rust_decimal::Decimal;
use std::collections::HashMap;

fn price_from_str(price_str: &str) -> Result<Decimal, rust_decimal::Error> {
    let price_str: String = price_str
        .chars()
        .filter(|c| c.is_digit(10) || c == &'.')
        .collect();
    Decimal::from_str_exact(&price_str)
}

#[derive(Debug, Clone)]
pub struct AbcProduct {
    sku: String,
    desc: String,
    upcs: Vec<Ean13>,
    list: Decimal,
    cost: Decimal,
    stock: f64,
    weight: f64,
    last_sold: Option<chrono::NaiveDate>,
}

impl AbcProduct {
    pub fn sku(&self) -> String {
        self.sku.clone()
    }

    pub fn desc(&self) -> String {
        self.desc.clone()
    }

    pub fn upcs(&self) -> Vec<Ean13> {
        self.upcs.to_vec()
    }

    pub fn list(&self) -> Decimal {
        self.list
    }

    pub fn cost(&self) -> Decimal {
        self.cost
    }

    pub fn stock(&self) -> f64 {
        self.stock
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn last_sold(&self) -> Option<chrono::NaiveDate> {
        self.last_sold
    }
}

pub struct AbcProductBuilder {
    sku: Option<String>,
    desc: Option<String>,
    upcs: Vec<Ean13>,
    list: Option<Decimal>,
    cost: Option<Decimal>,
    stock: Option<f64>,
    weight: Option<f64>,
    last_sold: Option<chrono::NaiveDate>,
}

impl AbcProductBuilder {
    pub fn new() -> Self {
        AbcProductBuilder {
            sku: None,
            desc: None,
            upcs: Vec::new(),
            list: None,
            cost: None,
            stock: None,
            weight: None,
            last_sold: None,
        }
    }

    pub fn with_sku(self, sku: &str) -> Self {
        AbcProductBuilder {
            sku: Some(sku.to_string()),
            ..self
        }
    }

    pub fn with_desc(self, desc: &str) -> Self {
        AbcProductBuilder {
            desc: Some(desc.to_string()),
            ..self
        }
    }

    pub fn with_upcs(self, upcs: Vec<Ean13>) -> Self {
        AbcProductBuilder { upcs, ..self }
    }

    pub fn add_upc(self, upc: Ean13) -> Self {
        let mut new_upcs = self.upcs.to_vec();
        new_upcs.push(upc);
        AbcProductBuilder {
            upcs: new_upcs,
            ..self
        }
    }

    pub fn with_list(self, list: Decimal) -> Self {
        AbcProductBuilder {
            list: Some(list),
            ..self
        }
    }

    pub fn with_cost(self, cost: Decimal) -> Self {
        AbcProductBuilder {
            cost: Some(cost),
            ..self
        }
    }

    pub fn with_stock(self, stock: f64) -> Self {
        AbcProductBuilder {
            stock: Some(stock),
            ..self
        }
    }

    pub fn with_weight(self, weight: f64) -> Self {
        AbcProductBuilder {
            weight: Some(weight),
            ..self
        }
    }

    pub fn build(self) -> Option<AbcProduct> {
        Some(AbcProduct {
            sku: self.sku.clone()?,
            desc: self.desc.clone()?,
            upcs: self.upcs,
            list: self.list?,
            cost: self.cost?,
            stock: self.stock?,
            weight: self.weight?,
            last_sold: self.last_sold,
        })
    }
}

pub type AbcProductsBySku = HashMap<String, AbcProduct>;

pub fn parse_abc_item_files(
    item_path: &str,
    posted_path: &str,
) -> Result<AbcProductsBySku, AbcParseError> {
    let mut item_data = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(item_path)?;
    let mut posted_data = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(posted_path)?;

    let mut i = 0;
    let mut products = HashMap::new();
    while let Some(row) = item_data.records().next() {
        i += 1;
        let row = row?;
        let sku = row
            .get(0)
            .ok_or(AbcParseError::Custom(format!(
                "Cannot deserialize sku in row {}",
                i
            )))?
            .to_string();
        let desc = row
            .get(1)
            .ok_or(AbcParseError::Custom(format!(
                "Cannot deserialize desc in row {}",
                i
            )))?
            .to_string();
        let upc_str: String = row
            .get(43)
            .ok_or(AbcParseError::Custom(format!(
                "Cannot fetch upcs in row {}",
                i
            )))?
            .chars()
            .filter(|c| c.is_digit(10) || *c == ',')
            .collect();
        let upcs: Vec<Ean13> = upc_str
            .split(",")
            .filter_map(|s| {
                if s.len() == 11 {
                    // Some ABC UPCs leave out the check digit, so make one up and let [`Ean13::from_str_nonstrict`] fix it
                    Ean13::from_str_nonstrict(&format!("{}0", s)).ok()
                } else if s.len() < 11 {
                    // Anything less than 11 characters long is probably a dead upc
                    None
                } else {
                    // Anything 12 characters and up has a chance of being a good upc
                    Ean13::from_str_nonstrict(s).ok()
                }
            })
            .collect();
        let list = row.get(6).ok_or(AbcParseError::Custom(format!(
            "Cannot fetch list price from row {}",
            i
        )))?;
        let list = price_from_str(list).or(Err(AbcParseError::Custom(format!(
            "Cannot parse a price in cents for list in row {}",
            i
        ))))?;
        let cost = row.get(8).ok_or(AbcParseError::Custom(format!(
            "Cannot fetch cost from row {}",
            i
        )))?;
        let cost = price_from_str(cost).or(Err(AbcParseError::Custom(format!(
            "Cannot parse a price in cents for cost in row {}",
            i
        ))))?;
        let weight: f64 = row
            .get(45)
            .ok_or(AbcParseError::Custom(format!(
                "Cannot fetch weight from row {}",
                i
            )))?
            .parse()
            .or(Err(AbcParseError::Custom(format!(
                "Failed to parse f64 from weight in row {}",
                i
            ))))?;

        products.insert(
            sku.clone(),
            AbcProduct {
                sku,
                desc,
                upcs,
                list,
                cost,
                weight,
                stock: 0.0,
                last_sold: None,
            },
        );
    }

    let mut i = 0;
    while let Some(row) = posted_data.records().next() {
        i += 1;
        let row = row?;
        let sku = row
            .get(0)
            .ok_or(AbcParseError::Custom(format!(
                "Cannot deserialize sku in row {} of posted items",
                i
            )))?
            .to_string();
        let stock_str = row
            .get(19)
            .ok_or(AbcParseError::Custom(format!(
                "Cannot deserialize stock in row {} of posted items",
                i
            )))?
            .to_string();
        let stock: f64 = stock_str.parse().or(Err(AbcParseError::Custom(format!(
            "Cannot parse f64 from stock_str in row {} of posted items",
            i
        ))))?;
        let last_sold_str: String = row
            .get(1)
            .ok_or(AbcParseError::Custom(format!(
                "Cannot deserialize last_sold in row {} of posted items",
                i
            )))?
            .to_string();
        let last_sold = chrono::NaiveDate::parse_from_str(&last_sold_str, "%Y-%m-%d").ok();
        let mut existing_record = products
            .get(&sku)
            .ok_or(AbcParseError::Custom(format!(
                "Cannot find existing product for item with sku {} in row {} of posted_data",
                &sku, i
            )))?
            .clone();
        existing_record.stock = stock;
        existing_record.sku = existing_record.sku.to_uppercase();
        existing_record.last_sold = last_sold;
        products.insert(sku, existing_record);
    }
    Ok(products)
}

pub type DuplicateProducts = Vec<AbcProduct>;

pub fn map_upcs(
    existing_map: &HashMap<String, AbcProduct>,
) -> HashMap<Ean13, (DuplicateProducts, AbcProduct)> {
    let mut upc_map = HashMap::new();
    for (_sku, product) in existing_map {
        for upc in product.upcs.iter() {
            if let Some((dup, prod)) = upc_map.insert(upc.clone(), (Vec::new(), product.to_owned()))
            {
                let mut dup = dup;
                dup.push(product.to_owned());
                dup.push(prod.clone());
                upc_map.insert(upc.clone(), (dup, prod));
            }
        }
    }
    upc_map
}

#[derive(Debug)]
pub enum AbcParseError {
    CsvError(csv::Error),
    MissingField(String, usize),
    Custom(String),
}

impl std::fmt::Display for AbcParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(field, row) => {
                write!(f, "Missing field `{}` in row {}", field, row)
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

impl std::error::Error for AbcParseError {}

impl From<csv::Error> for AbcParseError {
    fn from(value: csv::Error) -> Self {
        Self::CsvError(value)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ExportedProduct {
    pub sku: String,
    pub upc: Ean13,
    pub desc: String,
    pub weight: Option<f64>,
    pub cost: Decimal,
    pub retail: Option<Decimal>,
}
