# Image Downloading Feature

This document describes the image downloading functionality implemented in Ratatoskr for handling Telegram messages containing images.

## Overview

When a Telegram message contains images, Ratatoskr will automatically:
1. Detect the presence of images in the message
2. Select the highest quality version of the image
3. Download the image from Telegram's servers
4. Store it locally in a configured directory
5. Include image metadata in the Kafka message

## Configuration

### Environment Variables

Add the following environment variable to configure where images are stored:

```bash
# Optional: Directory to store downloaded images (default: ./images)
IMAGE_STORAGE_DIR=./images
```

### Directory Structure

Images are stored with the following naming convention:
```
{chat_id}_{message_id}_{file_unique_id}_{timestamp}.{extension}
```

Example:
```
-123456789_42_AgACAgIAAxkDAAIC_mF_abc123def456_1703123456.jpg
```

## Image Selection Logic

When a message contains multiple photo sizes (Telegram provides multiple resolutions), Ratatoskr selects the highest quality image based on:
1. Image dimensions (width × height)
2. The largest dimension combination is chosen

## Kafka Message Enhancement

When an image is downloaded, the original Telegram message sent to Kafka is enhanced with additional metadata:

```json
{
  // ... original Telegram message fields ...
  "downloaded_images": [
    {
      "file_id": "AgACAgIAAxkDAAIC_mF...",
      "file_unique_id": "abc123def456",
      "width": 1920,
      "height": 1080,
      "file_size": 245760,
      "local_path": "./images/-123456789_42_AgACAgIAAxkDAAIC_mF_abc123def456_1703123456.jpg"
    }
  ]
}
```

## Error Handling

- If image download fails, the error is logged but message processing continues
- The Kafka message will be sent without the `downloaded_images` field
- Directory creation errors will cause the handler to return an error
- Network errors during download are logged and handled gracefully

## Logging

The feature provides detailed logging at different levels:

### Info Level
- Image detection and download start
- Successful downloads with file paths
- Configuration details (storage directory)

### Debug Level
- Image metadata (dimensions, file IDs)
- Download progress information

### Error Level
- Download failures with detailed error messages
- Serialization errors
- File system errors

## File System Requirements

- The configured storage directory must be writable
- Sufficient disk space for image storage
- The application will create the directory structure if it doesn't exist

## Supported Image Formats

Ratatoskr supports all image formats that Telegram accepts:
- JPEG
- PNG
- WebP
- Any other format supported by Telegram

The file extension is preserved from the original Telegram file path.

## Performance Considerations

- Images are downloaded asynchronously
- Each image download is independent
- Large images may take longer to download
- Network timeouts are handled by the underlying HTTP client
- Downloads happen sequentially per message (not in parallel)

## Security Considerations

- Images are downloaded using Telegram's official API
- File paths are sanitized to prevent directory traversal
- Only authenticated bot tokens can access files
- Local file permissions follow system defaults

## Example Usage

1. Set up environment:
```bash
export IMAGE_STORAGE_DIR="/var/ratatoskr/images"
```

2. Send a photo via Telegram to your bot

3. Check the logs for download confirmation:
```
INFO ratatoskr::telegram_handlers: Downloading image from Telegram message message_id=123 chat_id=-456 file_id="AgACAgIAAxk..."
INFO ratatoskr::utils: Image downloaded successfully file_id="AgACAgIAAxk..." local_path="/var/ratatoskr/images/-456_123_abc123_1703123456.jpg"
```

4. The enhanced message appears in your Kafka topic with image metadata

## Troubleshooting

### Common Issues

**Permission Denied**
- Ensure the storage directory is writable
- Check file system permissions

**Network Errors**
- Verify internet connectivity
- Check if Telegram API is accessible
- Validate bot token permissions

**Disk Space**
- Monitor available disk space
- Implement log rotation if needed
- Consider cleanup policies for old images

### Debug Steps

1. Check logs for error messages
2. Verify environment variables are set correctly
3. Test directory creation manually
4. Ensure bot has access to file downloads (check with @BotFather)