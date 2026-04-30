use std::cell::RefCell;
use std::path::PathBuf;

use bb_core::data_handling::{
    appearance,
    article::Article,
    enums::{ArticleType, Location, SlotShape, UpgradeType},
    save::SaveData,
    upgrades::Upgrade,
};
use serde::Deserialize;
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

const WEAPONS_JSON: &str = include_str!("../../src-tauri/resources/weapons.json");
const ARMORS_JSON: &str = include_str!("../../src-tauri/resources/armors.json");
const ITEMS_JSON: &str = include_str!("../../src-tauri/resources/items.json");
const UPGRADES_JSON: &str = include_str!("../../src-tauri/resources/upgrades.json");

thread_local! {
    static SAVE: RefCell<Option<SaveData>> = const { RefCell::new(None) };
}

fn with_save<R>(f: impl FnOnce(&mut SaveData) -> R) -> Result<R, JsValue> {
    SAVE.with(|s| {
        let mut borrow = s.borrow_mut();
        match borrow.as_mut() {
            Some(save) => Ok(f(save)),
            None => Err(JsValue::from_str("No save loaded")),
        }
    })
}

fn to_js<T: serde::Serialize>(val: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(val).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn from_js<T: for<'de> Deserialize<'de>>(args: &JsValue, key: &str) -> Result<T, JsValue> {
    let v = js_sys::Reflect::get(args, &JsValue::from_str(key))
        .map_err(|_| JsValue::from_str(&format!("missing arg `{}`", key)))?;
    serde_wasm_bindgen::from_value(v)
        .map_err(|e| JsValue::from_str(&format!("arg `{}`: {}", key, e)))
}

fn from_js_value<T: for<'de> Deserialize<'de>>(val: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(val).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn parse_json(name: &str, raw: &str) -> Result<Value, JsValue> {
    serde_json::from_str(raw).map_err(|e| JsValue::from_str(&format!("{}: {}", name, e)))
}

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue> {
    match cmd {
        "make_save" => {
            let bytes: Vec<u8> = from_js(&args, "bytes")?;
            let save = SaveData::build_from_bytes(bytes, PathBuf::new())
                .map_err(|_| JsValue::from_str(
                    "Failed to load file, make sure its a decrypted character.",
                ))?;
            let val = to_js(&save)?;
            SAVE.with(|s| *s.borrow_mut() = Some(save));
            Ok(val)
        }
        "save" => {
            // Returns the bytes; JS handles download
            with_save(|save| save.file.bytes.clone())
                .and_then(|b| to_js(&b))
        }
        "return_weapons" => to_js(&parse_json("weapons", WEAPONS_JSON)?),
        "return_armors" => to_js(&parse_json("armors", ARMORS_JSON)?),
        "return_items" => to_js(&parse_json("items", ITEMS_JSON)?),
        "return_gem_effects" => {
            let v = parse_json("upgrades", UPGRADES_JSON)?;
            to_js(&v["gemEffects"])
        }
        "return_rune_effects" => {
            let v = parse_json("upgrades", UPGRADES_JSON)?;
            to_js(&v["runeEffects"])
        }
        "get_version" => Ok(JsValue::from_str(env!("CARGO_PKG_VERSION"))),
        "set_flag" => {
            let offset: usize = from_js(&args, "offset")?;
            let new_value: u8 = from_js(&args, "newValue")?;
            with_save(|s| s.file.set_flag(offset, new_value))?;
            Ok(JsValue::NULL)
        }
        "apply_mask" => {
            let offset: usize = from_js(&args, "offset")?;
            let mask: u8 = from_js(&args, "mask")?;
            with_save(|s| s.file.apply_mask(offset, mask))?;
            Ok(JsValue::NULL)
        }
        "get_isz" => with_save(|s| s.file.get_isz()).and_then(|v| to_js(&v)),
        "fix_isz" => with_save(|s| s.file.fix_isz()).map(|m| JsValue::from_str(&m)),
        "get_playtime" => with_save(|s| s.file.get_playtime()).map(JsValue::from),
        "set_playtime" => {
            let new_playtime: [u8; 4] = from_js(&args, "newPlaytime")?;
            with_save(|s| s.file.set_playtime(new_playtime))?;
            Ok(JsValue::NULL)
        }
        "edit_quantity" => {
            let number: u8 = from_js(&args, "number")?;
            let id: u32 = from_js(&args, "id")?;
            let value: u32 = from_js(&args, "value")?;
            let is_storage: bool = from_js(&args, "isStorage")?;
            with_save(|save| -> Result<JsValue, JsValue> {
                let res = if !is_storage {
                    save.inventory.edit_item(&mut save.file, number, id, value, is_storage)
                } else {
                    save.storage.edit_item(&mut save.file, number, id, value, is_storage)
                };
                res.map_err(|e| JsValue::from_str(&e.to_string()))?;
                to_js(save)
            })?
        }
        "transform_item" => {
            let index: usize = from_js(&args, "index")?;
            let id: u32 = from_js(&args, "id")?;
            let new_id: u32 = from_js(&args, "newId")?;
            let article_type: ArticleType = from_js(&args, "articleType")?;
            let is_storage: bool = from_js(&args, "isStorage")?;
            with_save(|save| -> Result<JsValue, JsValue> {
                let category = if !is_storage {
                    save.inventory.articles.get_mut(&article_type).unwrap()
                } else {
                    save.storage.articles.get_mut(&article_type).unwrap()
                };
                let item = category
                    .iter_mut()
                    .find(|x| x.id == id && x.index == index)
                    .ok_or_else(|| JsValue::from_str("item not found"))?;
                let old_type = item.article_type;
                item.transform(&mut save.file, new_id, is_storage)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                if item.article_type != old_type {
                    let moved = item.clone();
                    if let Some(old_cat) = save.inventory.articles.get_mut(&old_type) {
                        old_cat.retain(|x| x.index != index);
                    }
                    let new_cat = save
                        .inventory
                        .articles
                        .entry(moved.article_type)
                        .or_insert_with(Vec::new);
                    new_cat.push(moved);
                }
                to_js(save)
            })?
        }
        "edit_stat" => {
            let rel_offset: isize = from_js(&args, "relOffset")?;
            let length: usize = from_js(&args, "length")?;
            let times: usize = from_js(&args, "times")?;
            let value: u32 = from_js(&args, "value")?;
            with_save(|s| s.file.edit(rel_offset, length, times, value))?;
            Ok(JsValue::NULL)
        }
        "edit_effect" => {
            let new_effect_id: u32 = from_js(&args, "newEffectId")?;
            let index: usize = from_js(&args, "index")?;
            let info_js = js_sys::Reflect::get(&args, &JsValue::from_str("info"))
                .map_err(|_| JsValue::from_str("missing info"))?;
            let info: Value = from_js_value(info_js)?;
            edit_effect_or_shape(true, info, new_effect_id, index, String::new())
        }
        "edit_shape" => {
            let new_shape: String = from_js(&args, "newShape")?;
            let info_js = js_sys::Reflect::get(&args, &JsValue::from_str("info"))
                .map_err(|_| JsValue::from_str("missing info"))?;
            let info: Value = from_js_value(info_js)?;
            edit_effect_or_shape(false, info, 0, 0, new_shape)
        }
        "edit_slot" => {
            let is_storage: bool = from_js(&args, "isStorage")?;
            let article_type: ArticleType = from_js(&args, "articleType")?;
            let article_index: usize = from_js(&args, "articleIndex")?;
            let slot_index: usize = from_js(&args, "slotIndex")?;
            let new_shape: SlotShape = from_js(&args, "newShape")?;
            with_save(|save| -> Result<JsValue, JsValue> {
                let location = if is_storage { Location::Storage } else { Location::Inventory };
                let article: *mut Article = save
                    .get_article_mut(location, article_type, article_index)
                    .ok_or_else(|| JsValue::from_str("article not found"))?;
                let res = unsafe {
                    (*article).change_slot_shape(&mut save.file, slot_index, new_shape)
                };
                res.map_err(|e| JsValue::from_str(&e.to_string()))?;
                to_js(save)
            })?
        }
        "equip_gem" => {
            let upgrade_index: usize = from_js(&args, "upgradeIndex")?;
            let article_type: ArticleType = from_js(&args, "articleType")?;
            let article_index: usize = from_js(&args, "articleIndex")?;
            let slot_index: usize = from_js(&args, "slotIndex")?;
            let is_storage: bool = from_js(&args, "isStorage")?;
            with_save(|save| -> Result<JsValue, JsValue> {
                let res = if is_storage {
                    save.storage.equip_gem(&mut save.file, upgrade_index, article_type,
                        article_index, slot_index, is_storage)
                } else {
                    save.inventory.equip_gem(&mut save.file, upgrade_index, article_type,
                        article_index, slot_index, is_storage)
                };
                res.map_err(|e| JsValue::from_str(&e.to_string()))?;
                to_js(save)
            })?
        }
        "unequip_gem" => {
            let article_type: ArticleType = from_js(&args, "articleType")?;
            let article_index: usize = from_js(&args, "articleIndex")?;
            let slot_index: usize = from_js(&args, "slotIndex")?;
            let is_storage: bool = from_js(&args, "isStorage")?;
            with_save(|save| -> Result<JsValue, JsValue> {
                let res = if is_storage {
                    save.storage.unequip_gem(&mut save.file, article_type, article_index,
                        slot_index, is_storage)
                } else {
                    save.inventory.unequip_gem(&mut save.file, article_type, article_index,
                        slot_index, is_storage)
                };
                res.map_err(|e| JsValue::from_str(&e.to_string()))?;
                to_js(save)
            })?
        }
        "export_appearance" => {
            // Returns bytes; JS triggers download
            with_save(|s| appearance::export_bytes(&s.file)).and_then(|b| to_js(&b))
        }
        "import_appearance" => {
            let bytes: Vec<u8> = from_js(&args, "bytes")?;
            with_save(|s| appearance::import_bytes(&mut s.file, &bytes))?
                .map_err(|_| JsValue::from_str("The imported file is not a face"))?;
            Ok(JsValue::from_str("Successfully imported"))
        }
        "set_username" => {
            let new_username: String = from_js(&args, "newUsername")?;
            with_save(|s| s.username.set(&mut s.file, new_username))?
                .map_err(|_| JsValue::from_str("Failed to change name"))?;
            Ok(JsValue::from_str("Successfully changed name"))
        }
        "add_item" => {
            let id: u32 = from_js(&args, "id")?;
            let quantity: u32 = from_js(&args, "quantity")?;
            let is_storage: bool = from_js(&args, "isStorage")?;
            with_save(|save| -> Result<JsValue, JsValue> {
                let res = if !is_storage {
                    save.inventory.add_item(&mut save.file, id, quantity, is_storage)
                } else {
                    save.storage.add_item(&mut save.file, id, quantity, is_storage)
                };
                res.map_err(|_| JsValue::from_str("Failed to add the item"))?;
                to_js(save)
            })?
        }
        "edit_coordinates" => {
            let x: f32 = from_js(&args, "x")?;
            let y: f32 = from_js(&args, "y")?;
            let z: f32 = from_js(&args, "z")?;
            with_save(|s| s.position.coordinates.edit(&mut s.file, x, y, z))?;
            Ok(JsValue::NULL)
        }
        "teleport" => {
            let x: f32 = from_js(&args, "x")?;
            let y: f32 = from_js(&args, "y")?;
            let z: f32 = from_js(&args, "z")?;
            let map_id: Vec<u8> = from_js(&args, "mapId")?;
            with_save(|save| {
                let le_map = [0, 0, map_id[1], map_id[0]];
                for (i, j) in (0x04..0x08).enumerate() {
                    save.file.bytes[j] = le_map[i];
                }
                save.position.coordinates.edit(&mut save.file, x, y, z);
            })?;
            Ok(JsValue::NULL)
        }
        "change_weapon_level" => {
            let article_type: ArticleType = from_js(&args, "articleType")?;
            let article_index: usize = from_js(&args, "articleIndex")?;
            let slot_index: usize = from_js(&args, "slotIndex")?;
            let is_storage: bool = from_js(&args, "isStorage")?;
            let level: u8 = from_js(&args, "level")?;
            with_save(|save| -> Result<JsValue, JsValue> {
                let res = if is_storage {
                    save.storage.change_weapon_level(&mut save.file, article_type,
                        article_index, slot_index, is_storage, level)
                } else {
                    save.inventory.change_weapon_level(&mut save.file, article_type,
                        article_index, slot_index, is_storage, level)
                };
                let weapon = res.map_err(|e| JsValue::from_str(&e.to_string()))?;
                to_js(&json!({
                    "save": serde_json::to_value(&*save).map_err(|x| x.to_string())?,
                    "weapon": weapon
                }))
            })?
        }
        other => Err(JsValue::from_str(&format!("Unknown command `{}`", other))),
    }
}

fn edit_effect_or_shape(
    is_effect: bool,
    info: Value,
    new_effect_id: u32,
    index: usize,
    new_shape: String,
) -> Result<JsValue, JsValue> {
    with_save(|save| -> Result<JsValue, JsValue> {
        let location: Location = {
            let is_storage: bool = serde_json::from_value(info["isStorage"].clone())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            if is_storage { Location::Storage } else { Location::Inventory }
        };

        let upgrade: *mut Upgrade = if let Some(equipped) = info.get("equipped") {
            let article_type: ArticleType = serde_json::from_value(equipped["articleType"].clone())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            let article_index: usize = serde_json::from_value(equipped["articleIndex"].clone())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            let slot_index: usize = serde_json::from_value(equipped["slotIndex"].clone())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            save.get_equipped_upgrade_mut(location, article_type, article_index, slot_index)
                .map(|u| u as *mut _)
                .ok_or_else(|| JsValue::from_str("equipped upgrade not found"))?
        } else {
            let upgrade_type: UpgradeType = serde_json::from_value(info["upgradeType"].clone())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            let upgrade_index: usize = serde_json::from_value(info["upgradeIndex"].clone())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            save.get_upgrade_mut(location, upgrade_type, upgrade_index)
                .map(|u| u as *mut _)
                .ok_or_else(|| JsValue::from_str("upgrade not found"))?
        };

        unsafe {
            let res = if is_effect {
                (*upgrade).change_effect(&mut save.file, new_effect_id, index)
                    .map_err(|_| JsValue::from_str("Failed to edit the upgrade's effect"))
            } else {
                (*upgrade).change_shape(&mut save.file, new_shape)
                    .map_err(|_| JsValue::from_str("Failed to edit the upgrade's shape"))
            };
            res?;
        }
        to_js(save)
    })?
}
