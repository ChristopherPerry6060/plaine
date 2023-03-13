use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all(serialize = "PascalCase"))]
#[serde(rename_all(deserialize = "snake_case"))]
pub struct MonsoonItem {
    #[serde(alias = "_id")]
    #[serde(rename(serialize = "_id"))]
    id: Id,
    #[serde(alias = "SKU")]
    #[serde(rename(serialize = "SKU"))]
    sku: Option<String>,
    #[serde(alias = "Title")]
    title: Option<String>,
    #[serde(alias = "UPC")]
    upc: Option<String>,
    #[serde(alias = "ManufacturerPartNum")]
    manufacturer_part_num: Option<String>,
    #[serde(alias = "ASIN")]
    asin: Option<String>,
    #[serde(alias = "LocatorCode")]
    locator_code: Option<String>,
    #[serde(alias = "Quantity")]
    quantity: Option<String>,
    #[serde(alias = "Condition")]
    condition: Option<String>,
    #[serde(alias = "Price")]
    price: Option<String>,
    #[serde(alias = "FNSKU")]
    fnsku: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct Id {
    #[serde(alias = "$oid")]
    oid: Option<String>,
}

#[cfg(test)]
mod tests {
    use mongodb::bson::{bson, from_bson};

    use super::*;
    #[test]
    fn monsoon_item() {
        let doc = bson! ({
            "_id": {
                "$oid": "640a90ce29ee73b43340aa76"
            },
            "SKU": "mo40600000123",
            "Title": "AudioQue (5.0 meters)",
            "UPC": "092592061965",
            "ManufacturerPartNum": "GOLDG05R",
            "ASIN": "B005TI1PJ8",
            "LocatorCode": "01-01B",
            "Quantity": "0",
            "Condition": "New",
            "Price": "199.95",
            "FNSKU": "X062KI3RCB"
        });

        let monsoon_item_struct: MonsoonItem = from_bson(doc).unwrap();
        assert_eq!(monsoon_item_struct.condition, Some("New".to_string()));
    }
}
