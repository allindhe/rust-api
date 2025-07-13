use std::time::SystemTime;

use chrono::Utc;
use futures_util::StreamExt;
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, DateTime},
    error::Error,
    results::{InsertOneResult, UpdateResult},
    Client, Collection,
};

use crate::models::{
    booking_model::{Booking, FullBooking},
    dog_model::Dog,
    owner_model::Owner,
};

pub struct Database {
    booking: Collection<Booking>,
    dog: Collection<Dog>,
    owner: Collection<Owner>,
}

impl Database {
    pub async fn init(db_name: &str, clean_db: bool) -> Self {
        let uri = match std::env::var("MONGO_URI") {
            Ok(v) => v.to_string(),
            Err(_) => panic!("No MONGO_URI set up."),
        };

        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(db_name);

        let booking = db.collection("booking");
        let dog = db.collection("dog");
        let owner = db.collection("owner");

        if clean_db {
            db.drop().await.unwrap();
        }

        Database {
            booking,
            dog,
            owner,
        }
    }

    pub async fn create_owner(&self, owner: Owner) -> Result<InsertOneResult, Error> {
        let result = self.owner.insert_one(owner).await?;
        Ok(result)
    }

    pub async fn create_dog(&self, dog: Dog) -> Result<InsertOneResult, Error> {
        let result = self.dog.insert_one(dog).await?;
        Ok(result)
    }

    pub async fn create_booking(&self, booking: Booking) -> Result<InsertOneResult, Error> {
        let result = self.booking.insert_one(booking).await?;
        Ok(result)
    }

    pub async fn cancel_booking(&self, booking_id: &str) -> Result<UpdateResult, Error> {
        let result = self
            .booking
            .update_one(
                doc! { "_id": ObjectId::parse_str(booking_id).unwrap()},
                doc! { "$set": { "cancelled": true } },
            )
            .await?;

        Ok(result)
    }

    pub async fn get_bookings(&self) -> Result<Vec<FullBooking>, Error> {
        let now: SystemTime = Utc::now().into();

        let mut results = self
            .booking
            .aggregate(vec![
                doc! {
                    "$match": doc! {
                        "cancelled": false,
                        "start_time": {
                            "$gte": DateTime::from_system_time(now)
                        }
                    }
                },
                doc! {
                    "$lookup": doc! {
                        "from": "owner",
                        "localField": "owner",
                        "foreignField": "_id",
                        "as": "owner"
                    }
                },
                doc! {
                    "$unwind": doc! {
                        "path": "$owner"
                    }
                },
                doc! {
                    "$lookup": doc! {
                        "from": "dog",
                        "localField": "owner._id",
                        "foreignField": "owner",
                        "as": "dogs"
                    }
                },
            ])
            .await?;

        let mut bookings: Vec<FullBooking> = Vec::new();

        while let Some(result) = results.next().await {
            match result {
                Ok(doc) => {
                    let booking: FullBooking = from_document(doc)?;
                    bookings.push(booking);
                }
                Err(_err) => {
                    println!("Error parsing booking database entry")
                }
            }
        }

        Ok(bookings)
    }

    pub async fn get_booking(&self, booking_id: &str) -> Result<FullBooking, Error> {
        let id = ObjectId::parse_str(booking_id).map_err(|_| {
            Error::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Invalid ID",
            ))
        })?;

        let mut results = self
            .booking
            .aggregate(vec![
                doc! {
                    "$match": doc! {
                        "_id": id,
                    }
                },
                doc! {
                    "$lookup": doc! {
                        "from": "owner",
                        "localField": "owner",
                        "foreignField": "_id",
                        "as": "owner"
                    }
                },
                doc! {
                    "$unwind": doc! {
                        "path": "$owner"
                    }
                },
                doc! {
                    "$lookup": doc! {
                        "from": "dog",
                        "localField": "owner._id",
                        "foreignField": "owner",
                        "as": "dogs"
                    }
                },
            ])
            .await?;

        let doc = results.next().await.ok_or_else(|| {
            Error::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Booking not found",
            ))
        })??;
        let booking: FullBooking = from_document(doc)?;

        Ok(booking)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::owner_model::OwnerRequest;

    use super::*;

    async fn setup_db() -> Database {
        Database::init("dog_walking_test", true).await
    }

    #[actix_web::test]
    async fn test_create_booking() {
        let db = setup_db().await;
        let owner = Owner::try_from(OwnerRequest {
            name: "Name".to_string(),
            email: "test@example.com".to_string(),
            phone: "0001112233".to_string(),
            address: "1337 Whoville, Texas".to_string(),
        })
        .unwrap();

        match db.create_owner(owner).await {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        };
    }
}
