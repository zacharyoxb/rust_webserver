use crate::html_getters::web_content::WebContent;
use crate::method_handlers::handler_utils;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use std::convert::Infallible;

pub(crate) async fn generate_response(
    req: &Request<hyper::body::Incoming>,
    web_content: WebContent,
) -> Result<Response<Full<Bytes>>, Infallible> {
    // Check if the content is a 404 Not Found
    if web_content.is_not_found() {
        return handler_utils::packet_templates::send_not_found_packet(
            web_content.get_data().clone(),
        );
    }

    // If it's not a 404, proceed with the regular content
    let mut valid_is_match = false;
    let mut valid_if_none_match = false;

    // Handle If-Match when header present
    if let Some(header) = req.headers().get("If-Match") {
        match handler_utils::header_evals::if_match(header, web_content.get_etag().unwrap()) {
            Ok(true) => valid_is_match = true,
            Ok(false) => return handler_utils::packet_templates::send_precondition_failed_packet(),
            Err(_) => {}
        }
    }

    // Handle If-Unmodified-Since when header present and valid If-Match header is not present
    if let Some(header) = req.headers().get("If-Unmodified-Since") {
        if !valid_is_match {
            if let Ok(false) = handler_utils::header_evals::if_unmodified_since(
                header,
                web_content.get_last_modified().unwrap(),
            ) {
                return handler_utils::packet_templates::send_precondition_failed_packet();
            }
        }
    }

    // Handle If-None-Match when header present
    if let Some(header) = req.headers().get("If-None-Match") {
        match handler_utils::header_evals::if_none_match(header, web_content.get_etag().unwrap()) {
            Ok(true) => valid_if_none_match = true,
            Ok(false) => return handler_utils::packet_templates::send_not_modified_packet(),
            Err(_) => {}
        }
    }

    // Handle If-Modified-Since when header present and valid If-None-Match is not present
    if let Some(header) = req.headers().get("If-Modified-Since") {
        if !valid_if_none_match {
            if let Ok(false) = handler_utils::header_evals::if_modified_since(
                header,
                web_content.get_last_modified().unwrap(),
            ) {
                return handler_utils::packet_templates::send_not_modified_packet();
            }
        }
    }

    // Handle If-Range when header present and Bytes not empty (i.e. not a HEAD request)
    if let (Some(range_header), if_range_header, date_header) = (
        req.headers().get("Range"),
        req.headers().get("If-Range"),
        req.headers().get("Date"),
    ) {
        if let Ok(true) = handler_utils::header_evals::if_range(
            if_range_header,
            web_content.get_last_modified().unwrap(),
            web_content.get_etag().unwrap(),
            date_header,
        ) {
            if let Ok(sliced_content) =
                handler_utils::header_evals::range(web_content.get_data(), range_header)
            {
                return if sliced_content.len() == 1 {
                    let (data, start, end) = &sliced_content[0];
                    handler_utils::packet_templates::send_partial_content_packet(
                        data.clone(),
                        start,
                        end,
                        &web_content.get_data().len(),
                        web_content.get_last_modified().unwrap(),
                        web_content.get_etag().unwrap(),
                    )
                } else {
                    handler_utils::packet_templates::send_multipart_packet(
                        sliced_content,
                        &web_content.get_data().len(),
                        web_content.get_last_modified().unwrap(),
                        web_content.get_etag().unwrap(),
                    )
                };
            }
        }
    }

    // If no If-Range header/is a HEAD request, send ok response
    handler_utils::packet_templates::send_default_ok_packet(
        web_content.get_data().clone(),
        web_content.get_last_modified().unwrap().clone(),
        web_content.get_etag().unwrap(),
    )
}
