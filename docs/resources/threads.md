### Thread Object

##### Thread Structure

| Field            | Type                              | Description                                      |
|------------------|-----------------------------------|--------------------------------------------------|
| id               | snowflake                         | The ID of the thread                             |
| author           | [User](./users.md#user-structure) | The author of the thread                         |
| category_id      | snowflake                         | The ID of the category the thread was created in |
| title            | string                            | The title of the thread                          |
| flags            | [Thread Flags](#thread-flags)     | The thread's flags                               |
| original_message | [Message](#message-structure)     | The message the thread is referenced to          |

##### Thread Flags

| Value    | Name     | Description                                |
|----------|----------|--------------------------------------------|
| `1 << 0` | `PINNED` | The thread is at the top of category       |
| `1 << 1` | `LOCKED` | The thread isn't open for further messages |
| `1 << 2` | `NSFW`   | The thread contains NSFW content           |

##### Message Structure

| Field                 | Type                              | Description                                  |
|-----------------------|-----------------------------------|----------------------------------------------|
| id                    | snowflake                         | The ID of the message                        |
| author                | [User](./users.md#user-structure) | The author of the message                    |
| thread_id             | snowflake                         | The ID of the thread the message was sent to |
| content               | string                            | Contents of the message                      |
| flags                 | [Message Flags](#message-flags)   | The message's flags                          |
| referenced_message_id | ?snowflake                        | The source of a reply message                |
| updated_at            | ?timestamp                        | When this message was last edited            |

##### Message Flags

| Value    | Name           | Description                                              |
|----------|----------------|----------------------------------------------------------|
| `1 << 0` | `UNDELETEABLE` | This message cannot be deleted (original thread message) |
| `1 << 1` | `SYSTEM`       | The message was created by system user                   |

### Endpoints

#### Get Thread
```http
GET /threads/{thread.id}
```
Returns the [thread](#thread-structure) object.

#### Delete Thread
```http
DELETE /threads/{thread.id}
```
Deletes the [thread](#thread-structure) by given ID.

#### Get Message
```http
GET /threads/{thread.id}/messages/{message.id}
```
Returns the [message](#message-structure) by given ID from given thread.

#### Delete Message
```http
DELETE /threads/{thread.id}/messages/{message.id}
```
Deleted the [message](#message-structure) by given ID from given thread.

#### Get Thread Messages
```http
GET /threads/{thread.id}/messages/{message.id}
```
Returns a list of [messages](#message-structure) by given ID from given thread.

##### JSON Query

| Field  | Type   | Description                                          |
|--------|--------|------------------------------------------------------|
| limit  | number | Max number of messages to return (1-100, default 50) |
| after  | number | Get messages after this message ID                   |
| before | number | Get messages before this messages ID                 |

#### Modify Message
```http
PATCH /threads/{thread.id}/messages/{message.id}
```
Modifies [message](#message-structure) by given ID from given thread.

##### JSON Payload

| Field   | Type   | Description                         |
|---------|--------|-------------------------------------|
| content | string | New message content. Max 4096 chars |

#### Create Message
```http
POST /threads/{thread.id}/messages
```
Creates new message and return [message](#message-structure) object.

##### JSON Payload

| Field                 | Type       | Description                                        |
|-----------------------|------------|----------------------------------------------------|
| content               | string     | New message content. Max 4096 chars                |
| referenced_message_id | ?snowflake | The ID of the message this message should reply to |