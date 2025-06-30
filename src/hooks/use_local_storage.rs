use yew::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::Storage;

#[hook]
pub fn use_local_storage<T>(key: &str, default_value: T) -> (T, Callback<T>)
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + 'static,
{
    let key = key.to_string();
    let state = use_state(|| {
        // Try to load from local storage
        if let Some(storage) = get_local_storage() {
            if let Ok(Some(stored_value)) = storage.get_item(&key) {
                if let Ok(parsed_value) = serde_json::from_str::<T>(&stored_value) {
                    return parsed_value;
                }
            }
        }
        default_value
    });

    let setter = {
        let state = state.clone();
        let key = key.clone();
        Callback::from(move |new_value: T| {
            // Save to local storage
            if let Some(storage) = get_local_storage() {
                if let Ok(serialized) = serde_json::to_string(&new_value) {
                    let _ = storage.set_item(&key, &serialized);
                }
            }
            state.set(new_value);
        })
    };

    ((*state).clone(), setter)
}

fn get_local_storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok()?
}
