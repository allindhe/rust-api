use actix_web::{
    get, post,
    web::{self, Data, Json},
    HttpResponse,
};

use crate::{
    models::booking_model::{Booking, BookingRequest},
    services::db::Database,
};

#[post("/booking")]
pub async fn create_booking(db: Data<Database>, request: Json<BookingRequest>) -> HttpResponse {
    let booking_result = Booking::try_from(BookingRequest {
        owner: request.owner.clone(),
        start_time: request.start_time.clone(),
        duration_in_minutes: request.duration_in_minutes,
    });

    let booking = match booking_result {
        Ok(booking) => booking,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    match db.create_booking(booking).await {
        Ok(booking) => HttpResponse::Ok().json(booking),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/booking")]
pub async fn get_bookings(db: Data<Database>) -> HttpResponse {
    match db.get_bookings().await {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/booking/{id}")]
pub async fn get_booking(db: Data<Database>, path: web::Path<String>) -> HttpResponse {
    let booking_id = path.into_inner();

    match db.get_booking(&booking_id).await {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/booking/{id}/cancel")]
pub async fn cancel_booking(db: Data<Database>, path: web::Path<String>) -> HttpResponse {
    let booking_id = path.into_inner();

    match db.cancel_booking(&booking_id).await {
        Ok(bookings) => HttpResponse::Ok().json(bookings),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
