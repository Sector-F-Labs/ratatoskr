// Wrapper types to distinguish between different string parameters in dependency injection
#[derive(Debug, Clone)]
pub struct KafkaInTopic(pub String);

#[derive(Debug, Clone)]
pub struct ImageStorageDir(pub String);