use crate::read::all_listings_report::Condition;

use super::{all_listings_report::AllListingsReport, MonthlyStorageFees};
use anyhow::anyhow;
use csv::StringRecord;

#[test]
fn monthly_storage_fees() -> anyhow::Result<()> {
    let cols = vec![
        "asin",
        "fnsku",
        "product_name",
        "fulfillment_center",
        "country_code",
        "longest_side",
        "median_side",
        "shortest_side",
        "measurement_units",
        "weight",
        "weight_units",
        "item_volume",
        "volume_units",
        "product_size_tier",
        "average_quantity_on_hand",
        "average_quantity_pending_removal",
        "estimated_total_item_volume",
        "month_of_charge",
        "storage_rate",
        "currency",
        "estimated_monthly_storage_fee",
        "dangerous_goods_storage_type",
        "eligible_for_inventory_discount",
        "qualifies_for_inventory_discount",
        "total_incentive_fee_amount",
        "breakdown_incentive_fee_amount",
        "average_quantity_customer_orders",
    ];

    let row = vec![
        "B0B9CCP98J",
        "X003C6LE0L",
        "Sony XXXXXXX (Black)",
        "ABE2",
        "US",
        "10.28",
        "10.2",
        "4.69",
        "inches",
        "2.94",
        "pounds",
        "0.2841",
        "cubic feet",
        "Standard-Size",
        "0.57",
        "0.0",
        "0.161",
        "2022-11",
        "2.4",
        "USD",
        "0.39",
        "--",
        "N",
        "N",
        "0.0",
        "--",
        "0.0",
    ];
    let hdr = StringRecord::from(cols);
    let row1 = StringRecord::from(row);
    let de: MonthlyStorageFees = row1.deserialize(Some(&hdr))?;
    let pst = String::from("Standard-Size");
    let pn = String::from("Sony XXXXXXX (Black)");
    let asin = String::from("B0B9CCP98J");

    assert_eq!(de.product_size_tier, Some(pst));
    assert_eq!(de.product_name, Some(pn));
    assert_eq!(de.asin, Some(asin));
    Ok(())
}

#[test]
fn all_listings_report() -> anyhow::Result<()> {
    let hdr = StringRecord::from(vec![
        "seller-sku",
        "asin1",
        "item-name",
        "product-id-type",
        "item-condition",
        "product-id",
    ]);

    let row1 = StringRecord::from(vec![
        "mon0000000003_udf",
        "B00000NYIC",
        "BDI Bink 1025 Mobile Media Table, Salt",
        "1",
        "11",
        "B00BLQNYIC",
    ]);

    let rd: AllListingsReport = row1.deserialize(Some(&hdr))?;
    let sku = rd.seller_sku.ok_or_else(|| anyhow!("deserde failed"))?;
    let asin = rd.asin.ok_or_else(|| anyhow!("deserde failed"))?;
    let condition = rd.item_condition;

    assert_eq!(sku, "mon0000000003_udf");
    assert_eq!(asin, "B00000NYIC");
    assert_eq!(condition, Condition::New);
    Ok(())
}
