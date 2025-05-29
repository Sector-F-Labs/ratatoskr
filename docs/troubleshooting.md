# Troubleshooting Guide

This guide covers common runtime errors and their solutions when running Ratatoskr.

## Kafka Connection Issues

### Error: "Message production error: InvalidTopic"

**Example Error:**
```
ERROR ratatoskr::telegram_handlers: Failed to send message to Kafka topic=./images key="message" message_id=6288 chat_id=70661797 error=Message production error: InvalidTopic (Broker: Invalid topic)
```

**Cause:** The image storage directory path is being used as the Kafka topic instead of the actual topic name. This happens because both `kafka_in_topic` and `image_storage_dir` are `Arc<String>` types, and dptree's dependency injection matches by type, not by parameter order.

**Solution:** 
1. Use wrapper types to distinguish between different string parameters:
   ```rust
   // In structs.rs
   #[derive(Debug, Clone)]
   pub struct KafkaInTopic(pub String);
   
   #[derive(Debug, Clone)]
   pub struct ImageStorageDir(pub String);
   ```

2. Update handler signatures to use the wrapper types:
   ```rust
   pub async fn message_handler(
       bot: Bot,
       msg: Message,
       producer: Arc<FutureProducer>,
       kafka_in_topic: KafkaInTopic,
       image_storage_dir: ImageStorageDir,
   ) -> Result<(), Box<dyn Error + Send + Sync>> {
   ```

3. Update main.rs to create the wrapper types:
   ```rust
   let kafka_in_topic = KafkaInTopic(kafka_in_topic_val);
   let image_storage_dir = ImageStorageDir(image_storage_dir);
   ```

**Prevention:** Use distinct wrapper types for parameters of the same underlying type in dependency injection systems.

### Error: "Kafka producer creation error"

**Example Error:**
```
ERROR ratatoskr: Kafka producer creation error: BrokerTransportFailure
```

**Cause:** Cannot connect to the Kafka broker.

**Solutions:**
1. Verify Kafka broker is running: `docker ps` or check your Kafka service
2. Check the `KAFKA_BROKER` environment variable is correct
3. Test connectivity: `telnet localhost 9092` (or your broker address)
4. Ensure no firewall is blocking the connection
5. For Docker setups, verify network connectivity between containers

### Error: "Failed to subscribe to Kafka topic"

**Example Error:**
```
ERROR ratatoskr: Failed to subscribe to Kafka topic com.sectorflabs.ratatoskr.out: TopicNotFound
```

**Cause:** The Kafka topic doesn't exist.

**Solutions:**
1. Create the topic manually: `kafka-topics.sh --create --topic com.sectorflabs.ratatoskr.out --bootstrap-server localhost:9092`
2. Enable auto-topic creation in Kafka configuration
3. Check topic names for typos in environment variables

## Telegram Bot Issues

### Error: "TELEGRAM_BOT_TOKEN not set in environment"

**Cause:** Missing or incorrectly named environment variable.

**Solutions:**
1. Set the environment variable: `export TELEGRAM_BOT_TOKEN=your_token_here`
2. Check `.env` file exists and has correct format
3. Verify token from @BotFather is correct and active

### Error: "Telegram API error: Unauthorized"

**Cause:** Invalid or expired bot token.

**Solutions:**
1. Generate a new token from @BotFather
2. Verify the token is correctly copied (no extra spaces/characters)
3. Check if the bot was deleted or deactivated

### Error: "Error sending message to Telegram"

**Example Error:**
```
ERROR ratatoskr::kafka_processing: Error sending message to Telegram chat_id=123456789 error=RequestError(Network(request))
```

**Cause:** Network connectivity issues or Telegram API problems.

**Solutions:**
1. Check internet connectivity
2. Verify the chat_id exists and bot has access
3. Check if the user blocked the bot
4. Ensure message content complies with Telegram limits (4096 characters for text)

## Image Download Issues

### Error: "Failed to download image"

**Example Error:**
```
ERROR ratatoskr::telegram_handlers: Failed to download image message_id=123 chat_id=456 file_id=AgACAgIAAxk error=HTTP 404
```

**Cause:** Image file not found on Telegram servers or bot lacks file access permissions.

**Solutions:**
1. Check bot permissions with @BotFather
2. Verify the file hasn't expired (Telegram files have limited lifetime)
3. Ensure bot has `can_read_all_group_messages` if needed

### Error: "Permission denied" when saving images

**Cause:** Insufficient filesystem permissions for image storage directory.

**Solutions:**
1. Check directory permissions: `ls -la ./images`
2. Create directory with proper permissions: `mkdir -p ./images && chmod 755 ./images`
3. Ensure the user running Ratatoskr has write access
4. For Docker: check volume mount permissions

### Error: "No space left on device"

**Cause:** Insufficient disk space for image storage.

**Solutions:**
1. Free up disk space: `df -h` to check usage
2. Implement image cleanup policies
3. Use a different storage directory with more space
4. Configure log rotation to prevent log files from consuming space

## Message Processing Issues

### Error: "Error deserializing message from Kafka payload"

**Example Error:**
```
ERROR ratatoskr::kafka_processing: Error deserializing message from Kafka payload topic=com.sectorflabs.ratatoskr.out error=missing field `message_type`
```

**Cause:** Malformed JSON in Kafka message or version mismatch between message formats.

**Solutions:**
1. Validate JSON format of messages being sent to Kafka
2. Check for required fields in the message structure
3. Use legacy format if needed for backwards compatibility
4. Enable debug logging to see raw payload: `RUST_LOG=debug`

### Error: "Image file not found" when sending ImageMessage

**Cause:** Specified image path doesn't exist or is inaccessible.

**Solutions:**
1. Verify file exists: `ls -la /path/to/image.jpg`
2. Check file permissions are readable
3. Use absolute paths instead of relative paths
4. Ensure the file wasn't moved or deleted

## Configuration Issues

### Error: "Environment variable not found"

**Solutions:**
1. Create `.env` file from `.env.example`
2. Export variables in shell: `source .env`
3. Check variable names match exactly (case-sensitive)
4. For Docker: pass environment variables with `-e` flag or docker-compose

### Error: "Invalid log level"

**Cause:** Incorrect `RUST_LOG` environment variable.

**Solutions:**
1. Use valid log levels: `error`, `warn`, `info`, `debug`, `trace`
2. Example: `RUST_LOG=info` or `RUST_LOG=ratatoskr=debug`
3. Combine multiple modules: `RUST_LOG=ratatoskr=debug,rdkafka=info`

## Performance Issues

### High Memory Usage

**Symptoms:** Gradual memory increase over time.

**Solutions:**
1. Monitor for image accumulation - implement cleanup
2. Check for Kafka consumer lag
3. Review log retention policies
4. Consider implementing message batching

### Slow Message Processing

**Symptoms:** Delays in message delivery.

**Solutions:**
1. Check Kafka broker performance
2. Monitor network latency to Telegram API
3. Implement async/parallel processing where appropriate
4. Review image download timeouts

## Debug Strategies

### Enable Detailed Logging

```bash
RUST_LOG=debug cargo run
# or for specific modules
RUST_LOG=ratatoskr=debug,rdkafka=info cargo run
```

### Test Kafka Connectivity

```bash
# Test producer
echo "test message" | kafka-console-producer.sh --broker-list localhost:9092 --topic test

# Test consumer
kafka-console-consumer.sh --bootstrap-server localhost:9092 --topic test --from-beginning
```

### Test Telegram Bot

```bash
# Test bot token
curl "https://api.telegram.org/bot<YOUR_TOKEN>/getMe"
```

### Monitor Kafka Topics

```bash
# List topics
kafka-topics.sh --list --bootstrap-server localhost:9092

# Check topic details
kafka-topics.sh --describe --topic com.sectorflabs.ratatoskr.in --bootstrap-server localhost:9092
```

### Docker Troubleshooting

```bash
# Check container logs
docker logs ratatoskr

# Check container connectivity
docker exec -it ratatoskr ping kafka

# Check environment variables
docker exec -it ratatoskr env | grep -E "(KAFKA|TELEGRAM)"
```

## Getting Help

If you're still experiencing issues:

1. Check the GitHub issues for similar problems
2. Enable debug logging and include relevant logs
3. Provide your environment details (OS, Docker version, etc.)
4. Include your configuration (sanitized of sensitive data)
5. Describe the exact steps that reproduce the issue

For performance issues, include:
- System resources (CPU, memory, disk)
- Message volume and frequency
- Network latency measurements