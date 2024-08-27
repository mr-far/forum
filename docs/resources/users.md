### User Object

##### User Structure

| Field        | Type      | Description             |
|--------------|-----------|-------------------------|
| id           | snowflake | The ID of the user      |
| username     | string    | The user's name         |
| display_name | ?string   | The user's display name |
| bio          | ?string   | The ID of the user      |
| permissions  | integer   | The user's permissions  |
| flags        | integer   | The user's flags        |

##### User Flags
| Value    | Name          | Description                                                                      |
|----------|---------------|----------------------------------------------------------------------------------|
| `1 << 0` | `SYSTEM`      | User is a system                                                                 |
| `1 << 1` | `STAFF`       | Forum staff or trusted user                                                      |
| `1 << 3` | `QUARANTINED` | User is temperately restricted from creating/editing messages and threads        |
| `1 << 4` | `BANNED`      | User is temperately or permanently banned (restricted from interacting with API) |
| `1 << 5` | `SPAMMER`     | User is marked as a spammer (some operation can be added in the UI)              |
| `1 << 6` | `DELETED`     | User's account is deleted                                                        |

### Endpoints

#### Get Current User
```http
GET /users/@me
```
Returns the current [user](#user-object) object.

#### Get User
```http
GET /users/{user.id}
```
Returns the [user](#user-object) object for a given user ID.
