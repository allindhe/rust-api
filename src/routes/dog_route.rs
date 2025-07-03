use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};

use crate::{
    models::dog_model::{Dog, DogRequest},
    services::db::Database,
};

#[post("/dog")]
pub async fn create_dog(db: Data<Database>, request: Json<DogRequest>) -> HttpResponse {
    let dog_result = Dog::try_from(DogRequest {
        owner: request.owner.clone(),
        name: request.name.clone(),
        age: request.age,
        breed: request.breed.clone(),
    });

    let dog = match dog_result {
        Ok(dog) => dog,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    match db.create_dog(dog).await {
        Ok(dog) => HttpResponse::Ok().json(dog),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
