# Auth Session Object
Authenticating with the HFD API is performed with the `Authorization` HTTP header in the format `Authorization: TOKEN`.
##### Example User Token Authorization
`Authorization: 3h3iWjA8YPPwxoBIxV5rxNKkGpUnLg`

# Auth
### Endpoints

#### Register Account
```http
POST /auth/register
```

##### JSON payload
| Field          | Type   | Description                   |
|----------------|--------|-------------------------------|
| `username`     | string | The new account username.     |
| `display_name` | string | The new account display name. |
| `password`     | string | The new account password.     |

##### Response body
| Field   | Type                                     | Description                       |
|---------|------------------------------------------|-----------------------------------|
| `user`  | [User](./resources/users.md#user-object) | The user that session belongs to. |
| `token` | string                                   | The session token.                |

#### Login Account
```http
POST /auth/login
```

##### JSON payload
| Field      | Type   | Description                   |
|------------|--------|-------------------------------|
| `username` | string | The logging account username. |
| `password` | string | The logging account password. |

##### Response body
| Field   | Type                            | Description                       |
|---------|---------------------------------|-----------------------------------|
| `user`  | [User](./resources#user-object) | The user that session belongs to. |
| `token` | string                          | The session token.                |

#### Logout
```http
POST /auth/logout
```
Deletes the given authentication session.

##### JSON payload
| Field   | Type                            | Description                       |
|---------|---------------------------------|-----------------------------------|
| `token` | string                          | The session token.                |
