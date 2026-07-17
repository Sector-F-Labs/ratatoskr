# Troubleshooting Guide

This guide covers common runtime errors and their solutions when running Ratatoskr.

## Kafka Connection Issues

### Error: "Kafka producer creation error"

**Example Error:**
```
ERROR ratatoskr: Kafka producer creation error: BrokerTransportFailure
```

**Cause:** Cannot connect to the Kafka broker.

**Solutions:**
1. Verify Kafka broker is running: `docker-compose up -d` (or check your Kafka service)
2. Check the `KAFKA_BROKERS` environment variable is correct
3. Test connectivity: `telnet localhost 9092` (or your broker address)
4. Ensure no firewall is blocking the connection
5. For Docker setups, verify network connectivity between containers

### Error: "Failed to subscribe to Kafka topic"

**Example Error:**
```
ERROR ratatoskr: Failed to subscribe to Kafka topic ratatoskr.out: TopicNotFound
```

**Cause:** The Kafka topic doesn't exist. Ratatoskr creates `{prefix}.in`/`{prefix}.out` automatically on startup (`ensure_topics`), so this usually means the broker was unreachable at startup.

**Solutions:**
1. Create the topic manually: `kafka-topics.sh --create --topic ratatoskr.out --bootstrap-server localhost:9092`
2. Enable auto-topic creation in Kafka configuration
3. Check `KAFKA_TOPIC_PREFIX` for typos

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

## Incoming File Attachments

Ratatoskr does not download files from Telegram — incoming `file_attachments` only carry the `file_id` and a `file_url`. If your consumer fails to fetch a file:

1. Verify the `file_url` hasn't expired (Telegram file links are time-limited; re-fetch via the Bot API using `file_id` if needed)
2. Check bot permissions with @BotFather (e.g. `can_read_all_group_messages` for group chats)
3. Ensure your consumer has network access to `api.telegram.org`

## Message Processing Issues

### Error: "Error deserializing message from Kafka payload"

**Example Error:**
```
ERROR ratatoskr::kafka_processing: Error deserializing message from Kafka payload topic=ratatoskr.out error=missing field `message_type`
```

**Cause:** Malformed JSON in Kafka message or version mismatch between message formats.

**Solutions:**
1. Validate JSON format of messages being sent to Kafka
2. Check for required fields in the message structure
3. Enable debug logging to see raw payload: `RUST_LOG=debug`

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
1. Check for Kafka consumer lag
2. Review log retention policies
3. Consider implementing message batching

### Slow Message Processing

**Symptoms:** Delays in message delivery.

**Solutions:**
1. Check Kafka broker performance
2. Monitor network latency to Telegram API
3. Implement async/parallel processing where appropriate

## Debug Strategies

### Enable Detailed Logging

```bash
RUST_LOG=debug cargo run -- serve
# or for specific modules
RUST_LOG=ratatoskr=debug,rdkafka=info cargo run -- serve
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
kafka-topics.sh --describe --topic ratatoskr.in --bootstrap-server localhost:9092
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