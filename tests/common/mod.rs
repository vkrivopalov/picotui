#![allow(dead_code)]

use serde_json::json;

/// Mock cluster info JSON response
pub fn mock_cluster_info() -> serde_json::Value {
    json!({
        "capacityUsage": 30.5,
        "clusterName": "test-cluster",
        "clusterVersion": "1.0.0",
        "currentInstaceVersion": "25.6.0",
        "replicasetsCount": 2,
        "instancesCurrentStateOffline": 1,
        "instancesCurrentStateOnline": 5,
        "memory": {
            "usable": 4294967296_u64,
            "used": 1288490188_u64
        },
        "plugins": ["plugin1"]
    })
}

/// Mock tiers JSON response with multiple tiers, replicasets, and instances
pub fn mock_tiers() -> serde_json::Value {
    json!([
        {
            "name": "default",
            "replicasetCount": 2,
            "rf": 3,
            "bucketCount": 3000,
            "instanceCount": 4,
            "can_vote": true,
            "services": [],
            "memory": {
                "usable": 2147483648_u64,
                "used": 644245094_u64
            },
            "capacityUsage": 30.0,
            "replicasets": [
                {
                    "name": "r1",
                    "version": "1",
                    "state": "Online",
                    "instanceCount": 2,
                    "uuid": "uuid-r1",
                    "capacityUsage": 30.0,
                    "memory": {
                        "usable": 1073741824_u64,
                        "used": 322122547_u64
                    },
                    "instances": [
                        {
                            "name": "i1",
                            "httpAddress": "10.0.0.1:8080",
                            "version": "25.6.0",
                            "failureDomain": {"datacenter": "dc1", "rack": "r1"},
                            "isLeader": true,
                            "currentState": "Online",
                            "targetState": "Online",
                            "binaryAddress": "10.0.0.1:3301",
                            "pgAddress": "10.0.0.1:5432"
                        },
                        {
                            "name": "i2",
                            "httpAddress": "10.0.0.2:8080",
                            "version": "25.6.0",
                            "failureDomain": {"datacenter": "dc1", "rack": "r2"},
                            "isLeader": false,
                            "currentState": "Online",
                            "targetState": "Online",
                            "binaryAddress": "10.0.0.2:3301",
                            "pgAddress": "10.0.0.2:5432"
                        }
                    ]
                },
                {
                    "name": "r2",
                    "version": "1",
                    "state": "Online",
                    "instanceCount": 2,
                    "uuid": "uuid-r2",
                    "capacityUsage": 30.0,
                    "memory": {
                        "usable": 1073741824_u64,
                        "used": 322122547_u64
                    },
                    "instances": [
                        {
                            "name": "i3",
                            "httpAddress": "10.0.0.3:8080",
                            "version": "25.6.0",
                            "failureDomain": {"datacenter": "dc2", "rack": "r1"},
                            "isLeader": true,
                            "currentState": "Offline",
                            "targetState": "Online",
                            "binaryAddress": "10.0.0.3:3301",
                            "pgAddress": "10.0.0.3:5432"
                        },
                        {
                            "name": "i4",
                            "httpAddress": "10.0.0.4:8080",
                            "version": "25.6.0",
                            "failureDomain": {"datacenter": "dc2", "rack": "r2"},
                            "isLeader": false,
                            "currentState": "Online",
                            "targetState": "Online",
                            "binaryAddress": "10.0.0.4:3301",
                            "pgAddress": "10.0.0.4:5432"
                        }
                    ]
                }
            ]
        },
        {
            "name": "storage",
            "replicasetCount": 1,
            "rf": 2,
            "bucketCount": 0,
            "instanceCount": 2,
            "can_vote": false,
            "services": ["storage"],
            "memory": {
                "usable": 2147483648_u64,
                "used": 644245094_u64
            },
            "capacityUsage": 30.0,
            "replicasets": [
                {
                    "name": "s1",
                    "version": "1",
                    "state": "Online",
                    "instanceCount": 2,
                    "uuid": "uuid-s1",
                    "capacityUsage": 30.0,
                    "memory": {
                        "usable": 2147483648_u64,
                        "used": 644245094_u64
                    },
                    "instances": [
                        {
                            "name": "s1-i1",
                            "httpAddress": "10.0.1.1:8080",
                            "version": "25.6.0",
                            "failureDomain": {"datacenter": "dc1"},
                            "isLeader": true,
                            "currentState": "Online",
                            "targetState": "Online",
                            "binaryAddress": "10.0.1.1:3301",
                            "pgAddress": ""
                        },
                        {
                            "name": "s1-i2",
                            "httpAddress": "10.0.1.2:8080",
                            "version": "25.6.0",
                            "failureDomain": {"datacenter": "dc2"},
                            "isLeader": false,
                            "currentState": "Online",
                            "targetState": "Online",
                            "binaryAddress": "10.0.1.2:3301",
                            "pgAddress": ""
                        }
                    ]
                }
            ]
        }
    ])
}

/// Mock config response with auth disabled
pub fn mock_config_no_auth() -> serde_json::Value {
    json!({
        "isAuthEnabled": false
    })
}

/// Mock config response with auth enabled
pub fn mock_config_with_auth() -> serde_json::Value {
    json!({
        "isAuthEnabled": true
    })
}

/// Mock login success response
pub fn mock_login_success() -> serde_json::Value {
    json!({
        "auth": "test-auth-token-12345",
        "refresh": "test-refresh-token-67890"
    })
}

/// Convert ratatui buffer to a string for assertions
pub fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
    let mut result = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = buffer.cell((x, y)).unwrap();
            result.push_str(cell.symbol());
        }
        result.push('\n');
    }
    result
}

/// Check if buffer contains a string anywhere
pub fn buffer_contains(buffer: &ratatui::buffer::Buffer, needle: &str) -> bool {
    buffer_to_string(buffer).contains(needle)
}
