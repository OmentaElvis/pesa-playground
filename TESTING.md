# Pesa Playground Testing Philosophy

One rule of this project is: **Test anything that is testable.**

A robust test suite is essential for the maintainability and reliability of this project. It allows us to add new features and refactor existing code with confidence. This document outlines the structure of our tests and the utilities we use to write them.

## Test Structure

We use a two-tiered approach to testing in this project, leveraging Rust's built-in support for both unit and integration tests.

### 1. Unit Tests

Unit tests are for testing a small, isolated piece of logic.

- **What to test:** A single function, a specific algorithm, or a small, self-contained component. These tests should be "pure" and not have external dependencies like a database or network requests.
- **Where to place them:** In a `#[cfg(test)]` module at the bottom of the same file as the code being tested.
- **Example:**

  ```rust
  // in src-tauri/src/some_module.rs

  fn my_utility_function(a: i32, b: i32) -> i32 {
      a + b
  }

  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_my_utility_function() {
          assert_eq!(my_utility_function(2, 2), 4);
      }
  }
  ```

### 2. Integration Tests

Integration tests are for testing how different parts of the application work together.

- **What to test:** API endpoints, database interactions, and full user flows. These tests will verify that the core features of the application are working correctly from end to end.
- **Where to place them:** In the `src-tauri/tests/` directory. Each file in this directory is a separate integration test.
- **Structure:**

  ```
  src-tauri/
  ├── src/
  └── tests/
      ├── common/         // Shared testing utilities
      │   └── mod.rs
      └── c2b_api.rs      // Tests for the C2B API
  ```

## Testing Utilities

This section documents the various testing utilities we have designed to make writing tests easier and more consistent. All of these utilities are available in the `tests/common/` module.

### Test Database

The `setup_db()` function creates a new, isolated, in-memory SQLite database for each test run. It handles running all the necessary schema migrations.

**Usage:**

```rust
use crate::common;

#[tokio::test]
async fn my_test() -> anyhow::Result<()> {
    let db = common::setup_db().await?;
    // ... your test logic using the db connection
    Ok(())
}
```

### Data Factories

Data factories are functions that populate the test database with entities. They are designed to be flexible and configurable to suit different test scenarios.

#### `create_test_business`

Creates a new business.

**Options (`CreateTestBusinessOptions`):**
- `name: Option<String>`: The name of the business. Defaults to a random company name.
- `short_code: Option<String>`: The business shortcode. Defaults to "600000".

**Usage:**

```rust
// Create a business with default values
let business = common::create_test_business(&db, None).await?;

// Create a business with a specific shortcode
let business_with_options = common::create_test_business(
    &db,
    Some(common::CreateTestBusinessOptions {
        short_code: Some("123456".to_string()),
        ..Default::default()
    }),
)
.await?;
```

#### `create_test_project`

Creates a new project associated with a business.

**Options (`CreateTestProjectOptions`):**
- `name: Option<String>`: The name of the project. Defaults to a random company name.
- `simulation_mode: Option<SimulationMode>`: The simulation mode for the project. Defaults to `SimulationMode::AlwaysSuccess`.

**Usage:**

```rust
let business = common::create_test_business(&db, None).await?;

// Create a project with default values
let project = common::create_test_project(&db, business.id, None).await?;

// Create a project with a specific simulation mode
let project_with_options = common::create_test_project(
    &db,
    business.id,
    Some(common::CreateTestProjectOptions {
        simulation_mode: Some(SimulationMode::AlwaysFail),
        ..Default::default()
    }),
)
.await?;
```

#### `create_test_user`

Creates a new user with a linked account.

**Options (`CreateTestUserOptions`):**
- `name: Option<String>`: The user's name. Defaults to a random name.
- `phone: Option<String>`: The user's phone number. Defaults to a random phone number.
- `pin: Option<String>`: The user's PIN. Defaults to "1234".
- `balance: Option<i64>`: The user's account balance. Defaults to 10,000.

**Usage:**

```rust
// Create a user with a specific balance
let user = common::create_test_user(
    &db,
    Some(common::CreateTestUserOptions {
        balance: Some(5000),
        ..Default::default()
    }),
)
.await?;
```

#### `create_test_paybill`

Creates a new paybill account linked to a business.

**Options (`CreateTestPaybillOptions`):**
- `business_id: u32`: (Required) The ID of the business this paybill belongs to.
- `paybill_number: Option<u32>`: The paybill number. Defaults to 600000.
- `balance: Option<i64>`: The paybill's account balance. Defaults to 1,000,000.

**Usage:**

```rust
let business = common::create_test_business(&db, None).await?;
let paybill = common::create_test_paybill(
    &db,
    common::CreateTestPaybillOptions {
        business_id: business.id,
        balance: Some(100000),
        paybill_number: Some(123456),
    },
)
.await?;
```

#### `create_test_till`

Creates a new till account linked to a business.

**Options (`CreateTestTillOptions`):**
- `business_id: u32`: (Required) The ID of the business this till belongs to.
- `till_number: Option<u32>`: The till number. Defaults to 123456.
- `balance: Option<i64>`: The till's account balance. Defaults to 1,000,000.

**Usage:**

```rust
let business = common::create_test_business(&db, None).await?;
let till = common::create_test_till(
    &db,
    common::CreateTestTillOptions {
        business_id: business.id,
        balance: Some(50000),
        till_number: Some(654321),
    },
)
.await?;
```

#### `create_test_api_key`

Creates a new API key for a project.

**Usage:**

```rust
let project = common::create_test_project(&db, business.id, None).await?;
let api_key = common::create_test_api_key(&db, project.id).await?;
```

### API Test Harness

The `TestApp` struct provides a convenient way to test Axum API endpoints without running a full web server. It encapsulates the Axum `Router` and the database connection, allowing you to send simulated HTTP requests and inspect the responses.

**`TestApp` Struct:**

```rust
pub struct TestApp {
    pub router: Router,
    pub db: DatabaseConnection,
}
```

**`TestApp::new(project_id: u32)`:**

Initializes a new `TestApp` instance. It sets up an in-memory database, runs migrations, creates a mock `AppHandle`, and builds the Axum router with the provided `project_id`.

**Usage:**

```rust
use crate::common::TestApp;

#[tokio::test]
async fn my_api_test() -> anyhow::Result<()> {
    // Initialize TestApp with a project ID (e.g., 1 for a default project)
    let app = TestApp::new(1).await?;
    // ... rest of your API test logic
    Ok(())
}
```

**`TestApp::get(&self, url: &str, headers: Option<HeaderMap>)`:**

Sends a simulated `GET` request to the specified URL.

- `url`: The path of the API endpoint (e.g., "/oauth/v1/generate").
- `headers`: An optional `HeaderMap` to include custom headers (e.g., `Authorization`).

**Usage:**

```rust
use axum::http::header::HeaderMap;

// ... inside an async test function with `app` initialized ...

let mut headers = HeaderMap::new();
headers.insert(
    axum::http::header::AUTHORIZATION,
    "Basic some_base64_token".parse()?,
);

let mut response = app.get("/some/api/path", Some(headers)).await?;
assert_eq!(response.status(), 200);

let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
let json_response: serde_json::Value = serde_json::from_slice(&body)?;
// ... assert on json_response ...
```

**`TestApp::post(&self, url: &str, body: Body, headers: Option<HeaderMap>)`:**

Sends a simulated `POST` request to the specified URL with a request body.

- `url`: The path of the API endpoint (e.g., "/mpesa/c2b/v1/registerurl").
- `body`: The request body, typically `axum::body::Body::from(json_payload.to_string())`.
- `headers`: An optional `HeaderMap` to include custom headers.

**Usage:**

```rust
use axum::{body, body::Body, http::header::HeaderMap};
use serde_json::json;

// ... inside an async test function with `app` initialized ...

let request_body = json!({ "key": "value" });
let mut headers = HeaderMap::new();
headers.insert(
    axum::http::header::AUTHORIZATION,
    "Bearer some_access_token".parse()?,
);

let mut response = app
    .post(
        "/some/api/path",
        Body::from(request_body.to_string()),
        Some(headers),
    )
    .await?;
assert_eq!(response.status(), 200);

let body = body::to_bytes(response.into_body(), usize::MAX).await?;
let json_response: serde_json::Value = serde_json::from_slice(&body)?;
// ... assert on json_response ...
```
