use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TelegramMessageSentStatus {
    pub chat_id: i64,
    pub message_id: i32,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub original_message_type: String,
    pub original_correlation_id: Option<String>,
}

impl TelegramMessageSentStatus {
    pub fn new(
        chat_id: i64,
        message_id: i32,
        status: String,
        original_message_type: String,
        original_correlation_id: Option<String>,
    ) -> Self {
        Self {
            chat_id,
            message_id,
            status,
            timestamp: Utc::now(), // Note: tests will use a fixed timestamp
            original_message_type,
            original_correlation_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json;

    #[test]
    fn test_telegram_message_sent_status_serialization() {
        let timestamp = Utc.with_ymd_and_hms(2023, 10, 27, 10, 0, 0).unwrap();
        let status_message_obj = TelegramMessageSentStatus {
            chat_id: 12345,
            message_id: 67890,
            status: "success".to_string(),
            timestamp,
            original_message_type: "TestMessage".to_string(),
            original_correlation_id: Some("corr-123".to_string()),
        };

        let serialized_result = serde_json::to_string(&status_message_obj);
        assert!(serialized_result.is_ok());
        let json_string = serialized_result.unwrap();

        // Compare as serde_json::Value to avoid issues with field ordering
        let expected_value: serde_json::Value = serde_json::from_str(
            r#"{
                "chat_id": 12345,
                "message_id": 67890,
                "status": "success",
                "timestamp": "2023-10-27T10:00:00Z",
                "original_message_type": "TestMessage",
                "original_correlation_id": "corr-123"
            }"#,
        )
        .unwrap();
        let actual_value: serde_json::Value = serde_json::from_str(&json_string).unwrap();
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn test_telegram_message_sent_status_deserialization() {
        let json_string = r#"{
            "chat_id": 12345,
            "message_id": 67890,
            "status": "delivered",
            "timestamp": "2023-10-27T10:00:00Z",
            "original_message_type": "AnotherMessage",
            "original_correlation_id": "corr-456"
        }"#;

        let deserialized_result = serde_json::from_str::<TelegramMessageSentStatus>(json_string);
        assert!(deserialized_result.is_ok());
        let status_message_obj = deserialized_result.unwrap();

        let expected_timestamp = Utc.with_ymd_and_hms(2023, 10, 27, 10, 0, 0).unwrap();

        assert_eq!(status_message_obj.chat_id, 12345);
        assert_eq!(status_message_obj.message_id, 67890);
        assert_eq!(status_message_obj.status, "delivered");
        assert_eq!(status_message_obj.timestamp, expected_timestamp);
        assert_eq!(status_message_obj.original_message_type, "AnotherMessage");
        assert_eq!(
            status_message_obj.original_correlation_id,
            Some("corr-456".to_string())
        );
    }

    #[test]
    fn test_telegram_message_sent_status_deserialization_optional_correlation_null() {
        let json_string = r#"{
            "chat_id": 78901,
            "message_id": 23456,
            "status": "pending",
            "timestamp": "2023-11-01T12:30:00Z",
            "original_message_type": "OptionalTest",
            "original_correlation_id": null
        }"#;

        let deserialized_result = serde_json::from_str::<TelegramMessageSentStatus>(json_string);
        assert!(deserialized_result.is_ok());
        let status_message_obj = deserialized_result.unwrap();

        let expected_timestamp = Utc.with_ymd_and_hms(2023, 11, 1, 12, 30, 0).unwrap();

        assert_eq!(status_message_obj.chat_id, 78901);
        assert_eq!(status_message_obj.message_id, 23456);
        assert_eq!(status_message_obj.status, "pending");
        assert_eq!(status_message_obj.timestamp, expected_timestamp);
        assert_eq!(status_message_obj.original_message_type, "OptionalTest");
        assert_eq!(status_message_obj.original_correlation_id, None);
    }

    #[test]
    fn test_telegram_message_sent_status_deserialization_optional_correlation_missing() {
        let json_string = r#"{
            "chat_id": 65432,
            "message_id": 10987,
            "status": "failed",
            "timestamp": "2023-12-25T18:45:15Z",
            "original_message_type": "MissingCorrID"
        }"#; // original_correlation_id is completely missing

        let deserialized_result = serde_json::from_str::<TelegramMessageSentStatus>(json_string);
        assert!(deserialized_result.is_ok());
        let status_message_obj = deserialized_result.unwrap();

        let expected_timestamp = Utc.with_ymd_and_hms(2023, 12, 25, 18, 45, 15).unwrap();

        assert_eq!(status_message_obj.chat_id, 65432);
        assert_eq!(status_message_obj.message_id, 10987);
        assert_eq!(status_message_obj.status, "failed");
        assert_eq!(status_message_obj.timestamp, expected_timestamp);
        assert_eq!(status_message_obj.original_message_type, "MissingCorrID");
        assert_eq!(status_message_obj.original_correlation_id, None);
    }
}
