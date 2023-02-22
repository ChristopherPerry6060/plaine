use super::MonthlyStorageFees;
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
