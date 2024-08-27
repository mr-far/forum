### Category Object

##### Category Structure

| Field       | Type      | Description                    |
|-------------|-----------|--------------------------------|
| id          | snowflake | The ID of the category         |
| owner       | User      | The owner of the category      |
| title       | string    | Title of the category          |
| description | string    | Descriptions of the category   |
| locked      | bool      | Whether the category is locked |

### Endpoints

#### Get Category
```http
GET /categories/{category.id}
```
Returns the [category](#category-structure) object.

#### Create Category
```http
POST /categories
```
Creates new category and return [category](#category-structure) object.

##### JSON Payload

| Field       | Type      | Description                    |
|-------------|-----------|--------------------------------|
| title       | string    | The category's title           |
| description | string    | The category's description     |
| is_locked   | bool      | Whether the category is locked |

#### Delete Category
```http
DELETE /categories/{category.id}
```
Removes category.

#### Create Thread
```http
POST /categories/{category.id}/threads
```
Creates new thread and return [thread](./threads.md#thread-structure) object.

##### JSON Payload

| Field   | Type      | Description                    |
|---------|-----------|--------------------------------|
| title   | string    | The thread's title             |
| content | string    | The thread's topic             |
| is_nsfw | bool      | Whether the category is locked |

#### Get Threads
```http
GET /categories/{category.id}/threads
```
Returns a list of [thread](./threads.md#thread-structure) object.

##### JSON Query

| Field  | Type   | Description                                         |
|--------|--------|-----------------------------------------------------|
| limit  | number | Max number of threads to return (1-100, default 50) |
| after  | number | Get threads after this thread ID                    |
| before | number | Get threads before this thread ID                   |