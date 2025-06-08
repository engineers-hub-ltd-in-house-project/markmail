use actix_web::test;
use serde_json::json;
use uuid::Uuid;

use crate::models::form::{CreateFormRequest, Form};
use crate::tests::helpers::{create_test_user, get_auth_header, init_test_app};

#[actix_web::test]
async fn test_create_form() {
    let app = init_test_app().await;
    let user = create_test_user(&app.pool).await;
    let auth_header = get_auth_header(&user.id, &user.email);

    let form_data = CreateFormRequest {
        name: "Contact Form".to_string(),
        description: Some("A simple contact form".to_string()),
        slug: Some("contact".to_string()),
        markdown_content: "# Contact Us\n\nPlease fill out this form.".to_string(),
        form_fields: Some(json!([
            {
                "field_type": "text",
                "name": "name",
                "label": "Your Name",
                "required": true
            },
            {
                "field_type": "email",
                "name": "email",
                "label": "Email Address",
                "required": true
            }
        ])),
        settings: Some(json!({
            "submit_button_text": "Send Message",
            "success_message": "Thank you for contacting us!"
        })),
    };

    let req = test::TestRequest::post()
        .uri("/api/forms")
        .append_header(auth_header)
        .set_json(&form_data)
        .to_request();

    let resp = test::call_service(&app.app, req).await;
    assert!(resp.status().is_success());

    let form: Form = test::read_body_json(resp).await;
    assert_eq!(form.name, "Contact Form");
    assert_eq!(form.slug, "contact");
    assert_eq!(form.user_id, user.id);
}

#[actix_web::test]
async fn test_get_user_forms() {
    let app = init_test_app().await;
    let user = create_test_user(&app.pool).await;
    let auth_header = get_auth_header(&user.id, &user.email);

    // Create multiple forms
    for i in 0..3 {
        let form_data = CreateFormRequest {
            name: format!("Form {}", i),
            description: Some(format!("Description {}", i)),
            slug: Some(format!("form-{}", i)),
            markdown_content: format!("# Form {}", i),
            form_fields: Some(json!([])),
            settings: Some(json!({})),
        };

        let req = test::TestRequest::post()
            .uri("/api/forms")
            .append_header(auth_header.clone())
            .set_json(&form_data)
            .to_request();

        let resp = test::call_service(&app.app, req).await;
        assert!(resp.status().is_success());
    }

    // Get all forms
    let req = test::TestRequest::get()
        .uri("/api/forms")
        .append_header(auth_header)
        .to_request();

    let resp = test::call_service(&app.app, req).await;
    assert!(resp.status().is_success());

    let forms: Vec<Form> = test::read_body_json(resp).await;
    assert_eq!(forms.len(), 3);
}

#[actix_web::test]
async fn test_get_form_by_id() {
    let app = init_test_app().await;
    let user = create_test_user(&app.pool).await;
    let auth_header = get_auth_header(&user.id, &user.email);

    // Create a form
    let form_data = CreateFormRequest {
        name: "Test Form".to_string(),
        description: Some("Test description".to_string()),
        slug: Some("test-form".to_string()),
        markdown_content: "# Test Form".to_string(),
        form_fields: Some(json!([])),
        settings: Some(json!({})),
    };

    let req = test::TestRequest::post()
        .uri("/api/forms")
        .append_header(auth_header.clone())
        .set_json(&form_data)
        .to_request();

    let resp = test::call_service(&app.app, req).await;
    let form: Form = test::read_body_json(resp).await;

    // Get form by ID
    let req = test::TestRequest::get()
        .uri(&format!("/api/forms/{}", form.id))
        .append_header(auth_header)
        .to_request();

    let resp = test::call_service(&app.app, req).await;
    assert!(resp.status().is_success());

    let retrieved_form: Form = test::read_body_json(resp).await;
    assert_eq!(retrieved_form.id, form.id);
    assert_eq!(retrieved_form.name, form.name);
}

#[actix_web::test]
async fn test_update_form() {
    let app = init_test_app().await;
    let user = create_test_user(&app.pool).await;
    let auth_header = get_auth_header(&user.id, &user.email);

    // Create a form
    let form_data = CreateFormRequest {
        name: "Original Form".to_string(),
        description: Some("Original description".to_string()),
        slug: Some("original-form".to_string()),
        markdown_content: "# Original Form".to_string(),
        form_fields: Some(json!([])),
        settings: Some(json!({})),
    };

    let req = test::TestRequest::post()
        .uri("/api/forms")
        .append_header(auth_header.clone())
        .set_json(&form_data)
        .to_request();

    let resp = test::call_service(&app.app, req).await;
    let form: Form = test::read_body_json(resp).await;

    // Update the form
    let update_data = json!({
        "name": "Updated Form",
        "description": "Updated description",
        "status": "published"
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/forms/{}", form.id))
        .append_header(auth_header.clone())
        .set_json(&update_data)
        .to_request();

    let resp = test::call_service(&app.app, req).await;
    assert!(resp.status().is_success());

    let updated_form: Form = test::read_body_json(resp).await;
    assert_eq!(updated_form.name, "Updated Form");
    assert_eq!(
        updated_form.description,
        Some("Updated description".to_string())
    );
    assert_eq!(updated_form.status, "published");
}

#[actix_web::test]
async fn test_delete_form() {
    let app = init_test_app().await;
    let user = create_test_user(&app.pool).await;
    let auth_header = get_auth_header(&user.id, &user.email);

    // Create a form
    let form_data = CreateFormRequest {
        name: "To Delete".to_string(),
        description: None,
        slug: Some("to-delete".to_string()),
        markdown_content: "# To Delete".to_string(),
        form_fields: Some(json!([])),
        settings: Some(json!({})),
    };

    let req = test::TestRequest::post()
        .uri("/api/forms")
        .append_header(auth_header.clone())
        .set_json(&form_data)
        .to_request();

    let resp = test::call_service(&app.app, req).await;
    let form: Form = test::read_body_json(resp).await;

    // Delete the form
    let req = test::TestRequest::delete()
        .uri(&format!("/api/forms/{}", form.id))
        .append_header(auth_header.clone())
        .to_request();

    let resp = test::call_service(&app.app, req).await;
    assert!(resp.status().is_success());

    // Verify it's deleted
    let req = test::TestRequest::get()
        .uri(&format!("/api/forms/{}", form.id))
        .append_header(auth_header)
        .to_request();

    let resp = test::call_service(&app.app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_unauthorized_access() {
    let app = init_test_app().await;

    // Try to get forms without auth
    let req = test::TestRequest::get().uri("/api/forms").to_request();

    let resp = test::call_service(&app.app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_get_nonexistent_form() {
    let app = init_test_app().await;
    let user = create_test_user(&app.pool).await;
    let auth_header = get_auth_header(&user.id, &user.email);

    let fake_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/forms/{}", fake_id))
        .append_header(auth_header)
        .to_request();

    let resp = test::call_service(&app.app, req).await;
    assert_eq!(resp.status(), 404);
}
