use crate::app::Collection;
use crate::request::{Request, Response};

pub struct StoreKey {
    collection_id: String,
    request_id: String,
}

impl StoreKey {
    fn new(collection_id: &str, request_id: &str) -> Self {
        Self {
            collection_id: collection_id.to_string(),
            request_id: request_id.to_string(),
        }
    }

    pub fn from(collection: &Collection, request: &Request) -> Self {
        Self {
            collection_id: collection.id.clone(),
            request_id: request.id.clone(),
        }
    }

    fn is_invalid(&self) -> bool {
        self.is_empty() || self.collection_id.contains('/') || self.request_id.contains('/')
    }

    fn is_empty(&self) -> bool {
        self.collection_id.is_empty() || self.request_id.is_empty()
    }

    fn to_response_path(&self) -> std::path::PathBuf {
        let mut path = std::path::PathBuf::new();
        path.push(&self.collection_id);
        path.push(&self.request_id);
        path.push("responses.json");
        path
    }

    fn to_request_path(&self) -> std::path::PathBuf {
        let mut path = std::path::PathBuf::new();
        path.push(&self.collection_id);
        path.push(&self.request_id);
        path
    }

    fn to_path(&self) -> std::path::PathBuf {
        let mut path = std::path::PathBuf::new();
        path.push(&self.collection_id);
        path.push(&self.request_id);
        path.push("request.json");
        path
    }

    fn collection_data_path(&self) -> std::path::PathBuf {
        let mut path = std::path::PathBuf::new();
        path.push(&self.collection_id);
        path.push("collection.json");
        path
    }
}

#[derive(Clone)]
pub struct LocalStore {
    root_path: std::path::PathBuf,
    data_path: std::path::PathBuf,
}

impl LocalStore {
    pub fn new() -> Self {
        let root_path = std::env::current_dir().unwrap();

        Self {
            root_path: root_path.clone(),
            data_path: root_path.join("data"),
        }
    }

    pub fn update_responses(&self, key: StoreKey, response: &Response) -> anyhow::Result<()> {
        if let Err(e) = self.validate_response_key(&key) {
            return Err(e);
        }

        if let Err(e) = self.validate_response_file(&key) {
            return Err(e);
        }

        let file_path = self.data_path.join(key.to_response_path());
        // append to array then write back to file
        let mut responses: Vec<Response> = if file_path.exists() {
            let file = std::fs::File::open(&file_path).unwrap();
            let reader = std::io::BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_else(|_| vec![])
        } else {
            vec![]
        };
        responses.push(response.clone());
        let json_data = serde_json::to_string(&responses).unwrap();
        std::fs::write(file_path, json_data).unwrap();
        anyhow::Ok(())
    }

    pub fn save_all(&self, collections: Vec<Collection>) -> Result<(), String> {
        if collections.is_empty() {
            return Err("No collections to save".to_string());
        }

        for collection in collections {
            let key = StoreKey::new(&collection.id, "");
            if !self.validate_key(&key, true) {
                return Err(format!("Invalid key for collection {}", collection.id));
            }

            let collection_data_file = self.data_path.join(key.collection_data_path());
            let json_data = serde_json::to_string(&collection).map_err(|e| e.to_string())?;
            std::fs::write(collection_data_file, json_data).map_err(|e| e.to_string())?;

            let mut tmp = collection.requests.iter();
            while let Some(request) = tmp.next() {
                let key = StoreKey::from(&collection, &request);
                if !self.validate_key(&key, false) {
                    return Err(format!(
                        "Invalid key for collection {} and request {}",
                        collection.id, request.id
                    ));
                }

                let file_path = self.data_path.join(key.to_path());
                let json_data = serde_json::to_string(&request).map_err(|e| e.to_string())?;
                std::fs::write(file_path, json_data).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }


    pub fn load_all(&self) -> Result<Vec<Collection>, String> {
        let mut collections = Vec::new();
        if !self.data_path.exists() {
            return Ok(collections);
        }

        for entry in std::fs::read_dir(&self.data_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                let collection_data_file = path.join("collection.json");
                if collection_data_file.exists() {
                    let file =
                        std::fs::File::open(&collection_data_file).map_err(|e| e.to_string())?;
                    let reader = std::io::BufReader::new(file);
                    let collection: Collection =
                        serde_json::from_reader(reader).map_err(|e| e.to_string())?;
                    collections.push(collection);
                }
            }
        }

        Ok(collections)
    }

    fn validate_response_file(&self, key: &StoreKey) -> anyhow::Result<()> {
        let response_file_path = self.data_path.join(key.to_response_path());

        if !response_file_path.exists() {
            std::fs::File::create(&response_file_path)?;
        }

        let reader = std::io::BufReader::new(std::fs::File::open(&response_file_path)?);
        if serde_json::from_reader::<_, serde_json::Value>(reader).is_err() {
            std::fs::write(&response_file_path, "[]")?;
        }

        let response_json = serde_json::from_reader::<_, serde_json::Value>(
            std::io::BufReader::new(std::fs::File::open(&response_file_path)?),
        )?;

        if !response_json.is_array() {
            std::fs::write(&response_file_path, "[]")?;
        }

        Ok(())
    }

    fn validate_response_key(&self, key: &StoreKey) -> anyhow::Result<()> {
        if !self.data_path.exists() {
            std::fs::create_dir_all(&self.data_path).unwrap();
        }

        let collection_path = self.data_path.join(&key.collection_id);
        if !collection_path.exists() {
            if let Err(e) = std::fs::create_dir_all(&collection_path) {
                return Err(anyhow::anyhow!(e));
            }
        }

        let file_path = self.data_path.join(key.to_request_path());
        if !file_path.exists() {
            if let Err(e) = std::fs::create_dir_all(&file_path) {
                return Err(anyhow::anyhow!(e));
            }
        }

        let response_file_path = self.data_path.join("responses.json");
        if !response_file_path.exists() {
            if let Err(e) = std::fs::File::create(&response_file_path) {
                return Err(anyhow::anyhow!(e));
            }
        }

        Ok(())
    }

    fn validate_key(&self, key: &StoreKey, skip_request: bool) -> bool {
        if !self.data_path.exists() {
            std::fs::create_dir_all(&self.data_path).unwrap();
        }

        let collection_path = self.data_path.join(&key.collection_id);
        if !collection_path.exists() {
            std::fs::create_dir_all(collection_path).unwrap_or_else(|e| {
                panic!(
                    "Failed to create directory for collection {}: {}",
                    key.collection_id, e
                )
            });
        }

        let collection_data_file = self
            .data_path
            .join(&key.collection_id)
            .join("collection.json");
        if !collection_data_file.exists() {
            std::fs::File::create(collection_data_file).unwrap_or_else(|e| {
                panic!(
                    "Failed to create collection data file for collection {}: {}",
                    key.collection_id, e
                )
            });
        }

        if skip_request {
            return true;
        }

        if key.is_invalid() {
            return false;
        }

        let request_path = self.data_path.join(key.to_request_path());
        if !request_path.exists() {
            std::fs::create_dir_all(&request_path).unwrap_or_else(|e| {
                panic!(
                    "Failed to create directory for collection {} and request {}: {}",
                    key.collection_id, key.request_id, e
                )
            });
        }

        let file_path = self.data_path.join(key.to_path());
        if !file_path.exists() {
            std::fs::File::create(file_path).unwrap_or_else(|e| {
                panic!(
                    "Failed to create file for collection {} and request {}: {}",
                    key.collection_id, key.request_id, e
                )
            });
        }

        true
    }
}
