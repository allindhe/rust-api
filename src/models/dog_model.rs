use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Dog {
    pub _id: ObjectId,
    pub owner: ObjectId,
    pub name: Option<String>,
    pub age: Option<u8>,
    pub breed: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DogRequest {
    pub owner: String,
    pub name: Option<String>,
    pub age: Option<u8>,
    pub breed: Option<String>,
}

impl TryFrom<DogRequest> for Dog {
    type Error = Box<dyn std::error::Error>;

    fn try_from(item: DogRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: ObjectId::new(),
            owner: ObjectId::parse_str(&item.owner)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?,
            name: item.name,
            age: item.age,
            breed: item.breed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_json_to_dogrequest_and_convert_to_dog() {
        let json_data = json!({
            "owner": "507f1f77bcf86cd799439011",
            "name": "Max",
            "age": 5,
        });
        let dog_request: DogRequest = serde_json::from_value(json_data).unwrap();
        let dog = Dog::try_from(dog_request).unwrap();
        assert_eq!(dog.name, Some("Max".to_string()));
        assert_eq!(dog.age, Some(5));
        assert_eq!(dog.breed, None);
        assert_eq!(dog.owner.to_string(), "507f1f77bcf86cd799439011");
    }
}
