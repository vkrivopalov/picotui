//! API integration tests using wiremock mock HTTP server
//!
//! These tests verify that the app correctly interacts with the Picodata HTTP API.

mod common;

use common::{mock_cluster_info, mock_config_no_auth, mock_config_with_auth, mock_login_success, mock_tiers};
use picotui::api::{spawn_api_worker, ApiRequest, ApiResponse};
use std::sync::mpsc::channel;
use std::time::Duration;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to wait for a response with timeout
fn recv_timeout<T>(rx: &std::sync::mpsc::Receiver<T>, timeout_ms: u64) -> Option<T> {
    rx.recv_timeout(Duration::from_millis(timeout_ms)).ok()
}

#[tokio::test]
async fn test_get_config_no_auth() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_config_no_auth()))
        .mount(&mock_server)
        .await;

    let (req_tx, req_rx) = channel();
    let (res_tx, res_rx) = channel();

    spawn_api_worker(mock_server.uri(), req_rx, res_tx, false);

    // Request config
    req_tx.send(ApiRequest::GetConfig).unwrap();

    // Wait for response
    let response = recv_timeout(&res_rx, 5000).expect("Should receive response");

    match response {
        ApiResponse::Config(Ok(config)) => {
            assert!(!config.is_auth_enabled, "Auth should be disabled");
        }
        other => panic!("Unexpected response: {:?}", other),
    }

    // Shutdown
    req_tx.send(ApiRequest::Shutdown).unwrap();
}

#[tokio::test]
async fn test_get_config_with_auth() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_config_with_auth()))
        .mount(&mock_server)
        .await;

    let (req_tx, req_rx) = channel();
    let (res_tx, res_rx) = channel();

    spawn_api_worker(mock_server.uri(), req_rx, res_tx, false);

    req_tx.send(ApiRequest::GetConfig).unwrap();

    let response = recv_timeout(&res_rx, 5000).expect("Should receive response");

    match response {
        ApiResponse::Config(Ok(config)) => {
            assert!(config.is_auth_enabled, "Auth should be enabled");
        }
        other => panic!("Unexpected response: {:?}", other),
    }

    req_tx.send(ApiRequest::Shutdown).unwrap();
}

#[tokio::test]
async fn test_get_cluster_info() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/cluster"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_cluster_info()))
        .mount(&mock_server)
        .await;

    let (req_tx, req_rx) = channel();
    let (res_tx, res_rx) = channel();

    spawn_api_worker(mock_server.uri(), req_rx, res_tx, false);

    req_tx.send(ApiRequest::GetClusterInfo).unwrap();

    let response = recv_timeout(&res_rx, 5000).expect("Should receive response");

    match response {
        ApiResponse::ClusterInfo(Ok(info)) => {
            assert_eq!(info.cluster_name, "test-cluster");
            assert_eq!(info.cluster_version, "1.0.0");
            assert_eq!(info.instances_current_state_online, 5);
            assert_eq!(info.instances_current_state_offline, 1);
        }
        other => panic!("Unexpected response: {:?}", other),
    }

    req_tx.send(ApiRequest::Shutdown).unwrap();
}

#[tokio::test]
async fn test_get_tiers() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/tiers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_tiers()))
        .mount(&mock_server)
        .await;

    let (req_tx, req_rx) = channel();
    let (res_tx, res_rx) = channel();

    spawn_api_worker(mock_server.uri(), req_rx, res_tx, false);

    req_tx.send(ApiRequest::GetTiers).unwrap();

    let response = recv_timeout(&res_rx, 5000).expect("Should receive response");

    match response {
        ApiResponse::Tiers(Ok(tiers)) => {
            assert_eq!(tiers.len(), 2, "Should have 2 tiers");
            assert_eq!(tiers[0].name, "default");
            assert_eq!(tiers[1].name, "storage");
            assert_eq!(tiers[0].replicasets.len(), 2, "Default tier should have 2 replicasets");
            assert_eq!(tiers[0].replicasets[0].instances.len(), 2, "r1 should have 2 instances");
        }
        other => panic!("Unexpected response: {:?}", other),
    }

    req_tx.send(ApiRequest::Shutdown).unwrap();
}

#[tokio::test]
async fn test_login_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/session"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_login_success()))
        .mount(&mock_server)
        .await;

    let (req_tx, req_rx) = channel();
    let (res_tx, res_rx) = channel();

    spawn_api_worker(mock_server.uri(), req_rx, res_tx, false);

    req_tx
        .send(ApiRequest::Login {
            username: "admin".to_string(),
            password: "secret".to_string(),
            remember_me: false,
        })
        .unwrap();

    let response = recv_timeout(&res_rx, 5000).expect("Should receive response");

    match response {
        ApiResponse::Login(Ok(token_resp)) => {
            assert_eq!(token_resp.auth, "test-auth-token-12345");
        }
        other => panic!("Unexpected response: {:?}", other),
    }

    req_tx.send(ApiRequest::Shutdown).unwrap();
}

#[tokio::test]
async fn test_login_failure_401() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/session"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    let (req_tx, req_rx) = channel();
    let (res_tx, res_rx) = channel();

    spawn_api_worker(mock_server.uri(), req_rx, res_tx, false);

    req_tx
        .send(ApiRequest::Login {
            username: "admin".to_string(),
            password: "wrong".to_string(),
            remember_me: false,
        })
        .unwrap();

    let response = recv_timeout(&res_rx, 5000).expect("Should receive response");

    match response {
        ApiResponse::Login(Err(msg)) => {
            assert!(
                msg.contains("Invalid username or password"),
                "Should show friendly error message, got: {}",
                msg
            );
        }
        other => panic!("Unexpected response: {:?}", other),
    }

    req_tx.send(ApiRequest::Shutdown).unwrap();
}

#[tokio::test]
async fn test_authenticated_request_sends_bearer_token() {
    let mock_server = MockServer::start().await;

    // Expect Authorization header
    Mock::given(method("GET"))
        .and(path("/api/v1/cluster"))
        .and(header("Authorization", "Bearer my-test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_cluster_info()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let (req_tx, req_rx) = channel();
    let (res_tx, res_rx) = channel();

    spawn_api_worker(mock_server.uri(), req_rx, res_tx, false);

    // Set token first
    req_tx
        .send(ApiRequest::SetToken {
            auth: "my-test-token".to_string(),
            refresh: "refresh-token".to_string(),
        })
        .unwrap();

    // Small delay to ensure token is set
    std::thread::sleep(Duration::from_millis(50));

    // Now request cluster info - should include auth header
    req_tx.send(ApiRequest::GetClusterInfo).unwrap();

    let response = recv_timeout(&res_rx, 5000).expect("Should receive response");

    match response {
        ApiResponse::ClusterInfo(Ok(info)) => {
            assert_eq!(info.cluster_name, "test-cluster");
        }
        other => panic!("Unexpected response: {:?}", other),
    }

    req_tx.send(ApiRequest::Shutdown).unwrap();
}

#[tokio::test]
async fn test_cluster_info_401_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/cluster"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&mock_server)
        .await;

    let (req_tx, req_rx) = channel();
    let (res_tx, res_rx) = channel();

    spawn_api_worker(mock_server.uri(), req_rx, res_tx, false);

    req_tx.send(ApiRequest::GetClusterInfo).unwrap();

    let response = recv_timeout(&res_rx, 5000).expect("Should receive response");

    match response {
        ApiResponse::ClusterInfo(Err(msg)) => {
            // Should contain 401 or unauthorized indication
            assert!(
                msg.contains("401") || msg.to_lowercase().contains("unauthorized"),
                "Error should indicate auth failure, got: {}",
                msg
            );
        }
        other => panic!("Unexpected response: {:?}", other),
    }

    req_tx.send(ApiRequest::Shutdown).unwrap();
}

#[tokio::test]
async fn test_server_error_500() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/cluster"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let (req_tx, req_rx) = channel();
    let (res_tx, res_rx) = channel();

    spawn_api_worker(mock_server.uri(), req_rx, res_tx, false);

    req_tx.send(ApiRequest::GetClusterInfo).unwrap();

    let response = recv_timeout(&res_rx, 5000).expect("Should receive response");

    match response {
        ApiResponse::ClusterInfo(Err(msg)) => {
            assert!(
                msg.contains("500") || msg.to_lowercase().contains("error"),
                "Error should indicate server error, got: {}",
                msg
            );
        }
        other => panic!("Unexpected response: {:?}", other),
    }

    req_tx.send(ApiRequest::Shutdown).unwrap();
}

#[tokio::test]
async fn test_connection_refused() {
    // Use a port that's definitely not running anything
    let bad_url = "http://127.0.0.1:59999";

    let (req_tx, req_rx) = channel();
    let (res_tx, res_rx) = channel();

    spawn_api_worker(bad_url.to_string(), req_rx, res_tx, false);

    req_tx.send(ApiRequest::GetConfig).unwrap();

    let response = recv_timeout(&res_rx, 10000).expect("Should receive error response");

    match response {
        ApiResponse::Config(Err(msg)) => {
            // Should indicate connection failure
            assert!(
                msg.to_lowercase().contains("connect")
                    || msg.to_lowercase().contains("refused")
                    || msg.to_lowercase().contains("failed"),
                "Error should indicate connection problem, got: {}",
                msg
            );
        }
        other => panic!("Unexpected response: {:?}", other),
    }

    req_tx.send(ApiRequest::Shutdown).unwrap();
}

#[tokio::test]
async fn test_full_flow_no_auth() {
    let mock_server = MockServer::start().await;

    // Setup all mocks
    Mock::given(method("GET"))
        .and(path("/api/v1/config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_config_no_auth()))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/cluster"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_cluster_info()))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/tiers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_tiers()))
        .mount(&mock_server)
        .await;

    let (req_tx, req_rx) = channel();
    let (res_tx, res_rx) = channel();

    spawn_api_worker(mock_server.uri(), req_rx, res_tx, false);

    // 1. Get config
    req_tx.send(ApiRequest::GetConfig).unwrap();
    let config_resp = recv_timeout(&res_rx, 5000).unwrap();
    assert!(matches!(config_resp, ApiResponse::Config(Ok(_))));

    // 2. Get cluster info
    req_tx.send(ApiRequest::GetClusterInfo).unwrap();
    let cluster_resp = recv_timeout(&res_rx, 5000).unwrap();
    assert!(matches!(cluster_resp, ApiResponse::ClusterInfo(Ok(_))));

    // 3. Get tiers
    req_tx.send(ApiRequest::GetTiers).unwrap();
    let tiers_resp = recv_timeout(&res_rx, 5000).unwrap();
    assert!(matches!(tiers_resp, ApiResponse::Tiers(Ok(_))));

    req_tx.send(ApiRequest::Shutdown).unwrap();
}

#[tokio::test]
async fn test_full_flow_with_auth() {
    let mock_server = MockServer::start().await;

    // Setup all mocks
    Mock::given(method("GET"))
        .and(path("/api/v1/config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_config_with_auth()))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/session"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_login_success()))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/cluster"))
        .and(header("Authorization", "Bearer test-auth-token-12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_cluster_info()))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/tiers"))
        .and(header("Authorization", "Bearer test-auth-token-12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_tiers()))
        .mount(&mock_server)
        .await;

    let (req_tx, req_rx) = channel();
    let (res_tx, res_rx) = channel();

    spawn_api_worker(mock_server.uri(), req_rx, res_tx, false);

    // 1. Get config - auth required
    req_tx.send(ApiRequest::GetConfig).unwrap();
    let config_resp = recv_timeout(&res_rx, 5000).unwrap();
    match config_resp {
        ApiResponse::Config(Ok(config)) => {
            assert!(config.is_auth_enabled);
        }
        _ => panic!("Expected config response"),
    }

    // 2. Login
    req_tx
        .send(ApiRequest::Login {
            username: "admin".to_string(),
            password: "secret".to_string(),
            remember_me: false,
        })
        .unwrap();
    let login_resp = recv_timeout(&res_rx, 5000).unwrap();
    match login_resp {
        ApiResponse::Login(Ok(token)) => {
            assert_eq!(token.auth, "test-auth-token-12345");
        }
        _ => panic!("Expected login response"),
    }

    // 3. Get cluster info (with auth)
    req_tx.send(ApiRequest::GetClusterInfo).unwrap();
    let cluster_resp = recv_timeout(&res_rx, 5000).unwrap();
    assert!(matches!(cluster_resp, ApiResponse::ClusterInfo(Ok(_))));

    // 4. Get tiers (with auth)
    req_tx.send(ApiRequest::GetTiers).unwrap();
    let tiers_resp = recv_timeout(&res_rx, 5000).unwrap();
    assert!(matches!(tiers_resp, ApiResponse::Tiers(Ok(_))));

    req_tx.send(ApiRequest::Shutdown).unwrap();
}
