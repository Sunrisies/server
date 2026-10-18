use crate::handlers::site_features::{analytics_handler, feedback_handler, subscribe_handler};
use actix_web::web;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/v1/feedback", web::post().to(feedback_handler))
        .route("/v1/subscriptions", web::post().to(subscribe_handler))
        .route("/v1/analytics/dashboard", web::get().to(analytics_handler));
}
