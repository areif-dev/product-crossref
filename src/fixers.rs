use std::fs;

use abc_product::AbcProduct;
use abc_uiautomation::{
    inventory::{clear_upc, set_upc},
    read_text_box_value, set_text_box_value, UIElement,
};

use crate::product::{DuplicateProducts, ExportedProduct};

/// Controls ABC Client4 window to reorder the UPCs of an inventory item so that the primary UPC
/// from a vendor exported list is the primary UPC in ABC. IE the last UPC in ABC will match the
/// UPC from the export
///
/// # Arguments
/// * `inventory_window` - The [`UIElement`] representing the Inventory screen of Client4
/// * `abc_prod` - Represents the ABC item as it exists before manipulation
/// * `ex_prod` - The product listing that was exported from a vendor
///
/// # Errors
/// Forwards any [`abc_uiautomation::Error`]s resulting from
/// * Failing to clear exising UPCs
/// * Failing to set any new UPCs
pub fn fix_upc(
    inventory_window: &UIElement,
    abc_prod: &AbcProduct,
    ex_prod: &ExportedProduct,
) -> Result<(), abc_uiautomation::Error> {
    clear_upc(inventory_window, true)?;
    for upc in abc_prod.upcs() {
        if upc != ex_prod.upc {
            set_upc(inventory_window, upc)?;
        }
    }
    set_upc(inventory_window, ex_prod.upc)?;
    Ok(())
}

/// Controls ABC Client4 window to add or fix the weight value in an ABC inventory listing
///
/// # Arguments
/// * `inventory_window` - The [`UIElement`] representing the Inventory screen of Client4
/// * `abc_prod` - Represents the ABC item as it exists before manipulation
/// * `ex_prod` - The product listing that was exported from a vendor
///
/// # Errors
/// Forwards any [`abc_uiautomation::Error`]s resulting from failing to set the weight value in ABC
pub fn fix_weight(
    inventory_window: &UIElement,
    _abc_prod: &AbcProduct,
    ex_prod: &ExportedProduct,
) -> Result<(), abc_uiautomation::Error> {
    if let Some(weight) = ex_prod.weight {
        set_text_box_value(inventory_window, 15, weight.to_string())?;
    }
    Ok(())
}

/// Controls ABC Client4 window to add or fix the cost value in an ABC inventory listing
///
/// # Arguments
/// * `inventory_window` - The [`UIElement`] representing the Inventory screen of Client4
/// * `abc_prod` - Represents the ABC item as it exists before manipulation
/// * `ex_prod` - The product listing that was exported from a vendor
///
/// # Errors
/// Forwards any [`abc_uiautomation::Error`]s resulting from failing to set the cost value in ABC
pub fn fix_cost(
    inventory_window: &UIElement,
    _abc_prod: &AbcProduct,
    ex_prod: &ExportedProduct,
) -> Result<(), abc_uiautomation::Error> {
    set_text_box_value(inventory_window, 26, ex_prod.cost.to_string())?;
    Ok(())
}

/// Controls ABC Client4 window to add or fix the list/retail value in an ABC inventory listing
///
/// # Arguments
/// * `inventory_window` - The [`UIElement`] representing the Inventory screen of Client4
/// * `abc_prod` - Represents the ABC item as it exists before manipulation
/// * `ex_prod` - The product listing that was exported from a vendor
///
/// # Errors
/// Forwards any [`abc_uiautomation::Error`]s resulting from failing to set the retail value in ABC
pub fn fix_retail(
    inventory_window: &UIElement,
    _abc_prod: &AbcProduct,
    ex_prod: &ExportedProduct,
) -> Result<(), abc_uiautomation::Error> {
    if let Some(retail) = ex_prod.retail {
        set_text_box_value(inventory_window, 25, retail.to_string())?;
    }
    Ok(())
}

/// Controls ABC Client4 window to add or fix the group value in an ABC inventory listing
///
/// # Arguments
/// * `inventory_window` - The [`UIElement`] representing the Inventory screen of Client4
/// * `abc_prod` - Represents the ABC item as it exists before manipulation
/// * `ex_prod` - The product listing that was exported from a vendor
///
/// # Errors
/// Forwards any [`abc_uiautomation::Error`]s resulting from failing to set the group value in ABC
pub fn fix_group(
    inventory_window: &UIElement,
    _abc_prod: &AbcProduct,
    _ex_prod: &ExportedProduct,
) -> Result<(), abc_uiautomation::Error> {
    set_text_box_value(inventory_window, 39, "Z")?;
    Ok(())
}

/// Add the vendor sku to an ABC item listing by adding it to the list of alternative skus
///
/// # Arguments
/// * `inventory_window` - The [`UIElement`] representing the Inventory screen of Client4
/// * `abc_prod` - Represents the ABC item as it exists before manipulation
/// * `ex_prod` - The product listing that was exported from a vendor
///
/// # Errors
/// Forwards any [`abc_uiautomation::Error`]s resulting from failing to set the group value in ABC
pub fn fix_alt_sku(
    inventory_window: &UIElement,
    _abc_prod: &AbcProduct,
    ex_prod: &ExportedProduct,
) -> Result<(), abc_uiautomation::Error> {
    for i in 35..38 {
        let spot = read_text_box_value(inventory_window, i)?;
        if spot.is_empty() {
            set_text_box_value(inventory_window, i, &ex_prod.sku)?;
            break;
        }
    }
    Ok(())
}

/// Write log files to enumerate all products that failed to be cross referenced due to one of the
/// following:
///
/// * There are multiple ABC listings that share a UPC (duplicate_products.txt)
/// * There are no ABC products with a UPC that matches the UPC from the vendor export
/// (new_products.txt)
/// * There is a matching ABC listing, but either the list price or the cost is vastly different,
/// so it is worth having a human double check it (double_check.txt)
///
/// Also writes a list of products that were successfully matched in ABC (matched_products.txt)
///
/// # Arguments
/// * `dups` - The list of [`AbcProduct`]s that share a UPC
/// * `new` - The list of [`ExportedProduct`]s that do not already exist in ABC
/// * `check` - The list of [`ExportedProduct`]s that have a UPC match but seem to be vastly
/// different from the matching ABC listing
/// * `matches` - Lit of [`ExportedProduct`]s that have a good UPC match in ABC and were able to be adjusted in ABC
///
/// # Errors
/// Forwards any [`std::io::Error`]s resulting from trying to write any of the log files
pub fn write_logs(
    dups: Vec<&DuplicateProducts>,
    new: Vec<ExportedProduct>,
    check: Vec<ExportedProduct>,
    matches: Vec<ExportedProduct>,
) -> std::io::Result<()> {
    fs::write(
        "./duplicate_products.txt",
        format!(
            "The following products all share the same UPC. You may want to fix that.\n\n{:#?}",
            dups
        ),
    )?;
    fs::write(
        "./new_products.txt",
        format!(
            "The following products are new to ABC. Please enter them manually.\n\n{:#?}",
            new
        ),
    )?;
    fs::write("./double_check.txt", format!("The following products seem to have changed wildly. Please double check that their listings are correct.\n\n{:#?}", check))?;
    fs::write(
        "./matched_products.txt",
        format!(
            "The following products were successfully cross referenced.\n\n{:#?}",
            matches
        ),
    )?;
    Ok(())
}
